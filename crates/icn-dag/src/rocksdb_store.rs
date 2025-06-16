use crate::{Cid, CommonError, DagBlock, StorageService};
use rocksdb::{Options, DB};
use std::path::PathBuf;

#[derive(Debug)]
pub struct RocksDagStore {
    db: DB,
}

impl RocksDagStore {
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open RocksDB: {}", e)))?;
        Ok(Self { db })
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
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        let result = self.db.get_pinned(cid.to_string()).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to check block {}: {}", cid, e))
        })?;
        Ok(result.is_some())
    }

    fn len(&self) -> Result<u64, CommonError> {
        use rocksdb::IteratorMode;
        Ok(self.db.iterator(IteratorMode::Start).count() as u64)
    }
}
