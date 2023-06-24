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
    types::{Address, U256},
};
use op_challenger_solvers::fault::{AlphabetGame, ClaimData, Clock};
use std::{sync::Arc, time::Duration};
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

            while let Some(tx) = locked_receive_ch.recv().await {
                tracing::info!(target: "tx-dispatch-driver", "Signed transaction request received in dispatch driver. Sending transaction...");
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
                            let root_claim_data = game.claim_data(U256::zero()).await?;

                            // TODO: Global state is entirely in memory, this won't do. We need to
                            // persist games to a local database and load them on startup. In
                            // addition, it'd be great to get a reverse sync mechanism going so
                            // that games that are not locally stored can be fetched and existing
                            // ongoing games can be updated.
                            tracing::info!(target: "dispute-factory-driver", "Fetched root claim data successfully. Locking global state mutex and pushing new game...");
                            self.state.lock().await.games.push(AlphabetGame {
                                address: game_addr,
                                created_at,
                                state: vec![ClaimData {
                                    parent_index: root_claim_data.0 as usize,
                                    countered: root_claim_data.1,
                                    claim: root_claim_data.2.into(),
                                    position: root_claim_data.3,
                                    clock: Clock {
                                        duration: (root_claim_data.4 >> 64) as u64,
                                        timestamp: (root_claim_data.4 & (u64::MAX as u128)) as u64,
                                    },
                                }],
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

define_driver!(
    FaultGamePlayer,
    (|self: FaultGamePlayer| {
        async move {
            loop {
                tracing::info!(target: "fault-game-player", "Checking for updates in ongoing FaultDisputeGames...");

                // TODO: Look for ongoing disputes.

                // Check again in 30 seconds.
                tracing::debug!(target: "fault-game-player", "Done checking for updates. Sleeping for 30 seconds...");
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        }
    })
);
