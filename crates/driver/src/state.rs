//! The state module holds the [GlobalState] struct, which is shared between all drivers.

use op_challenger_solvers::fault::AlphabetGame;
use serde::{Deserialize, Serialize};

/// The [GlobalState] struct holds all of the shared state between drivers.
#[derive(Default, Serialize, Deserialize)]
pub struct GlobalState {
    /// A mutable vector of all [AlphabetGame]s that are currently in progress.
    pub alphabet_games: Vec<AlphabetGame>,
}
