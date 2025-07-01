use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct LedgerFileFormat {
    balances: HashMap<String, u64>,
}

#[derive(Debug)]
pub struct FileManaLedger {
    path: PathBuf,
    balances: Mutex<HashMap<Did, u64>>,
}

impl FileManaLedger {
    /// Create or load a ledger persisted on disk at `path`.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let balances = if path.exists() {
            let mut file = File::open(&path).map_err(|e| {
                CommonError::IoError(format!("Failed to open mana ledger file {path:?}: {e}"))
            })?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(|e| {
                CommonError::IoError(format!("Failed to read mana ledger file {path:?}: {e}"))
            })?;
            if contents.trim().is_empty() {
                HashMap::new()
            } else {
                let ledger: LedgerFileFormat = serde_json::from_str(&contents).map_err(|e| {
                    CommonError::DeserializationError(format!(
                        "Failed to parse mana ledger {path:?}: {e}"
                    ))
                })?;
                ledger
                    .balances
                    .into_iter()
                    .filter_map(|(k, v)| Did::from_str(&k).ok().map(|did| (did, v)))
                    .collect()
            }
        } else {
            HashMap::new()
        };
        Ok(Self {
            path,
            balances: Mutex::new(balances),
        })
    }

    fn persist(&self) -> Result<(), CommonError> {
        let balances = self.balances.lock().unwrap();
        let ledger = LedgerFileFormat {
            balances: balances
                .iter()
                .map(|(did, amount)| (did.to_string(), *amount))
                .collect(),
        };
        let serialized = serde_json::to_string(&ledger).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize ledger: {e}"))
        })?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| {
                CommonError::IoError(format!("Failed to open ledger file {:?}: {e}", self.path))
            })?;
        file.write_all(serialized.as_bytes()).map_err(|e| {
            CommonError::IoError(format!("Failed to write ledger file {:?}: {e}", self.path))
        })?;
        Ok(())
    }

    /// Return the current mana balance for `account`.
    pub fn get_balance(&self, account: &Did) -> u64 {
        let balances = self.balances.lock().unwrap();
        *balances.get(account).unwrap_or(&0)
    }

    /// Persist a new balance for `account`.
    pub fn set_balance(&self, account: &Did, amount: u64) -> Result<(), CommonError> {
        let mut balances = self.balances.lock().unwrap();
        balances.insert(account.clone(), amount);
        drop(balances);
        self.persist()
    }

    /// Deduct `amount` of mana from the account, erroring if the balance is insufficient.
    pub fn spend(&self, account: &Did, amount: u64) -> Result<(), CommonError> {
        let mut balances = self.balances.lock().unwrap();
        let balance = balances
            .get_mut(account)
            .ok_or_else(|| CommonError::DatabaseError("Account not found".into()))?;
        if *balance < amount {
            return Err(CommonError::PolicyDenied(format!(
                "Insufficient mana for DID {account}"
            )));
        }
        *balance -= amount;
        drop(balances);
        self.persist()
            .map_err(|e| CommonError::DatabaseError(format!("{e}")))
    }

    /// Credit `amount` of mana to the account.
    pub fn credit(&self, account: &Did, amount: u64) -> Result<(), CommonError> {
        let mut balances = self.balances.lock().unwrap();
        let entry = balances.entry(account.clone()).or_insert(0);
        *entry += amount;
        drop(balances);
        self.persist()
            .map_err(|e| CommonError::DatabaseError(format!("{e}")))
    }

    /// Add `amount` of mana to every stored account.
    pub fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        let mut balances = self.balances.lock().unwrap();
        for val in balances.values_mut() {
            *val += amount;
        }
        drop(balances);
        self.persist()
            .map_err(|e| CommonError::DatabaseError(format!("{e}")))
    }

    /// Return a list of all account DIDs stored in this ledger.
    pub fn all_accounts(&self) -> Vec<Did> {
        let balances = self.balances.lock().unwrap();
        balances.keys().cloned().collect()
    }
}

