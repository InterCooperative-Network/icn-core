use icn_common::{CommonError, Did, SystemTimeProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{rename, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

/// Identifier for a particular token class stored in a [`ResourceLedger`].
pub type TokenClassId = String;

/// Record of a token transfer for audit trails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    /// Unique identifier for this transfer.
    pub transfer_id: String,
    /// Token class being transferred.
    pub class_id: TokenClassId,
    /// Account sending the tokens.
    pub from: icn_common::Did,
    /// Account receiving the tokens.
    pub to: icn_common::Did,
    /// Amount transferred.
    pub amount: u64,
    /// Unix timestamp of the transfer.
    pub timestamp: u64,
    /// Transaction hash or CID for verification.
    pub transaction_hash: String,
    /// Optional metadata about the transfer.
    pub metadata: std::collections::HashMap<String, String>,
}

/// Defines the type of token and its properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    /// Fungible tokens where each unit is interchangeable (e.g., currencies, credits)
    Fungible,
    /// Non-fungible tokens representing unique items (e.g., certificates, assets)
    NonFungible,
    /// Semi-fungible tokens with both fungible and unique properties
    SemiFungible,
    /// Time banking tokens representing labor hours
    TimeBanking,
    /// Mutual credit tokens for community exchange
    MutualCredit,
    /// Local currency tokens with geographic/community restrictions
    LocalCurrency,
    /// Bulk purchasing tokens for collective buying power
    BulkPurchasing,
}

/// Defines how tokens can be transferred between accounts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferabilityRule {
    /// Tokens can be freely transferred to any account
    FreelyTransferable,
    /// Tokens can only be transferred to authorized accounts
    RestrictedTransfer {
        /// List of authorized recipient DIDs
        authorized_recipients: Vec<icn_common::Did>,
    },
    /// Tokens cannot be transferred once issued
    NonTransferable,
    /// Tokens can only be transferred back to the issuer
    IssuerOnly,
}

/// Defines scoping rules for token operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScopingRules {
    /// Geographic scope limitations
    pub geographic_scope: Option<String>,
    /// Community/federation scope limitations
    pub community_scope: Option<String>,
    /// Time-based validity restrictions
    pub validity_period: Option<(u64, u64)>, // (start_timestamp, end_timestamp)
    /// Maximum supply limit
    pub max_supply: Option<u64>,
    /// Minimum balance required for operations
    pub min_balance: Option<u64>,
}

/// Enhanced metadata for a token class with comprehensive properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClass {
    /// Human readable name for the token.
    pub name: String,
    /// Detailed description of the token's purpose.
    pub description: String,
    /// Symbol or abbreviation for the token.
    pub symbol: String,
    /// Number of decimal places for display (0 for whole numbers only).
    pub decimals: u8,
    /// Type of token defining its core properties.
    pub token_type: TokenType,
    /// Rules governing how tokens can be transferred.
    pub transferability: TransferabilityRule,
    /// Scoping rules and limitations.
    pub scoping_rules: ScopingRules,
    /// DID of the issuer/creator of this token class.
    pub issuer: icn_common::Did,
    /// Unix timestamp when this token class was created.
    pub created_at: u64,
    /// Optional metadata for specialized token types.
    pub metadata: std::collections::HashMap<String, String>,
}

/// Trait defining generic token accounting behaviour.
pub trait ResourceLedger: Send + Sync {
    /// Create a new token class with the given id and metadata.
    fn create_class(&self, class_id: &TokenClassId, class: TokenClass) -> Result<(), CommonError>;
    /// Fetch metadata for a token class if it exists.
    fn get_class(&self, class_id: &TokenClassId) -> Option<TokenClass>;
    /// Update metadata for an existing token class (only issuer can update).
    fn update_class(&self, class_id: &TokenClassId, class: TokenClass) -> Result<(), CommonError>;
    /// List all token classes in the ledger.
    fn list_classes(&self) -> Vec<(TokenClassId, TokenClass)>;
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
    /// Check if a transfer operation is allowed under token rules.
    fn can_transfer(
        &self,
        class_id: &TokenClassId,
        from: &Did,
        to: &Did,
        amount: u64,
    ) -> Result<bool, CommonError>;
    /// Get transfer history for an account in a specific token class.
    fn get_transfer_history(&self, class_id: &TokenClassId, did: &Did) -> Vec<TransferRecord>;
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
    transfer_history: HashMap<TokenClassId, HashMap<String, Vec<TransferRecord>>>, // class_id -> did -> transfers
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

