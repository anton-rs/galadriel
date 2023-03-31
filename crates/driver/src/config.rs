//! The `config` module contains the [DriverConfig].

use crate::types::PreparedCall;
use ethers::types::Address;
use tokio::sync::{mpsc, Mutex};

/// The [DriverConfig] struct contains the configuration for the [Driver].
#[derive(Debug)]
pub struct DriverConfig {
    /// The websocket endpoint of the RPC used to index events and send transactions on L1.
    pub l1_ws_endpoint: String,
    /// The HTTP endpoint of the trusted RPC used to compare proposed outputs against.
    /// This RPC should be 100% trusted- the bot will use this endpoint as the source of truth
    /// for the L2 chain in output attestation games.
    pub trusted_op_node_endpoint: String,
    /// The sending handle of the MPSC channel used to send transactions.
    pub tx_sender: mpsc::Sender<PreparedCall>,
    /// The receiving handle of the MPSC channel used to send transactions.
    pub tx_receiver: Mutex<mpsc::Receiver<PreparedCall>>,
    /// The address of the dispute game factory contract.
    pub dispute_game_factory: Address,
    /// The address of the L2OutputOracle contract.
    pub l2_output_oracle: Address,
}

impl DriverConfig {
    /// Creates a new [DriverConfig] with the given configuration.
    pub fn new(
        l1_ws_endpoint: String,
        trusted_op_node_endpoint: String,
        dispute_game_factory: Address,
        l2_output_oracle: Address,
    ) -> Self {
        // Create a new MPSC channel for sending transactions from the drivers.
        let (tx_sender, tx_receiver) = mpsc::channel(128);

        Self {
            l1_ws_endpoint,
            trusted_op_node_endpoint,
            tx_sender,
            tx_receiver: Mutex::new(tx_receiver),
            dispute_game_factory,
            l2_output_oracle,
        }
    }
}
