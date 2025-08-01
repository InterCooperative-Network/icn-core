#![allow(clippy::uninlined_format_args)]

use crate::{BlockMetadata, Cid, CommonError, DagBlock, StorageService};
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
    meta: std::collections::HashMap<Cid, BlockMetadata>,
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
            meta: std::collections::HashMap::new(),
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
        self.meta
            .insert(block.cid.clone(), BlockMetadata::default());
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
        self.meta.remove(cid);
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        let tree = self.tree()?;
        let exists = tree.contains_key(cid.to_string()).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to check block {}: {}", cid, e))
        })?;
        Ok(exists)
    }

    fn list_blocks(&self) -> Result<Vec<DagBlock>, CommonError> {
        let tree = self.tree()?;
        let mut blocks = Vec::new();
        for item in tree.iter() {
            let (_, val) =
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
        let mut removed = Vec::new();
        let to_remove: Vec<Cid> = self
            .meta
            .iter()
            .filter(|(_, m)| !m.pinned && m.ttl.map(|t| t <= now).unwrap_or(false))
            .map(|(c, _)| c.clone())
            .collect();
        let tree = self.tree()?;
        for cid in to_remove {
            tree.remove(cid.to_string()).map_err(|e| {
                CommonError::DatabaseError(format!("Failed to delete block {}: {}", cid, e))
            })?;
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
