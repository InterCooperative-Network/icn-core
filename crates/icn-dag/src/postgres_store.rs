use crate::{BlockMetadata, Cid, CommonError, DagBlock, StorageService};
use postgres::{Client, NoTls};
use std::collections::HashMap;

#[derive(Debug)]
pub struct PostgresDagStore {
    client: Client,
    meta: HashMap<Cid, BlockMetadata>,
}

impl PostgresDagStore {
    /// Connect to Postgres using the provided connection string.
    pub fn new(conn_str: &str) -> Result<Self, CommonError> {
        let mut client = Client::connect(conn_str, NoTls).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to connect to postgres: {}", e))
        })?;
        client
            .batch_execute("CREATE TABLE IF NOT EXISTS blocks (cid TEXT PRIMARY KEY, data BYTEA)")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to init table: {}", e)))?;
        Ok(Self {
            client,
            meta: HashMap::new(),
        })
    }
}

impl StorageService<DagBlock> for PostgresDagStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        icn_common::verify_block_integrity(block)?;
        let encoded = serde_json::to_vec(block).map_err(|e| {
            CommonError::SerializationError(format!(
                "Failed to serialize block {}: {}",
                block.cid, e
            ))
        })?;
        self.client
            .execute(
                "INSERT INTO blocks (cid, data) VALUES ($1, $2) ON CONFLICT (cid) DO UPDATE SET data = EXCLUDED.data",
                &[&block.cid.to_string(), &encoded],
            )
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store block {}: {}", block.cid, e)))?;
        self.meta
            .insert(block.cid.clone(), BlockMetadata::default());
        Ok(())
    }

    fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        let row_opt = self
            .client
            .query_opt(
                "SELECT data FROM blocks WHERE cid = $1",
                &[&cid.to_string()],
            )
            .map_err(|e| CommonError::DatabaseError(format!("Query failed: {}", e)))?;
        if let Some(row) = row_opt {
            let data: Vec<u8> = row.get(0);
            let block: DagBlock = serde_json::from_slice(&data).map_err(|e| {
                CommonError::DeserializationError(format!(
                    "Failed to deserialize block {}: {}",
                    cid, e
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

    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        self.client
            .execute("DELETE FROM blocks WHERE cid = $1", &[&cid.to_string()])
            .map_err(|e| {
                CommonError::DatabaseError(format!("Failed to delete block {}: {}", cid, e))
            })?;
        self.meta.remove(cid);
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        let row = self
            .client
            .query_one(
                "SELECT COUNT(1) FROM blocks WHERE cid = $1",
                &[&cid.to_string()],
            )
            .map_err(|e| {
                CommonError::DatabaseError(format!("Failed to check block {}: {}", cid, e))
            })?;
        let count: i64 = row.get(0);
        Ok(count > 0)
    }

    fn list_blocks(&self) -> Result<Vec<DagBlock>, CommonError> {
        let rows = self
            .client
            .query("SELECT data FROM blocks", &[])
            .map_err(|e| CommonError::DatabaseError(format!("Query failed: {}", e)))?;
        let mut blocks = Vec::new();
        for row in rows {
            let data: Vec<u8> = row.get(0);
            let block: DagBlock = serde_json::from_slice(&data).map_err(|e| {
                CommonError::DeserializationError(format!("Failed to deserialize block: {}", e))
            })?;
            blocks.push(block);
        }
        Ok(blocks)
    }

    fn pin_block(&mut self, cid: &Cid) -> Result<(), CommonError> {
        match self.meta.get_mut(cid) {
            Some(m) => {
                m.pinned = true;
                Ok(())
            }
            None => Err(CommonError::ResourceNotFound(format!(
                "Block {} not found",
                cid
            ))),
        }
    }

    fn unpin_block(&mut self, cid: &Cid) -> Result<(), CommonError> {
        match self.meta.get_mut(cid) {
            Some(m) => {
                m.pinned = false;
                Ok(())
            }
            None => Err(CommonError::ResourceNotFound(format!(
                "Block {} not found",
                cid
            ))),
        }
    }

    fn prune_expired(&mut self, now: u64) -> Result<Vec<Cid>, CommonError> {
        let mut removed = Vec::new();
        let to_remove: Vec<Cid> = self
            .meta
            .iter()
            .filter(|(_, m)| !m.pinned && m.ttl.map(|t| t <= now).unwrap_or(false))
            .map(|(c, _)| c.clone())
            .collect();
        for cid in &to_remove {
            self.client
                .execute("DELETE FROM blocks WHERE cid = $1", &[&cid.to_string()])
                .map_err(|e| {
                    CommonError::DatabaseError(format!("Failed to delete block {}: {}", cid, e))
                })?;
        }
        for cid in to_remove {
            self.meta.remove(&cid);
            removed.push(cid);
        }
        Ok(removed)
    }

    fn set_ttl(&mut self, cid: &Cid, ttl: Option<u64>) -> Result<(), CommonError> {
        match self.meta.get_mut(cid) {
            Some(m) => {
                m.ttl = ttl;
                Ok(())
            }
            None => Err(CommonError::ResourceNotFound(format!(
                "Block {} not found",
                cid
            ))),
        }
    }

    fn get_metadata(&self, cid: &Cid) -> Result<Option<BlockMetadata>, CommonError> {
        Ok(self.meta.get(cid).cloned())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
