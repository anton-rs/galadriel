//! The alphabet module contains an implementation of the [Game] trait for the
//! alphabet fault dispute game.

use super::{Claim, ClaimData, FaultGame, Position, Response};
use anyhow::{anyhow, Result};
use ethers::{
    abi::{self, Token},
    types::{Address, Bytes, U256},
    utils::keccak256,
};
use std::sync::Arc;

/// The maximum depth of the alphabet game.
/// TODO: This should be 63; Pad the tree.
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

impl FaultGame<u8> for AlphabetGame {
    fn respond(&self, parent_index: usize) -> Result<Response> {
        let parent = self.claim_data(parent_index)?;

        let mut is_attack = false;
        let mut secondary_move_pos = None;

        // Fetch our version of the parent claim.
        let our_parent_claim = self.claim_at(parent.position)?;

        // There are 2 possible response options to the root claim:
        // 1. Disagree with the root: Attack the root.
        // 2. Agree with the root: Do nothing.
        // There are 4 response options to a given claim that is *not* the root claim:
        // 1. Disagree with the parent, agree with grandparent: Attack the parent.
        // 2. Disagree with the parent, disagree with grandparent: Attack the parent *and* grandparent.
        // 3. Agree with the parent, disagree with grandparent: Do nothing.
        // 4. Agree with the parent, agree with grandparent: Defend the parent.
        if our_parent_claim != parent.claim {
            // We disagree with the parent; The move will always be an attack.
            is_attack = true;

            // If the parent is not the root, we check the grandparent as well.
            if parent.parent_index as u32 != u32::MAX {
                // Fetch our version of the grandparent claim.
                let grandparent = self.claim_data(parent.parent_index)?;
                let our_grandparent_claim = self.claim_at(grandparent.position)?;
                if our_grandparent_claim != grandparent.claim {
                    // Attack the grandparent as a secondary move; We disagree with it as well.
                    secondary_move_pos = Some(grandparent.position.make_move(is_attack));
                }
            }
        } else {
            // If we agree with the root claim, do nothing.
            if parent.parent_index as u32 == u32::MAX {
                return Ok(Response::DoNothing);
            }

            // Fetch our version of the grandparent claim. If we agree with it as well,
            // we defend the parent claim.
            let grandparent = self.claim_data(parent.parent_index)?;
            let our_grandparent_claim = self.claim_at(grandparent.position)?;
            if our_grandparent_claim != grandparent.claim {
                return Ok(Response::DoNothing);
            }
        }

        // Compute the position of the primary move.
        let move_pos = parent.position.make_move(is_attack);

        // If we are past the maximum depth, perform a step.
        // Otherwise, make a move.
        if move_pos.depth() > MAX_DEPTH {
            let mut state_index = 0;
            let mut state_data = Bytes::default();
            let proof = Bytes::default();

            // First, we need to find the pre/post state index within the claim data depending
            // on whether we are making an attack or defense step. If the index at depth of the
            // move position is 0, it is an attack where the prestate is the absolute prestate. In
            // this situation, the contract will determine the prestate itself and use the parent
            // claim as the poststate.
            if move_pos.index_at_depth() > 0 {
                let leaf_pos = if is_attack {
                    parent.position - 1
                } else {
                    parent.position + 1
                };

                // Search for the index of the claim that commits to the `leaf_pos`' trace index.
                // This claim must exist within the same path as the trace we're countering,
                // so we can walk up the DAG starting from the parent and find the claim that
                // commits to the same trace index as the `leaf_pos`.
                let mut state = parent;
                while state.position.right_index(MAX_DEPTH) != leaf_pos {
                    state_index = state.parent_index;
                    state = self.claim_data(state_index)?;
                }

                // Grab the state data for the prestate. The state data is the preimage for the
                // prestate claim.
                // If the move is an attack, the prestate of the step is at the trace index
                // relative to `state`.
                // If the move is a defense, the prestate of the step is at the trace index
                // relative to `parent`.
                state_data = if is_attack {
                    self.encode_claim(state.position)?
                } else {
                    self.encode_claim(parent.position)?
                }
            }

            Ok(Response::Step(
                state_index,
                parent_index,
                is_attack,
                state_data,
                proof,
            ))
        } else {
            Ok(Response::Move(
                is_attack,
                self.claim_at(move_pos)?,
                secondary_move_pos
                    .map(|pos| (parent.parent_index, self.claim_at(pos).unwrap_or_default())),
            ))
        }
    }

    fn claim_data(&self, index: usize) -> Result<&ClaimData> {
        self.state.get(index).ok_or(anyhow!("Invalid claim index"))
    }

    fn state_at(&self, position: u128) -> Result<u8> {
        self.trace
            .get(position.trace_index(MAX_DEPTH) as usize)
            .copied()
            .ok_or(anyhow!("Invalid trace index"))
    }

    fn claim_at(&self, position: u128) -> Result<Claim> {
        let claim_hash = keccak256(self.encode_claim(position)?);
        Ok(claim_hash.into())
    }
}

impl AlphabetGame {
    /// ABI encodes the pre-image for the given [Position].
    fn encode_claim(&self, position: u128) -> Result<Bytes> {
        Ok(abi::encode(&[
            Token::Uint(U256::from(position.trace_index(MAX_DEPTH))),
            Token::Uint(U256::from(self.state_at(position)?)),
        ])
        .into())
    }
}
