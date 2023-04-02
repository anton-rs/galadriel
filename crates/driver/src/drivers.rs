//! The `driver` module contains implementations of the [Driver] trait.

use crate::{
    bindings::{DisputeGame_Factory, L2OutputOracle},
    handlers,
    types::GameType,
    Driver, DriverConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use ethers::{
    providers::{Middleware, StreamExt},
    types::Address,
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
                            tracing::info!("DisputeGameCreated event contained an `OutputAttestation` game type, executing handler...");
                            handlers::game_created_output_attestation(
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

                // dbg!(dispute_game_created);
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
                handlers::output_proposed(Arc::clone(&self.config), &factory, output_proposed)
                    .await?;
            }

            Ok(())
        }
    })
);
