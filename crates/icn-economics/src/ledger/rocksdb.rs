use icn_common::{CommonError, Did};
use rocksdb::{WriteBatch, DB};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug)]
pub struct RocksdbManaLedger {
    db: DB,
    batch: Mutex<WriteBatch>,
    flush_every: Option<usize>,
}

impl Drop for RocksdbManaLedger {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

impl RocksdbManaLedger {
    /// Initialise a RocksDB backed mana ledger at `path`.
    ///
    /// `flush_every` controls how many write operations are batched before
    /// automatically flushing to disk. If `None`, writes must be flushed
    /// manually using [`Self::flush`].
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        // Batch up to 50 operations by default.
        Self::with_flush_interval(path, Some(50))
    }

    /// Create a ledger with an explicit flush interval.
    pub fn with_flush_interval(
        path: PathBuf,
        flush_every: Option<usize>,
    ) -> Result<Self, CommonError> {
        let db = DB::open_default(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open rocksdb: {e}")))?;
        Ok(Self {
            db,
            batch: Mutex::new(WriteBatch::default()),
            flush_every,
        })
    }

    /// Flush pending batched writes to disk.
    pub fn flush(&self) -> Result<(), CommonError> {
        let mut batch = self.batch.lock().unwrap();
        if batch.len() > 0 {
            self.db
                .write(batch.as_ref())
                .map_err(|e| CommonError::DatabaseError(format!("Failed to write batch: {e}")))?;
            *batch = WriteBatch::default();
        }
        self.db
            .flush()
            .map_err(|e| CommonError::DatabaseError(format!("Failed to flush ledger: {e}")))?;
        Ok(())
    }

    fn write_balance(&self, account: &Did, amount: u64) -> Result<(), CommonError> {
        let encoded = bincode::serialize(&amount).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize balance: {e}"))
        })?;
        let mut batch = self.batch.lock().unwrap();
        batch.put(account.to_string(), encoded);
        if let Some(n) = self.flush_every {
            if batch.len() >= n {
                drop(batch);
                self.flush()?;
            }
        }
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
