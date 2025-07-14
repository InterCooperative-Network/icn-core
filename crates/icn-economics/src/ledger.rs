use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{rename, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

/// Identifier for a particular token class stored in a [`ResourceLedger`].
pub type TokenClassId = String;

/// Basic metadata for a token class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClass {
    /// Human readable name for the token.
    pub name: String,
}

/// Trait defining generic token accounting behaviour.
pub trait ResourceLedger: Send + Sync {
    /// Create a new token class with the given id and metadata.
    fn create_class(&self, class_id: &TokenClassId, class: TokenClass) -> Result<(), CommonError>;
    /// Fetch metadata for a token class if it exists.
    fn get_class(&self, class_id: &TokenClassId) -> Option<TokenClass>;
    /// Increase the balance of `owner` in the given class by `amount`.
    fn mint(&self, class_id: &TokenClassId, owner: &Did, amount: u64) -> Result<(), CommonError>;
    /// Decrease the balance of `owner` in the given class by `amount`.
    fn burn(&self, class_id: &TokenClassId, owner: &Did, amount: u64) -> Result<(), CommonError>;
    /// Transfer tokens between accounts within the same class.
    fn transfer(
        &self,
        class_id: &TokenClassId,
        from: &Did,
        to: &Did,
        amount: u64,
    ) -> Result<(), CommonError>;
    /// Retrieve the balance for `owner` in the specified class.
    fn get_balance(&self, class_id: &TokenClassId, owner: &Did) -> u64;
}

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

    fn persist_locked(&self, balances: &HashMap<Did, u64>) -> Result<(), CommonError> {
        let ledger = LedgerFileFormat {
            balances: balances
                .iter()
                .map(|(did, amount)| (did.to_string(), *amount))
                .collect(),
        };
        let serialized = serde_json::to_string(&ledger).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize ledger: {e}"))
        })?;

        let tmp_path = self.path.with_extension("tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .map_err(|e| {
                CommonError::IoError(format!("Failed to open ledger file {tmp_path:?}: {e}"))
            })?;
        file.write_all(serialized.as_bytes()).map_err(|e| {
            CommonError::IoError(format!("Failed to write ledger file {tmp_path:?}: {e}"))
        })?;
        file.sync_all().map_err(|e| {
            CommonError::IoError(format!("Failed to sync ledger file {tmp_path:?}: {e}"))
        })?;
        drop(file);
        rename(&tmp_path, &self.path).map_err(|e| {
            CommonError::IoError(format!(
                "Failed to rename ledger file {:?} -> {:?}: {e}",
                tmp_path, self.path
            ))
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
        let result = self.persist_locked(&balances);
        drop(balances);
        result
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
        let result = self
            .persist_locked(&balances)
            .map_err(|e| CommonError::DatabaseError(format!("{e}")));
        drop(balances);
        result
    }

    /// Credit `amount` of mana to the account.
    pub fn credit(&self, account: &Did, amount: u64) -> Result<(), CommonError> {
        let mut balances = self.balances.lock().unwrap();
        let entry = balances.entry(account.clone()).or_insert(0);
        *entry += amount;
        let result = self
            .persist_locked(&balances)
            .map_err(|e| CommonError::DatabaseError(format!("{e}")));
        drop(balances);
        result
    }

    /// Add `amount` of mana to every stored account.
    pub fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        let mut balances = self.balances.lock().unwrap();
        for val in balances.values_mut() {
            *val += amount;
        }
        let result = self
            .persist_locked(&balances)
            .map_err(|e| CommonError::DatabaseError(format!("{e}")));
        drop(balances);
        result
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

// --- File based Resource Ledger -------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
struct ResourceLedgerFileFormat {
    classes: HashMap<TokenClassId, TokenClass>,
    balances: HashMap<TokenClassId, HashMap<String, u64>>, // did string
}

#[derive(Debug)]
pub struct FileResourceLedger {
    path: PathBuf,
    data: Mutex<ResourceLedgerFileFormat>,
}

impl FileResourceLedger {
    /// Create or load a ledger persisted on disk at `path`.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let data = if path.exists() {
            let mut file = File::open(&path).map_err(|e| {
                CommonError::IoError(format!("Failed to open resource ledger file {path:?}: {e}"))
            })?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(|e| {
                CommonError::IoError(format!("Failed to read resource ledger file {path:?}: {e}"))
            })?;
            if contents.trim().is_empty() {
                ResourceLedgerFileFormat::default()
            } else {
                serde_json::from_str(&contents).map_err(|e| {
                    CommonError::DeserializationError(format!(
                        "Failed to parse resource ledger {path:?}: {e}"
                    ))
                })?
            }
        } else {
            ResourceLedgerFileFormat::default()
        };
        Ok(Self {
            path,
            data: Mutex::new(data),
        })
    }

    fn persist_locked(&self, data: &ResourceLedgerFileFormat) -> Result<(), CommonError> {
        let serialized = serde_json::to_string(data).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize ledger: {e}"))
        })?;
        let tmp_path = self.path.with_extension("tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .map_err(|e| CommonError::IoError(format!("Failed to open ledger file {tmp_path:?}: {e}")))?;
        file.write_all(serialized.as_bytes()).map_err(|e| {
            CommonError::IoError(format!("Failed to write ledger file {tmp_path:?}: {e}"))
        })?;
        file.sync_all().map_err(|e| {
            CommonError::IoError(format!("Failed to sync ledger file {tmp_path:?}: {e}"))
        })?;
        drop(file);
        rename(&tmp_path, &self.path).map_err(|e| {
            CommonError::IoError(format!(
                "Failed to rename ledger file {:?} -> {:?}: {e}",
                tmp_path, self.path
            ))
        })?;
        Ok(())
    }
}

