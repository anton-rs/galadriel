#![doc = include_str!("../README.md")]

use crate::config::DriverConfig;
use anyhow::Result;
use async_trait::async_trait;

mod abi;
pub mod config;
pub mod drivers;

/// The [Driver] trait defines the interface for all driver loops that are ran by the `op-challenger` binary.
#[async_trait]
pub trait Driver: Send + Sync {
    /// Consumes the [Driver] and starts the event loop.
    async fn start_loop(self) -> Result<()>;
}
