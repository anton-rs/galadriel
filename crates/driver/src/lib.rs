#![doc = include_str!("../README.md")]

use crate::config::DriverConfig;
use anyhow::Result;
use std::{future::Future, pin::Pin};

mod abi;
pub mod config;
pub mod drivers;

/// The [Driver] trait defines the interface for all driver loops that are ran by the `op-challenger` binary.
pub trait Driver: Send + Sync {
    /// Returns the [Future] that starts the driver loop when awaited.
    fn pin_future(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}
