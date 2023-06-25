//! The `config` module contains the [DriverConfig].

use crate::SignerMiddlewareWS;
use ethers::{
    providers::{Http, Provider},
    types::{transaction::eip2718::TypedTransaction, Address},
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// The [DriverConfig] struct contains the configuration for the [Driver](crate::Driver) implementations.
pub struct DriverConfig {
    /// The provider used to index events and send transactions on L1.
    pub l1_provider: Arc<SignerMiddlewareWS>,
    /// The provider for the trusted op-node endpoint used to compare proposed outputs against.
    /// This RPC should be 100% trusted- the bot will use this endpoint as the source of truth
    /// for the L2 chain in output attestation games.
    pub node_provider: Arc<Provider<Http>>,
    /// The address of the dispute game factory contract.
    pub dispute_game_factory: Address,
    /// The address of the L2OutputOracle contract.
    pub l2_output_oracle: Address,
    /// The sending handle of the MPSC channel used to send transactions.
    pub tx_sender: mpsc::Sender<TypedTransaction>,
    /// The receiving handle of the MPSC channel used to send transactions.
    pub tx_receiver: Mutex<mpsc::Receiver<TypedTransaction>>,
}

impl DriverConfig {
    /// Creates a new [DriverConfig] with the given configuration.
    pub fn new(
        l1_provider: Arc<SignerMiddlewareWS>,
        node_provider: Arc<Provider<Http>>,
        dispute_game_factory: Address,
        l2_output_oracle: Address,
    ) -> Self {
        // Create a new MPSC channel for sending transactions from the drivers.
        let (tx_sender, tx_receiver) = mpsc::channel(128);

        Self {
            l1_provider,
            node_provider,
            dispute_game_factory,
            l2_output_oracle,
            tx_sender,
            tx_receiver: Mutex::new(tx_receiver),
        }
    }
}
