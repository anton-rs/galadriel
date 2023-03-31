use ethers::{
    prelude::{k256::ecdsa::SigningKey, ContractCall, SignerMiddleware},
    providers::{Provider, Ws},
    signers::Wallet,
    types::Address,
};

/// The [PreparedCall] type is a [ContractCall] that uses the [SignerMiddleware] to sign transactions.
pub type PreparedCall = ContractCall<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>, Address>;

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