    fn burn_locked(
        &self,
        data: &mut ResourceLedgerFileFormat,
        class_id: &TokenClassId,
        owner: &Did,
        amount: u64,
    ) -> Result<(), CommonError> {
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
        Ok(())
    }

    fn mint_locked(
        &self,
        data: &mut ResourceLedgerFileFormat,
        class_id: &TokenClassId,
        owner: &Did,
        amount: u64,
    ) -> Result<(), CommonError> {
        let entry = data
            .balances
            .entry(class_id.clone())
            .or_default()
            .entry(owner.to_string())
            .or_insert(0);
        *entry += amount;
        Ok(())
    }
}

impl ResourceLedger for FileResourceLedger {
    fn create_class(&self, class_id: &TokenClassId, class: TokenClass) -> Result<(), CommonError> {
        let mut data = self.data.lock().unwrap();
        if data.classes.contains_key(class_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Token class {} already exists",
                class_id
            )));
        }
        data.classes.insert(class_id.clone(), class);
        self.persist_locked(&data)
    }

    fn get_class(&self, class_id: &TokenClassId) -> Option<TokenClass> {
        let data = self.data.lock().unwrap();
        data.classes.get(class_id).cloned()
    }

    fn update_class(&self, class_id: &TokenClassId, class: TokenClass) -> Result<(), CommonError> {
        let mut data = self.data.lock().unwrap();
        if let Some(existing_class) = data.classes.get(class_id) {
            // Only issuer can update the class
            if existing_class.issuer != class.issuer {
                return Err(CommonError::PolicyDenied(
                    "Only issuer can update token class".into(),
                ));
            }
            data.classes.insert(class_id.clone(), class);
            self.persist_locked(&data)
        } else {
            Err(CommonError::InvalidInputError(format!(
                "Token class {} not found",
                class_id
            )))
        }
    }

    fn list_classes(&self) -> Vec<(TokenClassId, TokenClass)> {
        let data = self.data.lock().unwrap();
        data.classes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
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

    fn transfer(
        &self,
        class_id: &TokenClassId,
        from: &Did,
        to: &Did,
        amount: u64,
    ) -> Result<(), CommonError> {
        // Check if transfer is allowed
        if !self.can_transfer(class_id, from, to, amount)? {
            return Err(CommonError::PolicyDenied(
                "Transfer not allowed by token rules".into(),
            ));
        }

        let mut data = self.data.lock().unwrap();

        // Perform the transfer
        self.burn_locked(&mut data, class_id, from, amount)?;
        self.mint_locked(&mut data, class_id, to, amount)?;

        // Record the transfer
        let transfer_record = TransferRecord {
            transfer_id: format!(
                "{}:{}:{}",
                class_id,
                from,
                icn_common::SystemTimeProvider.unix_seconds()
            ),
            class_id: class_id.clone(),
            from: from.clone(),
            to: to.clone(),
            amount,
            timestamp: icn_common::SystemTimeProvider.unix_seconds(),
            transaction_hash: format!("hash_{}", icn_common::SystemTimeProvider.unix_seconds()),
            metadata: HashMap::new(),
        };

        data.transfer_history
            .entry(class_id.clone())
            .or_default()
            .entry(from.to_string())
            .or_default()
            .push(transfer_record.clone());

        data.transfer_history
            .entry(class_id.clone())
            .or_default()
            .entry(to.to_string())
            .or_default()
            .push(transfer_record);

        self.persist_locked(&data)
    }

    fn get_balance(&self, class_id: &TokenClassId, owner: &Did) -> u64 {
        let data = self.data.lock().unwrap();
        data.balances
            .get(class_id)
            .and_then(|m| m.get(&owner.to_string()).cloned())
            .unwrap_or(0)
    }

    fn can_transfer(
        &self,
        class_id: &TokenClassId,
        from: &Did,
        to: &Did,
        amount: u64,
    ) -> Result<bool, CommonError> {
        let data = self.data.lock().unwrap();

        // Get token class to check transferability rules
        let token_class = data.classes.get(class_id).ok_or_else(|| {
            CommonError::InvalidInputError(format!("Token class {} not found", class_id))
        })?;

        // Check transferability rules
        match &token_class.transferability {
            TransferabilityRule::FreelyTransferable => Ok(true),
            TransferabilityRule::RestrictedTransfer {
                authorized_recipients,
            } => Ok(authorized_recipients.contains(to)),
            TransferabilityRule::NonTransferable => Ok(false),
            TransferabilityRule::IssuerOnly => Ok(to == &token_class.issuer),
        }
    }

    fn get_transfer_history(&self, class_id: &TokenClassId, did: &Did) -> Vec<TransferRecord> {
        let data = self.data.lock().unwrap();
        data.transfer_history
            .get(class_id)
            .and_then(|class_history| class_history.get(&did.to_string()))
            .cloned()
            .unwrap_or_default()
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
        let balances = db.open_tree("token_balances").map_err(|e| {
            CommonError::DatabaseError(format!("Failed to open balances tree: {e}"))
        })?;
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
        self.classes
            .flush()
            .map_err(|e| CommonError::DatabaseError(format!("Failed to flush ledger: {e}")))?;
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

    fn write_balance(
        &self,
        class: &TokenClassId,
        did: &Did,
        amount: u64,
    ) -> Result<(), CommonError> {
        let data = bincode::serialize(&amount).map_err(|e| {
            CommonError::SerializationError(format!("Failed to serialize balance: {e}"))
        })?;
        self.balances
            .insert(Self::balance_key(class, did), data)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store balance: {e}")))?;
        self.balances
            .flush()
            .map_err(|e| CommonError::DatabaseError(format!("Failed to flush ledger: {e}")))?;
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
        if self.read_class(id)?.is_some() {
            return Err(CommonError::InvalidInputError(format!(
                "Token class {} already exists",
                id
            )));
        }
        self.write_class(id, &class)
    }

    fn get_class(&self, id: &TokenClassId) -> Option<TokenClass> {
        self.read_class(id).ok().flatten()
    }

    fn update_class(&self, id: &TokenClassId, class: TokenClass) -> Result<(), CommonError> {
        if let Some(existing_class) = self.read_class(id)? {
            // Only issuer can update the class
            if existing_class.issuer != class.issuer {
                return Err(CommonError::PolicyDenied(
                    "Only issuer can update token class".into(),
                ));
            }
            self.write_class(id, &class)
        } else {
            Err(CommonError::InvalidInputError(format!(
                "Token class {} not found",
                id
            )))
        }
    }

    fn list_classes(&self) -> Vec<(TokenClassId, TokenClass)> {
        let mut classes = Vec::new();
        for result in self.classes.iter() {
            if let Ok((key, value)) = result {
                if let (Ok(class_id), Ok(class)) = (
                    std::str::from_utf8(&key),
                    bincode::deserialize::<TokenClass>(&value),
                ) {
                    classes.push((class_id.to_string(), class));
                }
            }
        }
        classes
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

    fn transfer(
        &self,
        class: &TokenClassId,
        from: &Did,
        to: &Did,
        amount: u64,
    ) -> Result<(), CommonError> {
        // Check if transfer is allowed
        if !self.can_transfer(class, from, to, amount)? {
            return Err(CommonError::PolicyDenied(
                "Transfer not allowed by token rules".into(),
            ));
        }

        self.burn(class, from, amount)?;
        self.mint(class, to, amount)?;

        // For now, Sled implementation doesn't store transfer history
        // This could be enhanced to store transfer records in a separate tree
        Ok(())
    }

    fn get_balance(&self, class: &TokenClassId, owner: &Did) -> u64 {
        self.read_balance(class, owner).unwrap_or(0)
    }

    fn can_transfer(
        &self,
        class_id: &TokenClassId,
        _from: &Did,
        to: &Did,
        _amount: u64,
    ) -> Result<bool, CommonError> {
        // Get token class to check transferability rules
        let token_class = self.get_class(class_id).ok_or_else(|| {
            CommonError::InvalidInputError(format!("Token class {} not found", class_id))
        })?;

        // Check transferability rules
        match &token_class.transferability {
            TransferabilityRule::FreelyTransferable => Ok(true),
            TransferabilityRule::RestrictedTransfer {
                authorized_recipients,
            } => Ok(authorized_recipients.contains(to)),
            TransferabilityRule::NonTransferable => Ok(false),
            TransferabilityRule::IssuerOnly => Ok(to == &token_class.issuer),
        }
    }

    fn get_transfer_history(&self, _class_id: &TokenClassId, _did: &Did) -> Vec<TransferRecord> {
        // For now, Sled implementation doesn't store transfer history
        // This could be enhanced to store transfer records in a separate tree
        Vec::new()
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

/// Helper functions for creating common token types.
impl TokenClass {
    /// Create a new fungible token class.
    pub fn new_fungible(
        name: String,
        description: String,
        symbol: String,
        decimals: u8,
        issuer: Did,
    ) -> Self {
        Self {
            name,
            description,
            symbol,
            decimals,
            token_type: TokenType::Fungible,
            transferability: TransferabilityRule::FreelyTransferable,
            scoping_rules: ScopingRules {
                geographic_scope: None,
                community_scope: None,
                validity_period: None,
                max_supply: None,
                min_balance: None,
            },
            issuer,
            created_at: icn_common::SystemTimeProvider.unix_seconds(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new time banking token class.
    pub fn new_time_banking(
        name: String,
        description: String,
        issuer: Did,
        community_scope: Option<String>,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("unit".to_string(), "hours".to_string());
        metadata.insert("rate_type".to_string(), "equal".to_string()); // All labor hours equal

        Self {
            name,
            description,
            symbol: "TIME".to_string(),
            decimals: 2, // Allow fractional hours (e.g., 1.5 hours)
            token_type: TokenType::TimeBanking,
            transferability: TransferabilityRule::FreelyTransferable,
            scoping_rules: ScopingRules {
                geographic_scope: None,
                community_scope,
                validity_period: None,
                max_supply: None,
                min_balance: Some(0), // Can't go into debt without mutual agreement
            },
            issuer,
            created_at: icn_common::SystemTimeProvider.unix_seconds(),
            metadata,
        }
    }

    /// Create a new mutual credit token class.
    pub fn new_mutual_credit(
        name: String,
        description: String,
        symbol: String,
        issuer: Did,
        community_scope: String,
        credit_limit: u64,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("credit_limit".to_string(), credit_limit.to_string());
        metadata.insert("interest_rate".to_string(), "0".to_string()); // Typically no interest

        Self {
            name,
            description,
            symbol,
            decimals: 2,
            token_type: TokenType::MutualCredit,
            transferability: TransferabilityRule::FreelyTransferable,
            scoping_rules: ScopingRules {
                geographic_scope: None,
                community_scope: Some(community_scope),
                validity_period: None,
                max_supply: Some(credit_limit * 100), // Allow reasonable expansion
                min_balance: None,                    // Can go negative (credit)
            },
            issuer,
            created_at: icn_common::SystemTimeProvider.unix_seconds(),
            metadata,
        }
    }

    /// Create a new local currency token class.
    pub fn new_local_currency(
        name: String,
        description: String,
        symbol: String,
        issuer: Did,
        geographic_scope: String,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("currency_type".to_string(), "local".to_string());
        metadata.insert("backing".to_string(), "community_value".to_string());

        Self {
            name,
            description,
            symbol,
            decimals: 2,
            token_type: TokenType::LocalCurrency,
            transferability: TransferabilityRule::FreelyTransferable,
            scoping_rules: ScopingRules {
                geographic_scope: Some(geographic_scope),
                community_scope: None,
                validity_period: None,
                max_supply: None,
                min_balance: Some(0),
            },
            issuer,
            created_at: icn_common::SystemTimeProvider.unix_seconds(),
            metadata,
        }
    }

    /// Create a new bulk purchasing token class.
    pub fn new_bulk_purchasing(
        name: String,
        description: String,
        issuer: Did,
        target_product: String,
        minimum_quantity: u64,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("target_product".to_string(), target_product);
        metadata.insert("minimum_quantity".to_string(), minimum_quantity.to_string());
        metadata.insert("aggregation_period".to_string(), "30_days".to_string());

        Self {
            name,
            description,
            symbol: "BULK".to_string(),
            decimals: 0, // Whole units only
            token_type: TokenType::BulkPurchasing,
            transferability: TransferabilityRule::NonTransferable, // Can't transfer purchase commitments
            scoping_rules: ScopingRules {
                geographic_scope: None,
                community_scope: None,
                validity_period: Some((
                    icn_common::SystemTimeProvider.unix_seconds(),
                    icn_common::SystemTimeProvider.unix_seconds() + (30 * 24 * 60 * 60), // 30 days
                )),
                max_supply: None,
                min_balance: Some(1), // Must commit to at least 1 unit
            },
            issuer,
            created_at: icn_common::SystemTimeProvider.unix_seconds(),
            metadata,
        }
    }
}
