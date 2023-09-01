//! The `driver` module contains implementations of the [Driver] trait.

use crate::{
    bindings::{DisputeGame_Factory, FaultDisputeGame},
    types::GameType,
    Driver, DriverConfig, GlobalState,
};
use anyhow::Result;
use async_trait::async_trait;
use ethers::{
    providers::{Middleware, StreamExt},
    types::{Address, H256, U256},
};
use op_challenger_solvers::fault::{AlphabetGame, ClaimData, Clock, FaultGame, Response};
use std::{cmp::Ordering, sync::Arc, time::Duration};
use tokio::sync::Mutex;

/// The trace for the alphabet game.
/// TODO: Delete this.
const TRACE: [u8; 16] = [
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
];

/// Defines a new [Driver] implementation.
#[macro_export]
macro_rules! define_driver {
    ($name:ident, $inner:expr) => {
        #[allow(dead_code)]
        #[doc = concat!("Variant of the [Driver] trait: [", stringify!($name), "]")]
        pub struct $name {
            /// The configuration for all of the drivers.
            pub config: Arc<DriverConfig>,
            /// The global state.
            pub state: Arc<Mutex<GlobalState>>,
        }

        #[async_trait]
        impl Driver for $name {
            async fn start_loop(self) -> Result<()> {
                #[allow(clippy::redundant_closure_call)]
                $inner(self).await
            }
        }

        impl $name {
            #[doc = concat!("Creates a new instance of the [", stringify!($name), "] driver.")]
            pub fn new(config: Arc<DriverConfig>, state: Arc<Mutex<GlobalState>>) -> Self {
                Self { config, state }
            }
        }
    };
}

define_driver!(
    TxDispatchDriver,
    (|self: TxDispatchDriver| {
        async move {
            tracing::info!(target: "tx-dispatch-driver", "Starting transaction dispatch driver...");
            let mut locked_receive_ch = self.config.tx_receiver.lock().await;
            tracing::info!(target: "tx-dispatch-driver", "Locked receive channel mutex successfully. Beginning tx dispatch loop.");

            while let Some(mut tx) = locked_receive_ch.recv().await {
                tracing::info!(target: "tx-dispatch-driver", "Transaction dispatch request received in dispatch driver. Sending transaction...");

                // TODO: Check the mempool and simulate the transaction prior to sending it.
                match self.config.l1_provider.estimate_gas(&tx, None).await {
                    Ok(gas) => {
                        tracing::info!(target: "tx-dispatch-driver", "Transaction simulation successful. Gas estimate: {}", gas);
                        tx.set_gas(gas);
                        tx.set_gas_price(
                            self.config
                                .l1_provider
                                .get_gas_price()
                                .await
                                .unwrap_or(U256::one())
                                * 2,
                        );
                    }
                    Err(err) => {
                        tracing::error!(target: "tx-dispatch-driver", "Error simulating transaction: {}", err);
                        continue;
                    }
                }

                match self.config.l1_provider.send_transaction(tx, None).await {
                    Ok(res) => {
                        tracing::info!(target: "tx-dispatch-driver", "Transaction sent successfully. Tx hash: {}", res.tx_hash());
                    }
                    Err(e) => {
                        // Soft failure, log the error and continue.
                        tracing::error!(target: "tx-dispatch-driver", "Error sending transaction: {}", e);
                    }
                }
            }

            Ok(())
        }
    })
);

define_driver!(
    DisputeFactoryDriver,
    (|self: DisputeFactoryDriver| {
        async move {
            tracing::info!(target: "dispute-factory-driver", "Subscribing to DisputeGameCreated events...");

            let factory = DisputeGame_Factory::new(
                self.config.dispute_game_factory,
                Arc::clone(&self.config.l1_provider),
            );
            let mut stream = self
                .config
                .l1_provider
                .subscribe_logs(&factory.dispute_game_created_filter().filter)
                .await?;

            tracing::info!(target: "dispute-factory-driver", "Subscribed to DisputeGameCreated events, beginning event loop.");
            while let Some(dispute_game_created) = stream.next().await {
                tracing::debug!(target: "dispute-factory-driver", "DisputeGameCreated event received");

                // The DisputeGameCreated event contains a `gameType` field, which is a `GameType`.
                let game_type_raw = dispute_game_created.topics.get(2).ok_or(anyhow::anyhow!(
                    "Critical failure: `gameType` field not present in `DisputeGameCreated` event."
                ))?;
                // A [GameType] will always be a u8, so we can safely index the last byte in the
                // topic.
                let game_type_u8 = game_type_raw[31];
                // The address of the created dispute game proxy.
                let game_addr: Address = Address::from_slice(&dispute_game_created.topics.get(1).ok_or(anyhow::anyhow!(
                    "Critical failure: `disputeProxy` field not present in `DisputeGameCreated` event."
                ))?[12..]);

                // Attempt to dispatch the proper response based on the game type.
                if let Ok(game_type) = GameType::try_from(game_type_u8) {
                    match game_type {
                        GameType::Fault => {
                            tracing::info!(target: "dispute-factory-driver", "New Fault game created at address {}. Fetching root claim data...", game_addr);

                            // Fetch the root claim data.
                            let game = FaultDisputeGame::new(
                                game_addr,
                                Arc::clone(&self.config.l1_provider),
                            );
                            let created_at = game.created_at().await?;

                            // TODO: Global state is entirely in memory, this won't do. We need to
                            // persist games to a local database and load them on startup. In
                            // addition, it'd be great to get a reverse sync mechanism going so
                            // that games that are not locally stored can be fetched and existing
                            // ongoing games can be updated.
                            tracing::info!(target: "dispute-factory-driver", "Fetched root claim data successfully. Locking global state mutex and pushing new game...");
                            let mut state = self.state.lock().await;
                            state.alphabet_games.push(AlphabetGame {
                                address: game_addr,
                                created_at,
                                state: Vec::default(),
                                trace: Arc::new(TRACE),
                            });
                            tracing::info!(target: "dispute-factory-driver", "Pushed new game successfully. Forwarding dispatch to the fault game driver...");
                        }
                        GameType::Validity => {
                            tracing::error!(target: "dispute-factory-driver", "DisputeGameCreated event contained a `Validity` game type, which is not yet supported");
                        }
                        GameType::OutputAttestation => {
                            tracing::error!(target: "dispute-factory-driver", "DisputeGameCreated event contained a `OutputAttestation` game type, which is not yet supported");
                        }
                    }
                } else {
                    tracing::error!(target: "dispute-factory-driver", "DisputeGameCreated event contained an unknown game type: {}", game_type_u8);
                    continue;
                }
            }

            Ok(())
        }
    })
);

