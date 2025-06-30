use crate::EconError;
use icn_common::{CommonError, Did};
use rocksdb::DB;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub struct RocksdbManaLedger {
    db: DB,
}

impl RocksdbManaLedger {
    /// Initialise a RocksDB backed mana ledger at `path`.
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

    pub fn credit_all(&self, amount: u64) -> Result<(), EconError> {
        use rocksdb::IteratorMode;
        use std::str::FromStr;
        for item in self.db.iterator(IteratorMode::Start) {
            let (key, val) = item
                .map_err(|e| EconError::AdapterError(format!("Failed to iterate ledger: {e}")))?;
            let did_str = std::str::from_utf8(&key)
                .map_err(|e| EconError::AdapterError(format!("Invalid key: {e}")))?;
            let did =
                Did::from_str(did_str).map_err(|e| EconError::AdapterError(format!("{e}")))?;
            let mut bal: u64 = bincode::deserialize::<u64>(&val)
                .map_err(|e| EconError::AdapterError(format!("Failed to decode balance: {e}")))?;
            bal += amount;
            self.write_balance(&did, bal)?;
        }
        Ok(())
    }

    /// Return all account DIDs currently stored in the ledger.
    pub fn all_accounts(&self) -> Vec<Did> {
        use rocksdb::IteratorMode;
        use std::str::FromStr;
        let mut accounts = Vec::new();
        for item in self.db.iterator(IteratorMode::Start) {
            if let Ok((key, _)) = item {
                if let Ok(did_str) = std::str::from_utf8(&key) {
                    if let Ok(did) = Did::from_str(did_str) {
                        accounts.push(did);
                    }
                }
            }
        }
        accounts
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

    fn credit_all(&self, amount: u64) -> Result<(), EconError> {
        RocksdbManaLedger::credit_all(self, amount)
    }

    fn all_accounts(&self) -> Vec<Did> {
        RocksdbManaLedger::all_accounts(self)
    }
}
