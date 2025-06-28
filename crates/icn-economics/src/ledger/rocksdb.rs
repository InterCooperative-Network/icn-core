use crate::EconError;
use icn_common::{CommonError, Did};
use rocksdb::DB;
use std::path::PathBuf;

#[derive(Debug)]
pub struct RocksdbManaLedger {
    db: DB,
}

impl RocksdbManaLedger {
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = DB::open_default(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open rocksdb: {e}")))?;
        Ok(Self { db })
    }

    fn write_balance(&self, account: &Did, amount: u64) -> Result<(), CommonError> {
        let encoded = bincode::serialize(&amount).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize balance: {e}"))
        })?;
        self.db
            .put(account.to_string(), encoded)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store balance: {e}")))?;
        self.db
            .flush()
            .map_err(|e| CommonError::DatabaseError(format!("Failed to flush ledger: {e}")))?;
        Ok(())
    }

    fn read_balance(&self, account: &Did) -> Result<u64, CommonError> {
        if let Some(val) = self
            .db
            .get(account.to_string())
            .map_err(|e| CommonError::DatabaseError(format!("Failed to read balance: {e}")))?
        {
            let amt: u64 = bincode::deserialize(&val).map_err(|e| {
                CommonError::DeserializationError(format!("Failed to deserialize balance: {e}"))
            })?;
            Ok(amt)
        } else {
            Ok(0)
        }
    }
}

impl crate::ManaLedger for RocksdbManaLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        self.read_balance(did).unwrap_or(0)
    }

    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.write_balance(did, amount)
    }

    fn spend(&self, did: &Did, amount: u64) -> Result<(), EconError> {
        let current = self.read_balance(did)?;
        if current < amount {
            return Err(EconError::InsufficientBalance(format!(
                "Insufficient mana for DID {did}"
            )));
        }
        self.write_balance(did, current - amount)?;
        Ok(())
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), EconError> {
        let current = self.read_balance(did)?;
        self.write_balance(did, current + amount)?;
        Ok(())
    }
}
