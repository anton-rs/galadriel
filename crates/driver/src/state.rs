//! The state module holds the [GlobalState] struct, which is shared between all drivers.

use anyhow::Result;
use op_challenger_solvers::fault::AlphabetGame;
use postgres::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// The [GlobalState] struct holds all of the shared state between drivers.
#[derive(Default, Serialize, Deserialize)]
pub struct GlobalState {
    /// A mutable vector of all [AlphabetGame]s that are currently in progress.
    pub games: Vec<AlphabetGame>,
}

impl GlobalState {
    /// Loads the global state from the database.
    pub async fn load_from_db(db_client: Mutex<Client>) -> Result<Self> {
        let db_client = &mut db_client.lock().await;

        db_client.execute(
            r#"
CREATE TABLE IF NOT EXISTS alphabet_games (
    address VARCHAR(255),
    created_at INT,
    state TEXT,
    trace TEXT
);
        "#,
            &[],
        )?;

        todo!()
    }

    /// Dumps the global state to the database.
    pub async fn dump_to_db(&self, db_client: Mutex<Client>) -> Result<()> {
        let db_client = &mut db_client.lock().await;

        todo!()
    }
}