impl ResourceLedger for FileResourceLedger {
    fn create_class(&self, class_id: &TokenClassId, class: TokenClass) -> Result<(), CommonError> {
        let mut data = self.data.lock().unwrap();
        data.classes.insert(class_id.clone(), class);
        self.persist_locked(&data)
    }

    fn get_class(&self, class_id: &TokenClassId) -> Option<TokenClass> {
        let data = self.data.lock().unwrap();
        data.classes.get(class_id).cloned()
    }

    fn mint(&self, class_id: &TokenClassId, owner: &Did, amount: u64) -> Result<(), CommonError> {
        let mut data = self.data.lock().unwrap();
        let entry = data
            .balances
            .entry(class_id.clone())
            .or_default()
            .entry(owner.to_string())
            .or_insert(0);
        *entry += amount;
        self.persist_locked(&data)
    }

    fn burn(&self, class_id: &TokenClassId, owner: &Did, amount: u64) -> Result<(), CommonError> {
        let mut data = self.data.lock().unwrap();
        let entry = data
            .balances
            .entry(class_id.clone())
            .or_default()
            .entry(owner.to_string())
            .or_insert(0);
        if *entry < amount {
            return Err(CommonError::PolicyDenied("Insufficient balance".into()));
        }
        *entry -= amount;
        self.persist_locked(&data)
    }

    fn transfer(&self, class_id: &TokenClassId, from: &Did, to: &Did, amount: u64) -> Result<(), CommonError> {
        self.burn(class_id, from, amount)?;
        self.mint(class_id, to, amount)
    }

