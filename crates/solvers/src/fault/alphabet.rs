//! The alphabet module contains an implementation of the [Game] trait for the
//! alphabet fault dispute game.

use super::{Claim, ClaimData, Game, Position, Response};
use anyhow::{anyhow, Result};
use ethers::{
    abi::{self, Token},
    types::{Address, Bytes, U256},
    utils::keccak256,
};
use std::sync::Arc;

/// The maximum depth of the alphabet game.
/// TODO: This should be 64; Pad the tree.
const MAX_DEPTH: u64 = 4;

/// A struct containing information and the world state of a [FaultDisputeGame].
pub struct AlphabetGame {
    /// The address of the dispute game contract.
    pub address: Address,
    /// The UNIX timestamp of the game's creation.
    pub created_at: u128,
    /// The current state of the game DAG.
    pub state: Vec<ClaimData>,
    /// Our full execution trace
    pub trace: Arc<[u8]>,
}

impl Game<u8> for AlphabetGame {
    fn respond(&self, parent_index: usize) -> Result<Response> {
        let parent_claim = self
            .state
            .get(parent_index)
            .ok_or(anyhow!("Invalid parent index"))?;

        let mut secondary_move_pos = None;
        let mut is_attack = false;

        // Fetch our version of the parent claim.
        let our_parent_claim = self.claim_at(parent_claim.position)?;
        if our_parent_claim == parent_claim.claim {
            // The parent claim is valid according to our trace; Do nothing.
            return Ok(Response::DoNothing);
        }

        if parent_claim.parent_index as u32 == u32::MAX {
            // The parent claim is the root claim; Our only option is to attack it.
            is_attack = true;
        } else {
            // Fetch our version of the grandparent claim.
            let grandparent_claim = self
                .state
                .get(parent_claim.parent_index)
                .ok_or(anyhow!("Invalid grandparent index"))?;
            let our_grandparent_claim = self.claim_at(grandparent_claim.position)?;

            if our_parent_claim != parent_claim.claim {
                // Attack the parent; We disagree with it.
                is_attack = true;

                if our_grandparent_claim != grandparent_claim.claim {
                    // Attack the grandparent as a secondary move; We disagree with it as well.
                    secondary_move_pos = Some(grandparent_claim.position.make_move(is_attack));
                }
            }
        }

        // Compute the position of the primary move.
        let move_pos = parent_claim.position.make_move(is_attack);

        // If we are past the maximum depth, perform a step.
        if move_pos.depth() > MAX_DEPTH {
            let state_index = 0;
            let pre_state_preimage = Bytes::default();
            let proof = Bytes::default();

            // First, we need to find the pre/post state index within the claim data depending
            // on whether we are making an attack or defense step. If the index at depth of the
            // move position is 0, it is an attack where the prestate is the absolute prestate.
            if move_pos.index_at_depth() > 0 {
                let leaf_pos = if is_attack {
                    parent_claim.position - 1
                } else {
                    parent_claim.position + 1
                };
                let mut state_pos = leaf_pos;

                // Find the highest position that commits to the same trace index as the leaf.
                while state_pos.parent().right_index(MAX_DEPTH) == leaf_pos {
                    state_pos = state_pos.parent();
                }

                // Search for the index of the claim that commits to the prestate's trace index.
                todo!()
            }

            Ok(Response::Step(
                state_index,
                parent_index,
                is_attack,
                pre_state_preimage,
                proof,
            ))
        } else {
            // Find the trace index that our next claim must commit to.
            Ok(Response::Move(
                is_attack,
                self.claim_at(move_pos)?,
                secondary_move_pos
                    // TODO: Safeguard unwrap.
                    .map(|pos| (parent_claim.parent_index, self.claim_at(pos).unwrap())),
            ))
        }
    }

    fn state_at(&self, position: u128) -> Result<u8> {
        self.trace
            .get(position.trace_index(MAX_DEPTH) as usize)
            .copied()
            .ok_or(anyhow!("Invalid trace index"))
    }

    fn claim_at(&self, position: u128) -> Result<Claim> {
        let trace_at = self.state_at(position)?;
        let claim_hash = keccak256(abi::encode(&[
            Token::Uint(U256::from(position.trace_index(MAX_DEPTH))),
            Token::Uint(U256::from(trace_at)),
        ]));
        Ok(Claim::from(claim_hash))
    }
}
