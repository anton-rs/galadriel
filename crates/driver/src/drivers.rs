//! The `driver` module contains implementations of the [Driver] trait.

use crate::{
    abi::{DisputeGame_Factory, L2OutputOracle},
    types::GameType,
    Driver, DriverConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use ethers::prelude::{SignerMiddleware, Wallet};
use ethers::providers::{Provider, Ws};
use ethers::{
    core::k256::ecdsa::SigningKey,
    providers::{Middleware, StreamExt},
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
            /// The provider used to index and send transactions by all drivers.
            pub(crate) provider: Arc<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>,
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
            pub fn new(
                config: Arc<DriverConfig>,
                provider: Arc<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>,
            ) -> Self {
                Self { config, provider }
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
                match tx.send().await {
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
                Arc::clone(&self.provider),
            );
            let mut stream = self
                .provider
                .subscribe_logs(&factory.dispute_game_created_filter().filter)
                .await?;

            tracing::info!(target: "dispute-factory-driver", "Subscribed to DisputeGameCreated events, beginning event loop.");
            while let Some(dispute_game_created) = stream.next().await {
                tracing::debug!(target: "dispute-factory-driver", "DisputeGameCreated event received");

                // The DisputeGameCreated event contains a `gameType` field, which is a `GameType`.
                let game_type_raw = dispute_game_created.topics.get(1).ok_or(anyhow::anyhow!(
                    "DisputeGameCreated event did not contain a game type"
                ))?;
                // A [GameType] will always be a u8, so we can safely index the last byte in the
                // topic.
                let game_type_u8 = game_type_raw[31];

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
                            // TODO: If the dispute game type is `OutputAttestation`, check the `rootClaim`
                            // to see if we disagree with it. If we do, provide a signed message of the
                            // `rootClaim` to the `challenge` function on the dispute game contract.
                        }
                    }
                } else {
                    tracing::error!(target: "dispute-factory-driver", "DisputeGameCreated event contained an unknown game type: {}", game_type_u8);
                    continue;
                }

                // TODO: Track the dispute game contract address and the dispute game type in a
                // local database.

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
            let oracle =
                L2OutputOracle::new(self.config.l2_output_oracle, Arc::clone(&self.provider));
            let mut stream = self
                .provider
                .subscribe_logs(&oracle.output_proposed_filter().filter)
                .await?;

            tracing::info!(target: "output-attestation-driver", "Subscribed to OutputProposed events, beginning event loop.");
            while let Some(output_proposed) = stream.next().await {
                tracing::debug!(target: "output-attestation-driver", "OutputProposed event received");

                // TODO: Call `optimism_outputAtBlock` on the trusted RPC endpoint for the l2 block
                // number in the event. If the output is not the same as the one proposed, challenge
                // the output proposal. Check the mempool for any pending challenges before sending
                // our own - we don't want to waste ETH on a duplicated tx.
                // https://github.com/ethereum-optimism/optimism/blob/7354398f07b132cd8d5431af59b13ac39d25b3b8/op-node/node/api.go#L87

                dbg!(output_proposed);
            }

            Ok(())
        }
    })
);
