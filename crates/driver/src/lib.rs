#![doc = include_str!("../README.md")]

use crate::abi::L2OutputOracle;
use abi::DisputeGame_Factory;
use anyhow::Result;
use async_trait::async_trait;
use ethers::{
    providers::{self, Middleware, Provider, StreamExt},
    types::Address,
};
use std::sync::Arc;

/// Contains the smart contract bindings used by the driver.
mod abi;

/// The [Driver] trait defines the interface for all driver loops that are ran by the `op-challenger` binary.
#[async_trait]
pub trait Driver {
    /// Creates a new [Driver] with the given configuration.
    async fn try_new(config: DriverConfig) -> Result<Self>
    where
        Self: Sized;

    /// Starts the [Driver] loop.
    async fn start(self) -> Result<()>;
}

/// The [DriverConfig] struct contains the configuration for the [Driver].
#[derive(Debug, Clone, Default)]
pub struct DriverConfig {
    /// The URL of the RPC endpoint used to index and send transactions.
    pub ws_endpoint: String,
    /// The address of the dispute game factory contract.
    pub dispute_game_factory: Address,
    /// The address of the L2OutputOracle contract.
    pub l2_output_oracle: Address,
}

impl DriverConfig {
    /// Creates a new [DriverConfig] with the given configuration.
    pub fn new(
        ws_endpoint: String,
        dispute_game_factory: Address,
        l2_output_oracle: Address,
    ) -> Self {
        Self {
            ws_endpoint,
            dispute_game_factory,
            l2_output_oracle,
        }
    }
}

/// The [OutputAttestationDriver] maintains the state loop for the output attestation challenge
/// agent.
#[derive(Debug, Clone)]
pub struct OutputAttestationDriver {
    /// The configuration for the driver.
    pub config: DriverConfig,
    /// The provider used to index and send transactions.
    pub(crate) provider: Arc<Provider<providers::Ws>>,
}

#[async_trait]
impl Driver for OutputAttestationDriver {
    /// Creates a new [Driver] with the given configuration.
    async fn try_new(config: DriverConfig) -> Result<Self> {
        let provider = Provider::<providers::Ws>::connect(config.ws_endpoint.clone()).await?;
        Ok(Self {
            config,
            provider: Arc::new(provider),
        })
    }

    /// Starts the [Driver] loop.
    async fn start(self) -> Result<()> {
        // Run the factory monitor and the L2OutputOracle monitor in parallel.
        tokio::try_join!(
            self.start_factory_monitor(),
            self.start_output_oracle_monitor()
        )
        .map(|_| ())
    }
}

impl OutputAttestationDriver {
    /// Starts the [DisputeGame_Factory] event monitor.
    async fn start_factory_monitor(&self) -> Result<()> {
        tracing::info!("Subscribing to DisputeGameCreated events...");

        let factory =
            DisputeGame_Factory::new(self.config.dispute_game_factory, Arc::clone(&self.provider));
        let mut stream = self
            .provider
            .subscribe_logs(&factory.dispute_game_created_filter().filter)
            .await?;

        tracing::info!("Subscribed to DisputeGameCreated events, beginning event loop.");
        while let Some(dispute_game_created) = stream.next().await {
            tracing::debug!(target: "op-challenger-driver", "DisputeGameCreated event received");
            println!("{:?}", dispute_game_created);

            todo!()
        }

        Ok(())
    }

    /// Starts the [L2OutputOracle] event monitor.
    async fn start_output_oracle_monitor(&self) -> Result<()> {
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
