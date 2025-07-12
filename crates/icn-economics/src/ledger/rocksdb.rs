use icn_common::{CommonError, Did};
use rocksdb::DB;
use super::{ResourceLedger, TokenClass, TokenClassId};
use std::path::PathBuf;

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

    pub fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        use rocksdb::IteratorMode;
        use std::str::FromStr;
        for item in self.db.iterator(IteratorMode::Start) {
            let (key, val) = item.map_err(|e| {
                CommonError::DatabaseError(format!("Failed to iterate ledger: {e}"))
            })?;
            let did_str = std::str::from_utf8(&key)
                .map_err(|e| CommonError::DatabaseError(format!("Invalid key: {e}")))?;
            let did = Did::from_str(did_str)
                .map_err(|e| CommonError::InvalidInputError(format!("{e}")))?;
            let bal: u64 = bincode::deserialize::<u64>(&val).map_err(|e| {
                CommonError::DatabaseError(format!("Failed to decode balance: {e}"))
            })?;
            let new_bal = bal.saturating_add(amount);
            self.write_balance(&did, new_bal)?;
        }
        Ok(())
    }

    /// Return all account DIDs currently stored in the ledger.
    pub fn all_accounts(&self) -> Vec<Did> {
        use rocksdb::IteratorMode;
        use std::str::FromStr;
        let mut accounts = Vec::new();
        for (key, _) in self.db.iterator(IteratorMode::Start).flatten() {
            if let Ok(did_str) = std::str::from_utf8(&key) {
                if let Ok(did) = Did::from_str(did_str) {
                    accounts.push(did);
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

    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let current = self.read_balance(did)?;
        if current < amount {
            return Err(CommonError::PolicyDenied(format!(
                "Insufficient mana for DID {did}"
            )));
        }
        self.write_balance(did, current - amount)?;
        Ok(())
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let current = self.read_balance(did)?;
        self.write_balance(did, current + amount)?;
        Ok(())
    }

    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        RocksdbManaLedger::credit_all(self, amount)
    }

    fn all_accounts(&self) -> Vec<Did> {
        RocksdbManaLedger::all_accounts(self)
    }
}

// --- RocksDB based Resource Ledger --------------------------------------------

#[derive(Debug)]
pub struct RocksdbResourceLedger {
    db: DB,
}

impl RocksdbResourceLedger {
    /// Initialise a RocksDB backed resource ledger at `path`.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = DB::open_default(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open rocksdb: {e}")))?;
        Ok(Self { db })
    }

    fn class_key(id: &TokenClassId) -> String {
        format!("class:{id}")
    }

    fn balance_key(id: &TokenClassId, did: &Did) -> String {
        format!("bal:{id}:{}", did)
    }

    fn write_class(&self, id: &TokenClassId, class: &TokenClass) -> Result<(), CommonError> {
        let data = bincode::serialize(class).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize class: {e}"))
        })?;
        self.db
            .put(Self::class_key(id), data)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store class: {e}")))?;
        self.db.flush().map_err(|e| CommonError::DatabaseError(format!("Failed to flush ledger: {e}")))?;
        Ok(())
    }

    fn read_class(&self, id: &TokenClassId) -> Result<Option<TokenClass>, CommonError> {
        if let Some(val) = self
            .db
            .get(Self::class_key(id))
            .map_err(|e| CommonError::DatabaseError(format!("Failed to read class: {e}")))?
        {
            let class = bincode::deserialize::<TokenClass>(&val).map_err(|e| {
                CommonError::DeserializationError(format!("Failed to decode class: {e}"))
            })?;
            Ok(Some(class))
        } else {
            Ok(None)
        }
    }

    fn write_balance(&self, class: &TokenClassId, did: &Did, amount: u64) -> Result<(), CommonError> {
        let encoded = bincode::serialize(&amount).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize balance: {e}"))
        })?;
        self.db
            .put(Self::balance_key(class, did), encoded)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store balance: {e}")))?;
        self.db.flush().map_err(|e| CommonError::DatabaseError(format!("Failed to flush ledger: {e}")))?;
        Ok(())
    }

    fn read_balance(&self, class: &TokenClassId, did: &Did) -> Result<u64, CommonError> {
        if let Some(val) = self
            .db
            .get(Self::balance_key(class, did))
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

impl ResourceLedger for RocksdbResourceLedger {
    fn create_class(&self, id: &TokenClassId, class: TokenClass) -> Result<(), CommonError> {
        self.write_class(id, &class)
    }

    fn get_class(&self, id: &TokenClassId) -> Option<TokenClass> {
        self.read_class(id).ok().flatten()
    }

    fn mint(&self, class: &TokenClassId, owner: &Did, amount: u64) -> Result<(), CommonError> {
        let current = self.read_balance(class, owner)?;
        self.write_balance(class, owner, current + amount)
    }

    fn burn(&self, class: &TokenClassId, owner: &Did, amount: u64) -> Result<(), CommonError> {
        let current = self.read_balance(class, owner)?;
        if current < amount {
            return Err(CommonError::PolicyDenied("Insufficient balance".into()));
        }
        self.write_balance(class, owner, current - amount)
    }

    fn transfer(&self, class: &TokenClassId, from: &Did, to: &Did, amount: u64) -> Result<(), CommonError> {
        self.burn(class, from, amount)?;
        self.mint(class, to, amount)
    }

    fn get_balance(&self, class: &TokenClassId, owner: &Did) -> u64 {
        self.read_balance(class, owner).unwrap_or(0)
    }
}
