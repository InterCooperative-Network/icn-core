//! SQLite-backed implementation of the `ReputationStore` trait.

use crate::ReputationStore;
use icn_common::{CommonError, Did};
use rusqlite::{Connection, OptionalExtension};
use std::path::PathBuf;

#[cfg(feature = "persist-sqlite")]
pub struct SqliteReputationStore {
    path: PathBuf,
}

#[cfg(feature = "persist-sqlite")]
impl SqliteReputationStore {
    /// Create a SQLite backed store for executor reputation.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let conn = Connection::open(&path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS reputation (did TEXT PRIMARY KEY, score INTEGER)",
            [],
        )
        .map_err(|e| CommonError::DatabaseError(format!("Failed to create table: {e}")))?;
        Ok(Self { path })
    }

    fn read_score(&self, did: &Did) -> u64 {
        let conn = Connection::open(&self.path).expect("open sqlite");
        conn.query_row(
            "SELECT score FROM reputation WHERE did=?1",
            [&did.to_string()],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .unwrap_or(None)
        .unwrap_or(0) as u64
    }

    fn write_score(&self, did: &Did, score: u64) {
        let conn = Connection::open(&self.path).expect("open sqlite");
        let _ = conn.execute(
            "INSERT INTO reputation(did, score) VALUES(?1, ?2) \n             ON CONFLICT(did) DO UPDATE SET score=excluded.score",
            (&did.to_string(), score as i64),
        );
    }
}

#[cfg(feature = "persist-sqlite")]
impl ReputationStore for SqliteReputationStore {
    fn get_reputation(&self, did: &Did) -> u64 {
        self.read_score(did)
    }

    fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64) {
        let current = self.read_score(executor);
        let base: i64 = if success { 1 } else { -1 };
        let delta: i64 = base + (cpu_ms / 1000) as i64;
        let updated = (current as i64) + delta;
        let new_score = if updated < 0 { 0 } else { updated as u64 };
        self.write_score(executor, new_score);
    }
}
