use crate::{bindings::DisputeGame_Factory, utils, DriverConfig, GameType, SignerMiddlewareWS};
use anyhow::Result;
use ethers::{
    abi::Token,
    providers::Middleware,
    types::{Log, Transaction, H256, U256},
};
use std::sync::Arc;

/// Handles the `OutputProposed` event emitted by the [L2OutputOracle]. This event is emitted when
/// the [L2OutputOracle] receives a new output root from a proposer. This function will compare the
/// output root proposed to L1 to the output root given to us by our trusted RPC. If the output roots
/// do not match, the function will create a new output attestation game if there is not already a
/// creation tx in the mempool.
#[allow(dead_code)]
pub async fn output_proposed(
    config: Arc<DriverConfig>,
    factory: &DisputeGame_Factory<SignerMiddlewareWS>,
    output_proposed: Log,
) -> Result<()> {
    let proposed_root = output_proposed.topics.get(1).ok_or(anyhow::anyhow!(
        "Critical failure: Output Root topic not present in `OutputProposed` event."
    ))?;
    // Convert the H256 representing the l2 block number into a u64.
    let proposed_block = &output_proposed
        .topics
        .get(3)
        .ok_or(anyhow::anyhow!(
            "Critical failure: L2 Block Number topic not present in `OutputProposed` event."
        ))?
        .to_low_u64_be();

    match utils::compare_output_root(
        Arc::clone(&config.node_provider),
        proposed_root,
        *proposed_block,
    )
    .await
    {
        Ok((matches, output_at_block)) => {
            // Compare the output root proposed to L1 to the output root given to us by
            // our trusted RPC.
            if matches {
                tracing::debug!(target: "output-attestation-driver", "Output proposed on L1 for L2 block #{} matches output at block on trusted L2 RPC.", proposed_block);
            } else {
                tracing::warn!(target: "output-attestation-driver", "Output proposed by L1 does not match output at block on trusted node. L1: {:?}, L2: {:?}", proposed_root, output_at_block.output_root);

                // Check to see if someone has already challenged this output proposal.
                tracing::debug!(target: "output-attestation-driver", "Checking to see if a challenge has already been submitted to L1 for the disagreed upon output...");
                let tx_pool_content = config.l1_provider.txpool_content().await?;
                let is_pending_challenge =
                    // Only check pending transactions that are about to be included.
                    tx_pool_content.pending.values().any(|txs| {
                        // TODO: This is a bit messy - we may want to pull out this
                        // logic into a generic mempool search function.
                        txs.values().any(|tx| {
                            let Transaction { to, input, .. } = tx;
                            *to == Some(config.dispute_game_factory)
                                && input.starts_with(&[0x31, 0x42, 0xe5, 0x5e])
                                && U256::from(&input[4..36])
                                == U256::from(GameType::OutputAttestation as u8)
                                && H256::from_slice(&input[36..68]) == *proposed_root
                        })
                    });

                // If no one has challenged this output proposal, submit a challenge.
                // If someone has already sent a transaction to challenge this output,
                // we can safely ignore it as the `DisputeFactoryDriver` will pick up
                // the challenge and submit a response.
                if !is_pending_challenge {
                    tracing::info!(target: "output-attestation-driver", "No pending challenge found, submitting challenge to L1.");

                    // TODO: Consult cannon.
                    let initial_claim = [0u8; 32];

                    // Send a challenge creation transaction to the L1 dispute game factory.
                    config
                        .tx_sender
                        .send(
                            factory
                                .create(
                                    GameType::Fault as u8,
                                    initial_claim,
                                    ethers::abi::encode(&[Token::Uint(U256::from(
                                        *proposed_block,
                                    ))])
                                    .into(),
                                )
                                .tx,
                        )
                        .await?;
                } else {
                    tracing::debug!(target: "output-attestation-driver", "Pending challenge found, waiting for the game to be created.")
                }
            }
        }
        Err(e) => {
            // Soft failure, log the error and continue.
            tracing::error!(target: "output-attestation-driver", "Error getting output from node: {}", e);
        }
    }

    Ok(())
}

// /// Handles the `DisputeGameCreated` event emitted by the [DisputeGame_Factory] contract for the
// /// [GameType::OutputAttestation] game type.
// pub async fn game_created_output_attestation(
//     config: Arc<DriverConfig>,
//     game_addr: Address,
// ) -> Result<()> {
//     let game = DisputeGame_OutputAttestation::new(game_addr, Arc::clone(&config.l1_provider));
//     let self_is_creator = game.challenges(config.l1_provider.address()).call().await?;
//
//     if self_is_creator {
//         tracing::info!(target: "dispute-factory-driver", "Not challenging in game {}, you created it.", game_addr);
//     } else {
//         tracing::info!(target: "dispute-factory-driver", "Reviewing root claim in game {}", game_addr);
//
//         // TODO: If the dispute game type is `OutputAttestation`, check the `rootClaim`
//         // to see if we disagree with it. If we do, provide a signed message of the
//         // `rootClaim` to the `challenge` function on the dispute game contract.
//
//         let root_claim = H256::from(game.root_claim().call().await?);
//         let l2_block_number = game.l2_block_number().call().await?;
//
//         let (matches, _) = utils::compare_output_root(
//             Arc::clone(&config.node_provider),
//             &root_claim,
//             l2_block_number.as_u64(),
//         )
//         .await?;
//
//         if matches {
//             tracing::info!(target: "dispute-factory-driver", "Root claim in game {} matches the node's output root at block {}. Not offering a challenge.", game_addr, l2_block_number);
//         } else {
//             tracing::warn!(target: "dispute-factory-driver", "Root claim in game {} does not match the trusted node's output root at block {}.", game_addr, l2_block_number);
//
//             // Sign the root claim.
//             let signed_root = config.l1_provider.signer().sign_hash(root_claim)?;
//             tracing::debug!(target: "dispute-factory-driver", "Signed root claim successfully.");
//
//             // Submit a challenge to the root claim.
//             tracing::info!(
//                 "Challenging game {} with root claim {} at block {}.",
//                 game_addr,
//                 root_claim,
//                 l2_block_number
//             );
//             config
//                 .tx_sender
//                 .send(
//                     game.challenge(
//                         ethers::abi::encode(&[
//                             Token::Uint(signed_root.r),
//                             Token::Uint(signed_root.s),
//                             Token::Uint(signed_root.v.into()),
//                         ])
//                         .into(),
//                     )
//                     .tx,
//                 )
//                 .await?;
//         }
//     }
//
//     Ok(())
// }
