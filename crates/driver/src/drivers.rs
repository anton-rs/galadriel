//! The `driver` module contains implementations of the [Driver] trait.

use crate::{
    abi::{DisputeGame_Factory, L2OutputOracle},
    handlers,
    types::GameType,
    utils, Driver, DriverConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use ethers::types::H256;
use ethers::{
    providers::{Middleware, StreamExt},
    types::{Address, Bytes, Transaction, U256},
};
use std::sync::Arc;

/// Defines a new [Driver] implementation.
#[macro_export]
macro_rules! define_driver {
    ($name:ident, $inner:expr) => {
        #[allow(dead_code)]
        #[doc = concat!("Variant of the [Driver] trait: [", stringify!($name), "]")]
        pub struct $name {
            /// The configuration for all of the drivers.
            pub config: Arc<DriverConfig>,
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
            pub fn new(config: Arc<DriverConfig>) -> Self {
                Self { config }
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
                            tracing::error!(target: "dispute-factory-driver", "DisputeGameCreated event contained a `Fault` game type, which is not yet supported");
                        }
                        GameType::Validity => {
                            tracing::error!(target: "dispute-factory-driver", "DisputeGameCreated event contained a `Validity` game type, which is not yet supported");
                        }
                        GameType::OutputAttestation => {
                            tracing::info!("DisputeGameCreated event contained an `OutputAttestation` game type, which is supported. Checking for disagreement with the root claim...");
                            handlers::output_attestation_game_created(
                                Arc::clone(&self.config),
                                game_addr,
                            )
                            .await?;
                        }
                    }
                } else {
                    tracing::error!(target: "dispute-factory-driver", "DisputeGameCreated event contained an unknown game type: {}", game_type_u8);
                    continue;
                }

                // TODO: Track the dispute game contract address and the dispute game type in a
                // local database for games that require multiple responses.

                dbg!(dispute_game_created);
            }

            Ok(())
        }
    })
);

define_driver!(
    OutputAttestationDriver,
    (|self: OutputAttestationDriver| {
        async move {
            tracing::info!(target: "output-attestation-driver", "Subscribing to OutputProposed events...");
            let oracle = L2OutputOracle::new(
                self.config.l2_output_oracle,
                Arc::clone(&self.config.l1_provider),
            );
            let factory = DisputeGame_Factory::new(
                self.config.dispute_game_factory,
                Arc::clone(&self.config.l1_provider),
            );
            let mut stream = self
                .config
                .l1_provider
                .subscribe_logs(&oracle.output_proposed_filter().filter)
                .await?;

            tracing::info!(target: "output-attestation-driver", "Subscribed to OutputProposed events, beginning event loop.");
            while let Some(output_proposed) = stream.next().await {
                tracing::debug!(target: "output-attestation-driver", "OutputProposed event received");

                let proposed_root = output_proposed.topics.get(1).ok_or(anyhow::anyhow!(
                    "Critical failure: Output Root topic not present in `OutputProposed` event."
                ))?;
                // Convert the H256 representing the l2 block number into a u64.
                let proposed_block = &output_proposed
                    .topics
                    .get(3)
                    .ok_or(anyhow::anyhow!("Critical failure: L2 Block Number topic not present in `OutputProposed` event."))?
                    .to_low_u64_be();

                // TODO: Break out below logic into handler module

                match utils::compare_output_root(
                    Arc::clone(&self.config.node_provider),
                    proposed_root,
                    *proposed_block,
                )
                .await
                {
                    Ok((matches, output_at_block)) => {
                        // Compare the output root proposed to L1 to the output root given to us by
                        // our trusted RPC.
                        if matches {
                            tracing::debug!(target: "output-attestation-driver", "Output proposed on L1 for L2 block #{} matches output at block on trusted L2 RPC.", proposed_block);
                        } else {
                            tracing::warn!(target: "output-attestation-driver", "Output proposed by L1 does not match output at block on L2. L1: {:?}, L2: {:?}", proposed_root, output_at_block.output_root);

                            // Check to see if someone has already challenged this output proposal.
                            tracing::debug!(target: "output-attestation-driver", "Checking to see if a challenge has already been submitted to L1 for the disagreed upon output...");
                            let tx_pool_content = self.config.l1_provider.txpool_content().await?;
                            let is_pending_challenge =
                                // Only check pending transactions that are about to be included.
                                tx_pool_content.pending.values().any(|txs| {
                                    // TODO: This is a bit messy - we may want to pull out this
                                    // logic into a generic mempool search function.
                                    txs.values().any(|tx| {
                                        let Transaction { to, input, .. } = tx;
                                        *to == Some(self.config.dispute_game_factory)
                                            && input.starts_with(&[0x31, 0x42, 0xe5, 0x5e])
                                            && U256::from(&input[4..36])
                                                == U256::from(GameType::OutputAttestation as u8)
                                            && H256::from_slice(&input[36..68]) == *proposed_root
                                    })
                                });

                            // If no one has challenged this output proposal, submit a challenge.
                            // If someone has already sent a transaction to challenge this output,
                            // we can safely ignore it as the `DisputeFactoryDriver` will pick up
                            // the challenge and submit a response.
                            if !is_pending_challenge {
                                tracing::info!(target: "output-attestation-driver", "No pending challenge found, submitting challenge to L1.");

                                // Send a challenge creation transaction to the L1 dispute game factory.
                                self.config
                                    .tx_sender
                                    .send(
                                        factory
                                            .create(
                                                GameType::OutputAttestation as u8,
                                                proposed_root.to_fixed_bytes(),
                                                Bytes::new(), // TODO: We may want to include the l2 block number in the extra data.
                                            )
                                            .tx,
                                    )
                                    .await?;
                            }
                        }
                    }
                    Err(e) => {
                        // Soft failure, log the error and continue.
                        tracing::error!(target: "output-attestation-driver", "Error getting output from node: {}", e);
                    }
                }
            }

            Ok(())
        }
    })
);
