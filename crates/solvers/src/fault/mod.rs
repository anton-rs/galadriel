//! Data structures, types, and the game solver implementation for the various
//! fault dispute game variants.

mod position;
pub use position::{compute_gindex, Position};

mod types;
pub use types::*;

mod game;
pub use game::Game;

mod alphabet;
pub use alphabet::AlphabetGame;
