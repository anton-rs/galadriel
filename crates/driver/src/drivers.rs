//! The `driver` module contains implementations of the [Driver] trait.

use crate::{
    abi::{DisputeGame_Factory, L2OutputOracle},
    Driver, DriverConfig,
};
use anyhow::Result;
use ethers::providers::{self, Middleware, Provider, StreamExt, Ws};
use std::{future::Future, pin::Pin, sync::Arc};

#[derive(Default)]
pub struct DriverStack {
    pub drivers: Vec<Box<dyn Driver>>,
}

impl DriverStack {
    /// Creates a new [DriverStack].
    pub fn new(drivers: Vec<Box<dyn Driver>>) -> Self {
        Self { drivers }
    }

    /// Consumes the [DriverStack] and starts all contained [Driver]s in parallel.
    pub async fn start_drivers(self) -> Result<()> {
        for handle in self
            .drivers
            .into_iter()
            .map(|d| tokio::task::spawn(async move { d.pin_future().await }))
            .collect::<Vec<_>>()
        {
            handle.await??;
        }
        Ok(())
    }
}

/// Creates a new [DriverStack] with the given drivers.
#[macro_export]
macro_rules! driver_stack {
    ($cfg:expr, $provider:expr, $($driver:ident),+ $(,)?) => {
        {
            let drivers: Vec<Box<dyn Driver>> = vec![
                $(Box::new($driver::new($cfg.clone(), $provider.clone()))),+
            ];
            $crate::drivers::DriverStack::new(drivers)
        }
    };
}

/// The [DisputeDriver] maintains the state loop for the dispute challenge agent.
pub struct DisputeDriver {
    /// The configuration for the driver.
    pub config: DriverConfig,
    /// The provider used to index and send transactions.
    pub(crate) provider: Arc<Provider<Ws>>,
}

impl Driver for DisputeDriver {
    /// Returns the [Future] that starts the driver loop when awaited.
    fn pin_future(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(self.start_loop())
    }
}

impl DisputeDriver {
    /// Creates a new [Driver] with the given configuration.
    pub fn new(config: DriverConfig, provider: Arc<Provider<providers::Ws>>) -> Self {
        Self { config, provider }
    }

    /// Starts the [DisputeDriver] loop.
    async fn start_loop(&self) -> Result<()> {
        tracing::info!("Subscribing to DisputeGameCreated events...");

        let factory =
            DisputeGame_Factory::new(self.config.dispute_game_factory, Arc::clone(&self.provider));
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
}

/// The [OutputAttestationDriver] maintains the state loop for the output attestation challenge
/// agent.
pub struct OutputAttestationDriver {
    /// The configuration for the driver.
    pub config: DriverConfig,
    /// The provider used to index and send transactions.
    pub(crate) provider: Arc<Provider<providers::Ws>>,
}

impl Driver for OutputAttestationDriver {
    /// Returns the [Future] that starts the driver loop when awaited.
    fn pin_future(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(self.start_loop())
    }
}

impl OutputAttestationDriver {
    /// Creates a new [Driver] with the given configuration.
    pub fn new(config: DriverConfig, provider: Arc<Provider<providers::Ws>>) -> Self {
        Self { config, provider }
    }

    /// Starts the [OutputAttestationDriver] loop.
    async fn start_loop(&self) -> Result<()> {
        tracing::info!("Subscribing to OutputProposed events...");

        let factory = L2OutputOracle::new(self.config.l2_output_oracle, Arc::clone(&self.provider));
        let mut stream = self
            .provider
            .subscribe_logs(&factory.output_proposed_filter().filter)
            .await?;

        tracing::info!("Subscribed to OutputProposed events, beginning event loop.");
        while let Some(output_proposed) = stream.next().await {
            tracing::debug!(target: "op-challenger-driver", "OutputProposed event received");
            println!("{:?}", output_proposed);
        }

        Ok(())
    }
}
