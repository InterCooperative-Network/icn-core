use crate::EconError;
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
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let balances = if path.exists() {
            let mut file = File::open(&path).map_err(|e| {
                CommonError::IoError(format!("Failed to open mana ledger file {path:?}: {e}"))
            })?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(|e| {
                CommonError::IoError(format!("Failed to read mana ledger file {path:?}: {e}"))
            })?;
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

    pub fn get_balance(&self, account: &Did) -> u64 {
        let balances = self.balances.lock().unwrap();
        *balances.get(account).unwrap_or(&0)
    }

    pub fn set_balance(&self, account: &Did, amount: u64) -> Result<(), CommonError> {
        let mut balances = self.balances.lock().unwrap();
        balances.insert(account.clone(), amount);
        drop(balances);
        self.persist()
    }

    pub fn spend(&self, account: &Did, amount: u64) -> Result<(), EconError> {
        let mut balances = self.balances.lock().unwrap();
        let balance = balances
            .get_mut(account)
            .ok_or_else(|| EconError::AdapterError("Account not found".into()))?;
        if *balance < amount {
            return Err(EconError::InsufficientBalance(format!(
                "Insufficient mana for DID {account}"
            )));
        }
        *balance -= amount;
        drop(balances);
        self.persist()
            .map_err(|e| EconError::AdapterError(format!("{e}")))
    }

    pub fn credit(&self, account: &Did, amount: u64) -> Result<(), EconError> {
        let mut balances = self.balances.lock().unwrap();
        let entry = balances.entry(account.clone()).or_insert(0);
        *entry += amount;
        drop(balances);
        self.persist()
            .map_err(|e| EconError::AdapterError(format!("{e}")))
    }
}
