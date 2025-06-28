#![allow(clippy::uninlined_format_args)]

use crate::{Cid, CommonError, DagBlock, StorageService};
use std::path::PathBuf;

#[cfg(feature = "persist-sled")]
use bincode;
#[cfg(feature = "persist-sled")]
use sled;

#[cfg(feature = "persist-sled")]
#[derive(Debug)]
pub struct SledDagStore {
    db: sled::Db,
    tree_name: String,
}

#[cfg(feature = "persist-sled")]
impl SledDagStore {
    /// Create a new sled backed DAG store at the given path.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = sled::open(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sled DB: {}", e)))?;
        Ok(Self {
            db,
            tree_name: "dag_blocks_v1".into(),
        })
    }

    fn tree(&self) -> Result<sled::Tree, CommonError> {
        self.db
            .open_tree(&self.tree_name)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open tree: {}", e)))
    }
}

#[cfg(feature = "persist-sled")]
impl StorageService<DagBlock> for SledDagStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        icn_common::verify_block_integrity(block)?;
        let tree = self.tree()?;
        let encoded = bincode::serialize(block).map_err(|e| {
            CommonError::SerializationError(format!(
                "Failed to serialize block {}: {}",
                block.cid, e
            ))
        })?;
        tree.insert(block.cid.to_string(), encoded).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to insert block {}: {}", block.cid, e))
        })?;
        Ok(())
    }

    fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        let tree = self.tree()?;
        match tree.get(cid.to_string()).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to get block {}: {}", cid, e))
        })? {
            Some(ivec) => {
                let block: DagBlock = bincode::deserialize(&ivec).map_err(|e| {
                    CommonError::DeserializationError(format!(
                        "Failed to deserialize block {}: {}",
                        cid, e
                    ))
                })?;
                if &block.cid != cid {
                    return Err(CommonError::InvalidInputError(format!(
                        "CID mismatch for block read from sled: expected {}, found {}",
                        cid, block.cid
                    )));
                }
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        let tree = self.tree()?;
        tree.remove(cid.to_string()).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to delete block {}: {}", cid, e))
        })?;
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        let tree = self.tree()?;
        let exists = tree.contains_key(cid.to_string()).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to check block {}: {}", cid, e))
        })?;
        Ok(exists)
    }
}
