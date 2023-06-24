//! The game module holds the [Game] trait.

use crate::fault::types::{Claim, ClaimData, Response};
use anyhow::Result;

/// The [Game] trait defines the interface for a local copy of an onchain fault dispute game.
pub trait Game<T> {
    /// Respond to a [crate::Claim] made by a participant in the dispute game.
    ///
    /// ### Takes
    /// - `parent_index`: The index of the parent claim in the DAG array.
    ///
    /// ### Returns
    /// - `Ok(Response)`: The response to the claim.
    /// - `Err(anyhow::Error)`: An error occurred while determining the correct response to the
    ///    [Claim].
    fn respond(&self, parent_index: usize) -> Result<Response>;

    /// Fetch the [ClaimData] at the given index in the DAG array.
    ///
    /// ### Takes
    /// - `index`: The index of the claim in the DAG array.
    ///
    /// ### Returns
    /// - `Ok(ClaimData)`: The [ClaimData] at the given index.
    /// - `Err(anyhow::Error)`: An error occurred while fetching the [ClaimData].
    fn claim_data(&self, index: usize) -> Result<&ClaimData>;

    /// Fetch the state at the given position in the game tree. This state is always correct in
    /// the relative view of the participant.
    ///
    /// ### Takes
    /// - `position`: The position of the state within the game tree.
    ///
    /// ### Returns
    /// - `Ok(T)`: The state at the given position.
    /// - `Err(anyhow::Error)`: An error occurred while fetching the state.
    fn state_at(&self, position: u128) -> Result<T>;

    /// Fetch the [Claim] at the given position in the game tree. This [Claim] is always true in
    /// the relative view of the participant.
    ///
    /// ### Takes
    /// - `position`: The position of the claim within the game tree.
    ///
    /// ### Returns
    /// - `Ok(Claim)`: The [Claim] at the given position.
    /// - `Err(anyhow::Error)`: An error occurred while fetching the claim.
    fn claim_at(&self, position: u128) -> Result<Claim>;
}
