#![doc = include_str!("../README.md")]

use std::sync::Arc;

use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser};
use ethers::providers::{Provider, Ws};
use ethers::types::Address;
use op_challenger_driver::config::DriverConfig;
use op_challenger_driver::drivers::{DisputeDriver, OutputAttestationDriver};
use op_challenger_driver::{driver_stack, Driver};
use tracing::Level;

/// Arguments for the `op-challenger` binary.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Verbosity level (0-4)
    #[arg(long, short, help = "Verbosity level (0-4)", action = ArgAction::Count, env = "VERBOSITY")]
    v: u8,

    /// The Websocket RPC endpoint used to index and send transactions.
    #[arg(
        long,
        short,
        help = "The Websocket RPC endpoint used to index and send transactions.",
        env = "OP_CHALLENGER_WS"
    )]
    ws_endpoint: String,

    /// The address of the dispute game factory contract.
    #[arg(
        long,
        short,
        help = "The address of the dispute game factory contract.",
        env = "OP_CHALLENGER_DGF"
    )]
    dispute_game_factory: Address,

    /// The address of the L2OutputOracle contract.
    #[arg(
        long,
        short,
        help = "The address of the L2OutputOracle contract.",
        env = "OP_CHALLENGER_L2OO"
    )]
    l2_output_oracle: Address,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse the command arguments
    let Args {
        v,
        ws_endpoint,
        dispute_game_factory,
        l2_output_oracle,
    } = Args::parse();

    // Initialize the tracing subscriber
    init_tracing_subscriber(v)?;

    // Create the driver config.
    let driver_config = DriverConfig::new(ws_endpoint, dispute_game_factory, l2_output_oracle);
    tracing::info!(target: "op-challenger-cli", "Driver config created successfully.");

    // Connect to the websocket endpoint.
    tracing::debug!(target: "op-challenger-cli", "Connecting to websocket endpoint...");
    let ws_endpoint = Arc::new(Provider::<Ws>::connect(driver_config.ws_endpoint.clone()).await?);
    tracing::info!(target: "op-challenger-cli", "Websocket connected successfully @ {}", &driver_config.ws_endpoint);

    // Create the driver stack and start it.
    driver_stack!(
        driver_config,
        ws_endpoint,
        DisputeDriver,
        OutputAttestationDriver,
    )
    .start_drivers()
    .await?;

    Ok(())
}

/// Initializes the tracing subscriber
///
/// # Arguments
/// * `verbosity_level` - The verbosity level (0-4)
///
/// # Returns
/// * `Result<()>` - Ok if successful, Err otherwise.
fn init_tracing_subscriber(verbosity_level: u8) -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(match verbosity_level {
            0 => Level::ERROR,
            1 => Level::WARN,
            2 => Level::INFO,
            3 => Level::DEBUG,
            _ => Level::TRACE,
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber).map_err(|e| anyhow!(e))
}
