//! The `config` module contains the [DriverConfig].

use ethers::types::Address;

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