    fn get_balance(&self, class_id: &TokenClassId, owner: &Did) -> u64 {
        let data = self.data.lock().unwrap();
        data.balances
            .get(class_id)
            .and_then(|m| m.get(&owner.to_string()).cloned())
            .unwrap_or(0)
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
            let (key, val) = result.map_err(|e| {
                CommonError::DatabaseError(format!("Failed to iterate ledger: {e}"))
            })?;
            let did_str = std::str::from_utf8(&key)
                .map_err(|e| CommonError::DatabaseError(format!("Invalid key: {e}")))?;
            let did = Did::from_str(did_str)
                .map_err(|e| CommonError::InvalidInputError(format!("{e}")))?;
            let bal: u64 = bincode::deserialize::<u64>(val.as_ref()).map_err(|e| {
                CommonError::DatabaseError(format!("Failed to decode balance: {e}"))
            })?;
            let new_bal = bal.saturating_add(amount);
            self.write_balance(&did, new_bal)
                .map_err(|e| CommonError::DatabaseError(format!("{e}")))?;
        }
        Ok(())
    }

    /// Retrieve a list of all account DIDs stored in the ledger.
    pub fn all_accounts(&self) -> Vec<Did> {
        use std::str::FromStr;
        let mut accounts = Vec::new();
        for (key, _) in self.tree.iter().flatten() {
            if let Ok(did_str) = std::str::from_utf8(&key) {
                if let Ok(did) = Did::from_str(did_str) {
                    accounts.push(did);
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

// --- Sled based Resource Ledger ----------------------------------------------

#[cfg(feature = "persist-sled")]
#[derive(Debug)]
pub struct SledResourceLedger {
    classes: sled::Tree,
    balances: sled::Tree,
}

#[cfg(feature = "persist-sled")]
impl SledResourceLedger {
    /// Open or create a sled database at `path` for token accounting.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = sled::open(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sled DB: {e}")))?;
        let classes = db
            .open_tree("token_classes")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open classes tree: {e}")))?;
        let balances = db
            .open_tree("token_balances")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open balances tree: {e}")))?;
        Ok(Self { classes, balances })
    }

    fn class_key(id: &TokenClassId) -> Vec<u8> {
        id.as_bytes().to_vec()
    }

    fn balance_key(class: &TokenClassId, did: &Did) -> Vec<u8> {
        format!("{class}:{}", did).into_bytes()
    }

    fn write_class(&self, id: &TokenClassId, class: &TokenClass) -> Result<(), CommonError> {
        let data = bincode::serialize(class).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize class: {e}"))
        })?;
        self.classes
            .insert(Self::class_key(id), data)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store class: {e}")))?;
        self.classes.flush().map_err(|e| CommonError::DatabaseError(format!("Failed to flush ledger: {e}")))?;
        Ok(())
    }

    fn read_class(&self, id: &TokenClassId) -> Result<Option<TokenClass>, CommonError> {
        if let Some(val) = self
            .classes
            .get(Self::class_key(id))
            .map_err(|e| CommonError::DatabaseError(format!("Failed to read class: {e}")))?
        {
            let class = bincode::deserialize(val.as_ref()).map_err(|e| {
                CommonError::DeserializationError(format!("Failed to decode class: {e}"))
            })?;
            Ok(Some(class))
        } else {
            Ok(None)
        }
    }

    fn write_balance(&self, class: &TokenClassId, did: &Did, amount: u64) -> Result<(), CommonError> {
        let data = bincode::serialize(&amount).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize balance: {e}"))
        })?;
        self.balances
            .insert(Self::balance_key(class, did), data)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store balance: {e}")))?;
        self.balances.flush().map_err(|e| CommonError::DatabaseError(format!("Failed to flush ledger: {e}")))?;
        Ok(())
    }

    fn read_balance(&self, class: &TokenClassId, did: &Did) -> Result<u64, CommonError> {
        if let Some(val) = self
            .balances
            .get(Self::balance_key(class, did))
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
}

#[cfg(feature = "persist-sled")]
impl ResourceLedger for SledResourceLedger {
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

#[cfg(feature = "persist-sqlite")]
pub mod sqlite;
#[cfg(feature = "persist-sqlite")]
pub use sqlite::{SqliteManaLedger, SqliteResourceLedger};

#[cfg(feature = "persist-rocksdb")]
pub mod rocksdb;
#[cfg(feature = "persist-rocksdb")]
pub use rocksdb::{RocksdbManaLedger, RocksdbResourceLedger};

