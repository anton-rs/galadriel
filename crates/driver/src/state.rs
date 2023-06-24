//! The state module holds the [GlobalState] struct, which is shared between all drivers.

use op_challenger_solvers::fault::AlphabetGame;

/// The [GlobalState] struct holds all of the shared state between drivers.
#[derive(Default)]
pub struct GlobalState {
    /// A mutable vector of all [AlphabetGame]s that are currently in progress.
    pub games: Vec<AlphabetGame>,
}