// Whole thing's scuffed, mocking it out.
define_driver!(
    FaultGameWatcherDriver,
    (|self: FaultGameWatcherDriver| {
        async move {
            loop {
                tracing::info!(target: "fault-game-watcher", "Checking for updates in ongoing FaultDisputeGames...");

                let mut global_state = self.state.lock().await;
                for game in global_state.alphabet_games.iter_mut() {
                    let contract =
                        FaultDisputeGame::new(game.address, Arc::clone(&self.config.l1_provider));

                    // TODO: Resolve when clocks are out.

                    // Fetch the latest length of the claim data array in the game.
                    // TODO: Just add a getter, it's a hassle to use `eth_getStorageAt` for this.
                    // 🤮
                    let mut slot = [0u8; 32];
                    slot[31] = 0x01;
                    let length = U256::from(
                        self.config
                            .l1_provider
                            .get_storage_at(game.address, H256::from_slice(&slot), None)
                            .await?
                            .to_fixed_bytes(),
                    )
                    .as_usize();

                    let local_len = game.state.len();
                    match length.cmp(&local_len) {
                        Ordering::Greater => {
                            tracing::info!(target: "fault-game-watcher", "New claim data found in game at address {}. Fetching...", game.address);

                            // Add the new claims to the local state and process them in-order.
                            // TODO: Batch query here would reduce RPC calls by a lot.
                            for i in local_len..length {
                                // Fetch the claim data at the given index.
                                let claim_data = contract.claim_data(i.into()).await?;

                                // Add the new claim data to the local state.
                                game.state.push(ClaimData {
                                    parent_index: claim_data.0 as usize,
                                    countered: claim_data.1,
                                    claim: claim_data.2.into(),
                                    position: claim_data.3,
                                    clock: Clock {
                                        duration: (claim_data.4 >> 64) as u64,
                                        timestamp: (claim_data.4 & (u64::MAX as u128)) as u64,
                                    },
                                });

                                // TODO(perf): We can be smarter about which claims we respond to. Fetch
                                // the full state and only respond to claims that need a counter
                                // from us. Maybe a `respond_to_all` function within the `FaultGame` trait
                                // would be useful to hide this logic from the driver.
                                match game.respond(i) {
                                    Ok(res) => match res {
                                        Response::Move(is_attack, claim, _) => {
                                            tracing::debug!(target: "fault-game-watcher", "Dispatching move against claim at index={} for game at address {}", i, game.address);
                                            // TODO: This is ugly. We should have a single function to
                                            // dispatch a move.
                                            let tx = if is_attack {
                                                contract.attack(i.into(), claim.into()).tx
                                            } else {
                                                contract.defend(i.into(), claim.into()).tx
                                            };
                                            self.config.tx_sender.send(tx).await?;
                                            tracing::info!(target: "fault-game-watcher", "Dispatched move against claim at index={} for game at address {}", i, game.address);

                                            // We never need to respond to a secondary move because the
                                            // claims are processed in-order.
                                        }
                                        Response::Step(
                                            state_index,
                                            parent_index,
                                            is_attack,
                                            state_data,
                                            proof,
                                        ) => {
                                            let tx = contract
                                                .step(
                                                    state_index.into(),
                                                    parent_index.into(),
                                                    is_attack,
                                                    state_data,
                                                    proof,
                                                )
                                                .tx;
                                            self.config.tx_sender.send(tx).await?;
                                        }
                                        _ => {
                                            tracing::debug!(target: "fault-game-watcher", "No response to new claim (index: {}) at address {}", i, game.address);
                                        }
                                    },
                                    Err(e) => {
                                        tracing::error!(target: "fault-game-watcher", "Failed to formulate response to new claim data: {}", e);
                                    }
                                }
                            }
                        }
                        Ordering::Less => {
                            tracing::error!(target: "fault-game-watcher", "Local claim data length is greater than the on-chain length. This should never happen, please report this as a bug!! Local: {}, On-chain: {}", local_len, length);
                        }
                        _ => {
                            tracing::debug!(target: "fault-game-watcher", "No new claim data found in game at address {}", game.address);
                        }
                    }
                }

                // Drop the mutex lock on the global state so that other drivers may access it
                // while this thread sleeps.
                drop(global_state);

                // Check again in 5 minutes.
                tracing::debug!(target: "fault-game-watcher", "Done checking for updates. Sleeping for 5 minutes...");
                tokio::time::sleep(Duration::from_secs(60 * 5)).await;
            }
        }
    })
);
