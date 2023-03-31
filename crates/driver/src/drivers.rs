//! The `driver` module contains implementations of the [Driver] trait.

use crate::{
    abi::{DisputeGame_Factory, L2OutputOracle},
    Driver, DriverConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use ethers::providers::{Middleware, StreamExt};
use ethers::providers::{Provider, Ws};
use std::sync::Arc;

/// Defines a new [Driver] implementation.
#[macro_export]
macro_rules! define_driver {
    ($name:ident, $inner:expr) => {
        #[doc = concat!("Variant of the [Driver] trait: [", stringify!($name), "]")]
        pub struct $name {
            /// The configuration for all of the drivers.
            pub config: Arc<DriverConfig>,
            /// The provider used to index and send transactions by all drivers.
            pub(crate) provider: Arc<Provider<Ws>>,
        }

        #[async_trait]
        impl Driver for $name {
            async fn start_loop(self) -> Result<()> {
                $inner(self).await
            }
        }

        impl $name {
            /// Creates a new [$name] with the given configuration.
            pub fn new(config: Arc<DriverConfig>, provider: Arc<Provider<Ws>>) -> Self {
                Self { config, provider }
            }
        }
    };
}

define_driver!(
    DisputeDriver,
    (|self: DisputeDriver| {
        async move {
            tracing::info!("Subscribing to DisputeGameCreated events...");

            let factory = DisputeGame_Factory::new(
                self.config.dispute_game_factory,
                Arc::clone(&self.provider),
            );
            let mut stream = self
                .provider
                .subscribe_logs(&factory.dispute_game_created_filter().filter)
                .await?;

            tracing::info!("Subscribed to DisputeGameCreated events, beginning event loop.");
            while let Some(output_proposed) = stream.next().await {
                tracing::debug!(target: "op-challenger-driver", "DisputeGameCreated event received");
                println!("{:?}", output_proposed);
            }

            Ok(())
        }
    })
);

define_driver!(
    OutputAttestationDriver,
    (|self: OutputAttestationDriver| {
        async move {
            tracing::info!("Subscribing to OutputProposed events...");

            let oracle =
                L2OutputOracle::new(self.config.l2_output_oracle, Arc::clone(&self.provider));
            let mut stream = self
                .provider
                .subscribe_logs(&oracle.output_proposed_filter().filter)
                .await?;

            tracing::info!("Subscribed to OutputProposed events, beginning event loop.");
            while let Some(output_proposed) = stream.next().await {
                tracing::debug!(target: "op-challenger-driver", "OutputProposed event received");
                println!("{:?}", output_proposed);
            }

            Ok(())
        }
    })
);
