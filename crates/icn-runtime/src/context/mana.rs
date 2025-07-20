//! Mana management types and implementations for the ICN runtime.

use super::errors::HostAbiError;
use icn_common::{CommonError, Did};
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "cli")]
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Supported mana ledger persistence backends.
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LedgerBackend {
    File,
    #[cfg(feature = "persist-sqlite")]
    Sqlite,
    #[cfg(feature = "persist-sled")]
    Sled,
    #[cfg(feature = "persist-rocksdb")]
    Rocksdb,
}

impl FromStr for LedgerBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "file" => Ok(LedgerBackend::File),
            #[cfg(feature = "persist-sqlite")]
            "sqlite" => Ok(LedgerBackend::Sqlite),
            #[cfg(feature = "persist-sled")]
            "sled" => Ok(LedgerBackend::Sled),
            #[cfg(feature = "persist-rocksdb")]
            "rocksdb" => Ok(LedgerBackend::Rocksdb),
            _ => Err(format!("Invalid ledger backend: {}", s)),
        }
    }
}

/// Placeholder for icn_economics::ManaRepository
pub trait ManaRepository: Send + Sync + std::fmt::Debug {
    // Define methods as needed, e.g.:
    // async fn get_balance(&self, account: &Did) -> Result<u64, EconError>;
    // async fn spend_mana(&self, account: &Did, amount: u64) -> Result<(), EconError>;
    // async fn credit_mana(&self, account: &Did, amount: u64) -> Result<(), EconError>;
}

/// Simple wrapper around the selected `ManaLedger` implementation for use inside the runtime.
#[derive(Clone)]
pub struct SimpleManaLedger {
    ledger: Arc<dyn icn_economics::ManaLedger>,
}

impl std::fmt::Debug for SimpleManaLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleManaLedger")
    }
}

impl SimpleManaLedger {
    /// Create a new ledger at the given path. Panics if the ledger cannot be
    /// initialized.
    pub fn new(path: PathBuf) -> Self {
        #[cfg(feature = "persist-sled")]
        let ledger = Arc::new(
            icn_economics::SledManaLedger::new(path)
                .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
        ) as Arc<dyn icn_economics::ManaLedger>;
        #[cfg(all(not(feature = "persist-sled"), feature = "persist-sqlite"))]
        let ledger = Arc::new(
            icn_economics::SqliteManaLedger::new(path)
                .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
        ) as Arc<dyn icn_economics::ManaLedger>;
        #[cfg(all(
            not(feature = "persist-sled"),
            not(feature = "persist-sqlite"),
            feature = "persist-rocksdb"
        ))]
        let ledger = Arc::new(
            icn_economics::RocksdbManaLedger::new(path)
                .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
        ) as Arc<dyn icn_economics::ManaLedger>;
        #[cfg(not(any(
            feature = "persist-sled",
            feature = "persist-sqlite",
            feature = "persist-rocksdb"
        )))]
        let ledger = Arc::new(
            icn_economics::FileManaLedger::new(path)
                .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
        ) as Arc<dyn icn_economics::ManaLedger>;
        Self { ledger }
    }

    /// Create a ledger using a specific backend.
    pub fn new_with_backend(path: PathBuf, backend: LedgerBackend) -> Self {
        let ledger: Arc<dyn icn_economics::ManaLedger> = match backend {
            LedgerBackend::File => Arc::new(
                icn_economics::FileManaLedger::new(path)
                    .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
            ) as Arc<_>,
            #[cfg(feature = "persist-sqlite")]
            LedgerBackend::Sqlite => Arc::new(
                icn_economics::SqliteManaLedger::new(path)
                    .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
            ) as Arc<_>,
            #[cfg(feature = "persist-sled")]
            LedgerBackend::Sled => Arc::new(
                icn_economics::SledManaLedger::new(path)
                    .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
            ) as Arc<_>,
            #[cfg(feature = "persist-rocksdb")]
            LedgerBackend::Rocksdb => Arc::new(
                icn_economics::RocksdbManaLedger::new(path)
                    .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
            ) as Arc<_>,
            #[allow(unreachable_patterns)]
            _ => Arc::new(
                icn_economics::FileManaLedger::new(path)
                    .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
            ) as Arc<_>,
        };
        Self { ledger }
    }

    /// Get the current balance for an account.
    pub fn get_balance(&self, account: &Did) -> u64 {
        self.ledger.get_balance(account)
    }

    /// Set the balance for an account.
    pub fn set_balance(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        self.ledger.set_balance(account, amount)?;
        Ok(())
    }

    /// Spend mana from an account.
    pub fn spend(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        self.ledger.spend(account, amount).map_err(|err| match err {
            CommonError::PolicyDenied(msg) if msg.contains("Insufficient mana") => {
                HostAbiError::InsufficientMana
            }
            other => HostAbiError::from(other),
        })
    }

    /// Credit mana to an account.
    pub fn credit(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        self.ledger.credit(account, amount)?;
        Ok(())
    }

    /// Credit mana to all accounts.
    pub fn credit_all(&self, amount: u64) -> Result<(), HostAbiError> {
        self.ledger.credit_all(amount)?;
        Ok(())
    }
}

impl icn_economics::ManaLedger for SimpleManaLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        self.ledger.get_balance(did)
    }

    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.ledger.set_balance(did, amount)
    }

    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.ledger.spend(did, amount)
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.ledger.credit(did, amount)
    }

    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        self.ledger.credit_all(amount)
    }

    fn all_accounts(&self) -> Vec<Did> {
        self.ledger.all_accounts()
    }
}
