//! SQLite-backed implementation of `ReputationStore` using an async connection pool.

#[cfg(feature = "persist-sqlite")]
use crate::{AsyncReputationStore, ReputationStore};
#[cfg(all(feature = "persist-sqlite", feature = "async"))]
use async_trait::async_trait;
#[cfg(feature = "persist-sqlite")]
use icn_common::{CommonError, Did};
#[cfg(feature = "persist-sqlite")]
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
#[cfg(feature = "persist-sqlite")]
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(feature = "persist-sqlite")]
#[derive(Debug, Clone)]
pub struct SqliteReputationStore {
    pool: SqlitePool,
}

#[cfg(feature = "persist-sqlite")]
impl SqliteReputationStore {
    pub async fn new(path: PathBuf) -> Result<Self, CommonError> {
        let url = format!("sqlite://{}", path.to_string_lossy());
        let options = SqliteConnectOptions::from_str(&url)
            .map_err(|e| CommonError::DatabaseError(format!("Invalid DB url: {e}")))?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        sqlx::query("CREATE TABLE IF NOT EXISTS reputation (did TEXT PRIMARY KEY, score INTEGER)")
            .execute(&pool)
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Failed to create table: {e}")))?;
        Ok(Self { pool })
    }

    async fn read_score(&self, did: &Did) -> Result<u64, CommonError> {
        let row = sqlx::query("SELECT score FROM reputation WHERE did=?")
            .bind(did.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Failed to read score: {e}")))?;
        Ok(row.map(|r| r.get::<i64, _>(0) as u64).unwrap_or(0))
    }

    async fn write_score(&self, did: &Did, score: u64) -> Result<(), CommonError> {
        sqlx::query(
            "INSERT INTO reputation(did, score) VALUES(?, ?) \
             ON CONFLICT(did) DO UPDATE SET score=excluded.score",
        )
        .bind(did.to_string())
        .bind(score as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| CommonError::DatabaseError(format!("Failed to write score: {e}")))?;
        Ok(())
    }
}

#[cfg(feature = "persist-sqlite")]
impl ReputationStore for SqliteReputationStore {
    fn get_reputation(&self, did: &Did) -> u64 {
        tokio::runtime::Handle::try_current()
            .map(|h| h.block_on(self.read_score(did)))
            .unwrap_or_else(|_| {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(self.read_score(did))
            })
            .unwrap_or(0)
    }

    fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64) {
        let fut = async {
            let current = self.read_score(executor).await.unwrap_or(0);
            let base: i64 = if success { 1 } else { -1 };
            let delta: i64 = base + (cpu_ms / 1000) as i64;
            let updated = (current as i64) + delta;
            let new_score = if updated < 0 { 0 } else { updated as u64 };
            let _ = self.write_score(executor, new_score).await;
        };
        match tokio::runtime::Handle::try_current() {
            Ok(h) => h.block_on(fut),
            Err(_) => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(fut);
            }
        }
    }

    fn record_proof_attempt(&self, prover: &Did, success: bool) {
        let fut = async {
            let current = self.read_score(prover).await.unwrap_or(0);
            let delta: i64 = if success { 1 } else { -1 };
            let updated = (current as i64) + delta;
            let new_score = if updated < 0 { 0 } else { updated as u64 };
            let _ = self.write_score(prover, new_score).await;
        };
        match tokio::runtime::Handle::try_current() {
            Ok(h) => h.block_on(fut),
            Err(_) => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(fut);
            }
        }
    }
}

#[cfg(all(feature = "persist-sqlite", feature = "async"))]
#[async_trait]
impl AsyncReputationStore for SqliteReputationStore {
    async fn get_reputation(&self, did: &Did) -> u64 {
        self.read_score(did).await.unwrap_or(0)
    }

    async fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64) {
        let current = self.read_score(executor).await.unwrap_or(0);
        let base: i64 = if success { 1 } else { -1 };
        let delta: i64 = base + (cpu_ms / 1000) as i64;
        let updated = (current as i64) + delta;
        let new_score = if updated < 0 { 0 } else { updated as u64 };
        let _ = self.write_score(executor, new_score).await;
    }

    async fn record_proof_attempt(&self, prover: &Did, success: bool) {
        let current = self.read_score(prover).await.unwrap_or(0);
        let delta: i64 = if success { 1 } else { -1 };
        let updated = (current as i64) + delta;
        let new_score = if updated < 0 { 0 } else { updated as u64 };
        let _ = self.write_score(prover, new_score).await;
    }
}
