use crate::{Cid, CommonError, DagBlock, StorageService};
use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Debug)]
pub struct SqliteDagStore {
    conn: Connection,
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
        Ok(Self { conn })
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

    fn iter(&self) -> Result<Vec<DagBlock>, CommonError> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM blocks")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to prepare query: {}", e)))?;
        let rows = stmt
            .query_map([], |row| {
                let data: Vec<u8> = row.get(0)?;
                Ok(data)
            })
            .map_err(|e| CommonError::DatabaseError(format!("Query failed: {}", e)))?;
        let mut blocks = Vec::new();
        for r in rows {
            let data = r.map_err(|e| CommonError::DatabaseError(format!("Row error: {}", e)))?;
            let block: DagBlock = serde_json::from_slice(&data).map_err(|e| {
                CommonError::DeserializationError(format!("Failed to deserialize block: {}", e))
            })?;
            blocks.push(block);
        }
        Ok(blocks)
    }
}
