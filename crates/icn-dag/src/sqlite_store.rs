use crate::{BlockMetadata, Cid, CommonError, DagBlock, StorageService};
use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Debug)]
pub struct SqliteDagStore {
    conn: Connection,
    meta: std::collections::HashMap<Cid, BlockMetadata>,
}

impl SqliteDagStore {
    /// Create a new SQLite backed DAG store.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let conn = Connection::open(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {}", e)))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (cid TEXT PRIMARY KEY, data BLOB)",
            [],
        )
        .map_err(|e| CommonError::DatabaseError(format!("Failed to create table: {}", e)))?;
        Ok(Self {
            conn,
            meta: std::collections::HashMap::new(),
        })
    }
}

impl StorageService<DagBlock> for SqliteDagStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        icn_common::verify_block_integrity(block)?;
        let encoded = serde_json::to_vec(block).map_err(|e| {
            CommonError::SerializationError(format!(
                "Failed to serialize block {}: {}",
                block.cid, e
            ))
        })?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO blocks (cid, data) VALUES (?1, ?2)",
                params![block.cid.to_string(), encoded],
            )
            .map_err(|e| {
                CommonError::DatabaseError(format!("Failed to store block {}: {}", block.cid, e))
            })?;
        self.meta
            .insert(block.cid.clone(), BlockMetadata::default());
        Ok(())
    }

    fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM blocks WHERE cid = ?1")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to prepare query: {}", e)))?;
        let mut rows = stmt
            .query(params![cid.to_string()])
            .map_err(|e| CommonError::DatabaseError(format!("Query failed: {}", e)))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| CommonError::DatabaseError(format!("Row fetch failed: {}", e)))?
        {
            let data: Vec<u8> = row
                .get(0)
                .map_err(|e| CommonError::DatabaseError(format!("Failed to read blob: {}", e)))?;
            let block: DagBlock = serde_json::from_slice(&data).map_err(|e| {
                CommonError::DeserializationError(format!(
                    "Failed to deserialize block {}: {}",
                    cid, e
                ))
            })?;
            if &block.cid != cid {
                return Err(CommonError::InvalidInputError(format!(
                    "CID mismatch for block read from sqlite: expected {}, found {}",
                    cid, block.cid
                )));
            }
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        self.conn
            .execute(
                "DELETE FROM blocks WHERE cid = ?1",
                params![cid.to_string()],
            )
            .map_err(|e| {
                CommonError::DatabaseError(format!("Failed to delete block {}: {}", cid, e))
            })?;
        self.meta.remove(cid);
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        let count: u32 = self
            .conn
            .query_row(
                "SELECT COUNT(1) FROM blocks WHERE cid = ?1",
                params![cid.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| {
                CommonError::DatabaseError(format!("Failed to check block {}: {}", cid, e))
            })?;
        Ok(count > 0)
    }

    fn list_blocks(&self) -> Result<Vec<DagBlock>, CommonError> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM blocks")
            .map_err(|e| CommonError::DatabaseError(format!("Prepare failed: {}", e)))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, Vec<u8>>(0))
            .map_err(|e| CommonError::DatabaseError(format!("Query failed: {}", e)))?;
        let mut blocks = Vec::new();
        for row_result in rows {
            let data =
                row_result.map_err(|e| CommonError::DatabaseError(format!("Row error: {}", e)))?;
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
            self.conn
                .execute(
                    "DELETE FROM blocks WHERE cid = ?1",
                    params![cid.to_string()],
                )
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