impl crate::ManaLedger for FileManaLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        FileManaLedger::get_balance(self, did)
    }

    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        FileManaLedger::set_balance(self, did, amount)
    }

    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        FileManaLedger::spend(self, did, amount)
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        FileManaLedger::credit(self, did, amount)
    }

    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        FileManaLedger::credit_all(self, amount)
    }

    fn all_accounts(&self) -> Vec<Did> {
        FileManaLedger::all_accounts(self)
    }
}

// --- Persistent Sled-based Mana Ledger ---

#[cfg(feature = "persist-sled")]
#[derive(Debug)]
pub struct SledManaLedger {
    tree: sled::Tree,
}

#[cfg(feature = "persist-sled")]
impl SledManaLedger {
    /// Open or create a sled database at `path` for mana accounting.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = sled::open(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sled DB: {e}")))?;
        let tree = db
            .open_tree("mana_balances")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open tree: {e}")))?;
        Ok(Self { tree })
    }

    fn write_balance(&self, account: &Did, amount: u64) -> Result<(), CommonError> {
        let encoded = bincode::serialize(&amount).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize balance: {e}"))
        })?;
        self.tree
            .insert(account.to_string(), encoded)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store balance: {e}")))?;
        self.tree
            .flush()
            .map_err(|e| CommonError::DatabaseError(format!("Failed to flush ledger: {e}")))?;
        Ok(())
    }

    fn read_balance(&self, account: &Did) -> Result<u64, CommonError> {
        if let Some(val) = self
            .tree
            .get(account.to_string())
            .map_err(|e| CommonError::DatabaseError(format!("Failed to read balance: {e}")))?
        {
            let amt: u64 = bincode::deserialize(val.as_ref()).map_err(|e| {
                CommonError::DeserializationError(format!("Failed to deserialize balance: {e}"))
            })?;
            Ok(amt)
        } else {
            Ok(0)
        }
    }

    pub fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        use std::str::FromStr;
        for result in self.tree.iter() {
            let (key, val) = result
                .map_err(|e| CommonError::DatabaseError(format!("Failed to iterate ledger: {e}")))?;
            let did_str = std::str::from_utf8(&key)
                .map_err(|e| CommonError::DatabaseError(format!("Invalid key: {e}")))?;
            let did =
                Did::from_str(did_str).map_err(|e| CommonError::InvalidInputError(format!("{e}")))?;
            let mut bal: u64 = bincode::deserialize::<u64>(val.as_ref())
                .map_err(|e| CommonError::DatabaseError(format!("Failed to decode balance: {e}")))?;
            bal += amount;
            self.write_balance(&did, bal)
                .map_err(|e| CommonError::DatabaseError(format!("{e}")))?;
        }
        Ok(())
    }

    /// Retrieve a list of all account DIDs stored in the ledger.
    pub fn all_accounts(&self) -> Vec<Did> {
        use std::str::FromStr;
        let mut accounts = Vec::new();
        for result in self.tree.iter() {
            if let Ok((key, _)) = result {
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

#[cfg(feature = "persist-sled")]
impl crate::ManaLedger for SledManaLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        self.read_balance(did).unwrap_or(0)
    }

    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.write_balance(did, amount)
    }

    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let current = self
            .read_balance(did)
            .map_err(|e| CommonError::DatabaseError(format!("{e}")))?;
        if current < amount {
            return Err(CommonError::PolicyDenied(format!(
                "Insufficient mana for DID {did}"
            )));
        }
        self.write_balance(did, current - amount)
            .map_err(|e| CommonError::DatabaseError(format!("{e}")))
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let current = self
            .read_balance(did)
            .map_err(|e| CommonError::DatabaseError(format!("{e}")))?;
        self.write_balance(did, current + amount)
            .map_err(|e| CommonError::DatabaseError(format!("{e}")))
    }

    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        SledManaLedger::credit_all(self, amount)
    }

    fn all_accounts(&self) -> Vec<Did> {
        SledManaLedger::all_accounts(self)
    }
}

#[cfg(feature = "persist-sqlite")]
pub mod sqlite;
#[cfg(feature = "persist-sqlite")]
pub use sqlite::SqliteManaLedger;

#[cfg(feature = "persist-rocksdb")]
pub mod rocksdb;
#[cfg(feature = "persist-rocksdb")]
pub use rocksdb::RocksdbManaLedger;
