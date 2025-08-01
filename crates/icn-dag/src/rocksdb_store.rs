use crate::{BlockMetadata, Cid, CommonError, DagBlock, StorageService};
use rocksdb::{Options, DB};
use std::path::PathBuf;

#[derive(Debug)]
pub struct RocksDagStore {
    db: DB,
    meta: std::collections::HashMap<Cid, BlockMetadata>,
}

impl RocksDagStore {
    /// Create a new RocksDB backed DAG store.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open RocksDB: {}", e)))?;
        Ok(Self {
            db,
            meta: std::collections::HashMap::new(),
        })
    }
}

impl StorageService<DagBlock> for RocksDagStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        icn_common::verify_block_integrity(block)?;
        let encoded = bincode::serialize(block).map_err(|e| {
            CommonError::SerializationError(format!(
                "Failed to serialize block {}: {}",
                block.cid, e
            ))
        })?;
        self.db.put(block.cid.to_string(), encoded).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to store block {}: {}", block.cid, e))
        })?;
        self.meta
            .insert(block.cid.clone(), BlockMetadata::default());
        Ok(())
    }

    fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        match self.db.get(cid.to_string()).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to get block {}: {}", cid, e))
        })? {
            Some(bytes) => {
                let block: DagBlock = bincode::deserialize(&bytes).map_err(|e| {
                    CommonError::DeserializationError(format!(
                        "Failed to deserialize block {}: {}",
                        cid, e
                    ))
                })?;
                if &block.cid != cid {
                    return Err(CommonError::InvalidInputError(format!(
                        "CID mismatch for block read from rocksdb: expected {}, found {}",
                        cid, block.cid
                    )));
                }
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        self.db.delete(cid.to_string()).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to delete block {}: {}", cid, e))
        })?;
        self.meta.remove(cid);
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        let result = self.db.get_pinned(cid.to_string()).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to check block {}: {}", cid, e))
        })?;
        Ok(result.is_some())
    }

    fn list_blocks(&self) -> Result<Vec<DagBlock>, CommonError> {
        use rocksdb::IteratorMode;
        let mut blocks = Vec::new();
        for item in self.db.iterator(IteratorMode::Start) {
            let (_key, val) =
                item.map_err(|e| CommonError::DatabaseError(format!("Iteration error: {}", e)))?;
            let block: DagBlock = bincode::deserialize(&val).map_err(|e| {
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
        use rocksdb::WriteBatch;
        let mut removed = Vec::new();
        let to_remove: Vec<Cid> = self
            .meta
            .iter()
            .filter(|(_, m)| !m.pinned && m.ttl.map(|t| t <= now).unwrap_or(false))
            .map(|(c, _)| c.clone())
            .collect();
        let mut batch = WriteBatch::default();
        for cid in &to_remove {
            batch.delete(cid.to_string());
        }
        if !to_remove.is_empty() {
            self.db
                .write(batch)
                .map_err(|e| CommonError::DatabaseError(format!("Batch delete failed: {}", e)))?;
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
