//! The types module contains all of the types relevant to the fault dispute game.

use ethers::types::{Bytes, H256};

/// The [Claim] type represents a claim on the execution trace at a given trace index that is
/// made by a participant in a dispute game.
pub type Claim = H256;

/// The [Clock] struct represents a clock that is used to track the duration and timestamp of a
/// given [Claim] within the game.
pub struct Clock {
    /// The duration remaining on the chess clock.
    pub duration: u64,
    /// The timestamp at which the clock was last updated.
    pub timestamp: u64,
}

/// The [ClaimData] struct represents a [Claim] as well as the data associated with it.
pub struct ClaimData {
    /// The index of the parent claim in the DAG array.
    pub parent_index: usize,
    /// Whether or not the current claim has ever been countered.
    pub countered: bool,
    /// The claim that is being made at the trace index relative to the position.
    pub claim: Claim,
    /// The position of the claim within the game tree.
    pub position: u128,
    /// The clock that is used to track the duration elapsed and timestamp of the claim.
    pub clock: Clock,
}

/// A [Response] is an action taken by a participant in the dispute game in response to
/// a claim made by another participant.
pub enum Response {
    /// Do nothing.
    DoNothing,
    /// Create a counter claim against the parent claim and optionally the grandparent.
    Move(bool, Claim, Option<(usize, Claim)>),
    /// Perform a VM step against the parent claim.
    Step(usize, usize, bool, Bytes, Bytes),
}
