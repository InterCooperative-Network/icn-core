use crate::{AsyncStorageService, BlockMetadata, Cid, CommonError, DagBlock};
use deadpool_postgres::{Config as PoolConfig, ManagerConfig, Pool, RecyclingMethod};
use std::collections::HashMap;
use tokio_postgres::NoTls;

#[derive(Debug)]
pub struct PostgresDagStore {
    pool: Pool,
    meta: HashMap<Cid, BlockMetadata>,
}

impl PostgresDagStore {
    /// Connect to Postgres using the provided connection string.
    pub async fn new(conn_str: &str) -> Result<Self, CommonError> {
        let pg_cfg = conn_str
            .parse::<tokio_postgres::Config>()
            .map_err(|e| CommonError::DatabaseError(format!("Invalid connection: {e}")))?;
        let mgr_cfg = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let manager = deadpool_postgres::Manager::from_config(pg_cfg, NoTls, mgr_cfg);
        let pool = Pool::builder(manager).max_size(16).build().unwrap();
        let client = pool
            .get()
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
        client
            .batch_execute(
                "CREATE TABLE IF NOT EXISTS blocks (
                    cid TEXT PRIMARY KEY,
                    data BYTEA NOT NULL,
                    pinned BOOLEAN NOT NULL DEFAULT FALSE,
                    ttl BIGINT
                );
                 CREATE INDEX IF NOT EXISTS idx_blocks_pinned_ttl ON blocks (pinned, ttl);",
            )
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Failed to init table: {e}")))?;
        let rows = client
            .query("SELECT cid, pinned, ttl FROM blocks", &[])
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Failed to load metadata: {e}")))?;
        let mut meta = HashMap::new();
        for row in rows {
            let cid_str: String = row.get(0);
            let pinned: bool = row.get(1);
            let ttl: Option<i64> = row.get(2);
            let cid = icn_common::parse_cid_from_string(&cid_str)?;
            meta.insert(
                cid,
                BlockMetadata {
                    pinned,
                    ttl: ttl.map(|t| t as u64),
                },
            );
        }
        Ok(Self { pool, meta })
    }
}

