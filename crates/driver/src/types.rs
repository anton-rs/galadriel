use ethers::{
    prelude::SignerMiddleware,
    providers::{Provider, Ws},
    signers::LocalWallet,
    types::H256,
};
use serde::{Deserialize, Serialize};

/// The [SignerMiddlewareWS] type is a [SignerMiddleware] that uses a [Provider] with a [Ws] transport.
pub type SignerMiddlewareWS = SignerMiddleware<Provider<Ws>, LocalWallet>;

/// The [GameType] enum defines the different types of [DisputeGames] with cloneable
/// implementations in the [DisputeGame_Factory] contract.
#[allow(dead_code)]
#[repr(u8)]
pub enum GameType {
    Fault = 0,
    Validity = 1,
    OutputAttestation = 2,
}

impl TryFrom<u8> for GameType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GameType::Fault),
            1 => Ok(GameType::Validity),
            2 => Ok(GameType::OutputAttestation),
            _ => Err(anyhow::anyhow!("Invalid game type")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputAtBlockResponse {
    pub output_root: H256,
}
