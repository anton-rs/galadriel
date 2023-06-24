#![doc = include_str!("../README.md")]

use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser};
use ethers::{
    prelude::{Address, Provider, SignerMiddleware, Ws},
    providers::Http,
    signers::LocalWallet,
};
use op_challenger_driver::{
    DisputeFactoryDriver, Driver, DriverConfig, GlobalState, TxDispatchDriver,
};
use std::sync::Arc;
use tokio::{sync::Mutex, task::JoinSet};
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
        help = "The Websocket RPC endpoint used to index and send transactions.",
        env = "OP_CHALLENGER_L1_WS"
    )]
    l1_ws_endpoint: String,

    /// The HTTP RPC endpoint used to compare proposed outputs against.
    /// This RPC should be 100% trusted- the bot will use this endpoint as the source of truth
    /// for the L2 chain in output attestation games.
    #[arg(
        long,
        help = "The HTTP RPC endpoint used to compare proposed outputs against.",
        env = "OP_CHALLENGER_TRUSTED_OP_NODE_RPC"
    )]
    trusted_op_node_endpoint: String,

    /// The private key used for signing transactions.
    #[arg(
        long,
        help = "The private key used for signing transactions.",
        env = "OP_CHALLENGER_KEY"
    )]
    signer_key: String,

    /// The address of the dispute game factory contract.
    #[arg(
        long,
        help = "The address of the dispute game factory contract.",
        env = "OP_CHALLENGER_DGF"
    )]
    dispute_game_factory: Address,

    /// The address of the L2OutputOracle contract.
    #[arg(
        long,
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
        l1_ws_endpoint,
        trusted_op_node_endpoint,
        signer_key,
        dispute_game_factory,
        l2_output_oracle,
    } = Args::parse();

    // Initialize the tracing subscriber
    init_tracing_subscriber(v)?;

    // Connect to the websocket endpoint.
    tracing::debug!(target: "op-challenger-cli", "Connecting to websocket endpoint...");
    let l1_endpoint = Arc::new(
        SignerMiddleware::new_with_provider_chain(
            Provider::<Ws>::connect(&l1_ws_endpoint).await?,
            signer_key.parse::<LocalWallet>()?,
        )
        .await?,
    );
    tracing::info!(target: "op-challenger-cli", "Websocket connected successfully @ {}", &l1_ws_endpoint);

    // Connect to the node endpoint.
    tracing::debug!(target: "op-challenger-cli", "Connecting to node endpoint...");
    let node_endpoint = Arc::new(Provider::<Http>::try_from(&trusted_op_node_endpoint)?);
    tracing::info!(target: "op-challenger-cli", "Node connected successfully @ {}", &trusted_op_node_endpoint);

    // Create the driver config.
    let driver_config = Arc::new(DriverConfig::new(
        l1_endpoint,
        node_endpoint,
        dispute_game_factory,
        l2_output_oracle,
    ));
    let global_state = Arc::new(Mutex::new(GlobalState::default()));
    tracing::info!(target: "op-challenger-cli", "Driver config created successfully.");

    // Creates a new driver stack and starts the driver loops.
    // TODO: Extend to support a configurable driver stack.
    macro_rules! start_driver_stack {
        ($cfg:expr, $state:expr, $($driver:ident),+ $(,)?) => {
            let mut set = JoinSet::new();

            $(set.spawn(
                $driver::new(Arc::clone(&$cfg), Arc::clone(&$state)).start_loop()
            );)*

            while let Some(result) = set.join_next().await {
                result??;
            }
        }
    }

    // Start the driver stack
    tracing::info!(target: "op-challenger-cli", "Starting driver stack...");
    start_driver_stack!(
        driver_config,
        global_state,
        TxDispatchDriver,
        DisputeFactoryDriver,
    );

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