#[async_trait::async_trait]
impl AsyncStorageService<DagBlock> for PostgresDagStore {
    async fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        icn_common::verify_block_integrity(block)?;
        let encoded = serde_json::to_vec(block).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize block {}: {e}", block.cid))
        })?;
        let meta = self.meta.get(&block.cid).cloned().unwrap_or_default();
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
        client
            .execute(
                "INSERT INTO blocks (cid, data, pinned, ttl) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (cid) DO UPDATE SET data = EXCLUDED.data, pinned = EXCLUDED.pinned, ttl = EXCLUDED.ttl",
                &[
                    &block.cid.to_string(),
                    &encoded,
                    &meta.pinned,
                    &meta.ttl.map(|t| t as i64),
                ],
            )
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store block {}: {e}", block.cid)))?;
        self.meta.insert(block.cid.clone(), meta);
        Ok(())
    }

    async fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
        let row_opt = client
            .query_opt(
                "SELECT data FROM blocks WHERE cid = $1",
                &[&cid.to_string()],
            )
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Query failed: {e}")))?;
        if let Some(row) = row_opt {
            let data: Vec<u8> = row.get(0);
            let block: DagBlock = serde_json::from_slice(&data).map_err(|e| {
                CommonError::DeserializationError(format!(
                    "Failed to deserialize block {}: {e}",
                    cid
                ))
            })?;
            if &block.cid != cid {
                return Err(CommonError::InvalidInputError(format!(
                    "CID mismatch for block read from postgres: expected {}, found {}",
                    cid, block.cid
                )));
            }
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    async fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
        client
            .execute("DELETE FROM blocks WHERE cid = $1", &[&cid.to_string()])
            .await
            .map_err(|e| {
                CommonError::DatabaseError(format!("Failed to delete block {}: {e}", cid))
            })?;
        self.meta.remove(cid);
        Ok(())
    }

    async fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
        let row = client
            .query_one(
                "SELECT COUNT(1) FROM blocks WHERE cid = $1",
                &[&cid.to_string()],
            )
            .await
            .map_err(|e| {
                CommonError::DatabaseError(format!("Failed to check block {}: {e}", cid))
            })?;
        let count: i64 = row.get(0);
        Ok(count > 0)
    }

    async fn list_blocks(&self) -> Result<Vec<DagBlock>, CommonError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
        let rows = client
            .query("SELECT data FROM blocks", &[])
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Query failed: {e}")))?;
        let mut blocks = Vec::new();
        for row in rows {
            let data: Vec<u8> = row.get(0);
            let block: DagBlock = serde_json::from_slice(&data).map_err(|e| {
                CommonError::DeserializationError(format!("Failed to deserialize block: {e}"))
            })?;
            blocks.push(block);
        }
        Ok(blocks)
    }

    async fn pin_block(&mut self, cid: &Cid) -> Result<(), CommonError> {
        match self.meta.get_mut(cid) {
            Some(m) => {
                m.pinned = true;
                let client = self
                    .pool
                    .get()
                    .await
                    .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
                client
                    .execute(
                        "UPDATE blocks SET pinned = true WHERE cid = $1",
                        &[&cid.to_string()],
                    )
                    .await
                    .map_err(|e| {
                        CommonError::DatabaseError(format!("Failed to pin block {}: {e}", cid))
                    })?;
                Ok(())
            }
            None => Err(CommonError::ResourceNotFound(format!(
                "Block {} not found",
                cid
            ))),
        }
    }

    async fn unpin_block(&mut self, cid: &Cid) -> Result<(), CommonError> {
        match self.meta.get_mut(cid) {
            Some(m) => {
                m.pinned = false;
                let client = self
                    .pool
                    .get()
                    .await
                    .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
                client
                    .execute(
                        "UPDATE blocks SET pinned = false WHERE cid = $1",
                        &[&cid.to_string()],
                    )
                    .await
                    .map_err(|e| {
                        CommonError::DatabaseError(format!("Failed to unpin block {}: {e}", cid))
                    })?;
                Ok(())
            }
            None => Err(CommonError::ResourceNotFound(format!(
                "Block {} not found",
                cid
            ))),
        }
    }

    async fn prune_expired(&mut self, now: u64) -> Result<Vec<Cid>, CommonError> {
        let mut removed = Vec::new();
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
        let rows = client
            .query(
                "SELECT cid FROM blocks WHERE pinned = false AND ttl IS NOT NULL AND ttl <= $1",
                &[&(now as i64)],
            )
            .await
            .map_err(|e| CommonError::DatabaseError(format!("GC query failed: {e}")))?;
        for row in rows {
            let cid_str: String = row.get(0);
            let cid = icn_common::parse_cid_from_string(&cid_str)?;
            client
                .execute("DELETE FROM blocks WHERE cid = $1", &[&cid_str])
                .await
                .map_err(|e| {
                    CommonError::DatabaseError(format!("Failed to delete block {}: {e}", cid))
                })?;
            self.meta.remove(&cid);
            removed.push(cid);
        }
        Ok(removed)
    }

    async fn set_ttl(&mut self, cid: &Cid, ttl: Option<u64>) -> Result<(), CommonError> {
        match self.meta.get_mut(cid) {
            Some(m) => {
                m.ttl = ttl;
                let client = self
                    .pool
                    .get()
                    .await
                    .map_err(|e| CommonError::DatabaseError(format!("Pool error: {e}")))?;
                client
                    .execute(
                        "UPDATE blocks SET ttl = $1 WHERE cid = $2",
                        &[&ttl.map(|t| t as i64), &cid.to_string()],
                    )
                    .await
                    .map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to update TTL for block {}: {e}",
                            cid
                        ))
                    })?;
                Ok(())
            }
            None => Err(CommonError::ResourceNotFound(format!(
                "Block {} not found",
                cid
            ))),
        }
    }

    async fn get_metadata(&self, cid: &Cid) -> Result<Option<BlockMetadata>, CommonError> {
        Ok(self.meta.get(cid).cloned())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
