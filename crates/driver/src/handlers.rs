use crate::{abi::DisputeGame_OutputAttestation, DriverConfig};
use anyhow::Result;
use ethers::{abi::Token, types::Address};
use std::sync::Arc;

/// Handles the `DisputeGameCreated` event emitted by the [DisputeGame_Factory] contract for the
/// [GameType::OutputAttestation] game type.
pub async fn output_attestation_game_created(
    config: Arc<DriverConfig>,
    game_addr: Address,
) -> Result<()> {
    let game = DisputeGame_OutputAttestation::new(game_addr, Arc::clone(&config.l1_provider));
    let self_is_creator = game.challenges(config.l1_provider.address()).call().await?;

    if self_is_creator {
        tracing::info!(target: "dispute-factory-driver", "Not challenging in game {}, you created it.", game_addr);
    } else {
        tracing::info!(target: "dispute-factory-driver", "Reviewing root claim in game {}", game_addr);

        // TODO: If the dispute game type is `OutputAttestation`, check the `rootClaim`
        // to see if we disagree with it. If we do, provide a signed message of the
        // `rootClaim` to the `challenge` function on the dispute game contract.

        let signed_root = config
            .l1_provider
            .signer()
            .sign_hash(game.root_claim().call().await?.into())?;
        tracing::debug!(target: "dispute-factory-driver", "Signed root claim successfully.");

        config
            .tx_sender
            .send(
                game.challenge(
                    ethers::abi::encode(&[
                        Token::Uint(signed_root.r),
                        Token::Uint(signed_root.s),
                        Token::Uint(signed_root.v.into()),
                    ])
                    .into(),
                )
                .tx,
            )
            .await?;
    }

    Ok(())
}
