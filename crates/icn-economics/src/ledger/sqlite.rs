use icn_common::{CommonError, Did};
use rusqlite::{Connection, OptionalExtension};
use super::{ResourceLedger, TokenClass, TokenClassId};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub struct SqliteManaLedger {
    path: PathBuf,
}

impl SqliteManaLedger {
    /// Create a new SQLite based mana ledger stored at `path`.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let conn = Connection::open(&path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mana_balances (did TEXT PRIMARY KEY, amount INTEGER)",
            [],
        )
        .map_err(|e| CommonError::DatabaseError(format!("Failed to create table: {e}")))?;
        Ok(Self { path })
    }

    fn write_balance(&self, account: &Did, amount: u64) -> Result<(), CommonError> {
        let conn = Connection::open(&self.path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        conn.execute(
            "INSERT INTO mana_balances(did, amount) VALUES (?1, ?2) \
             ON CONFLICT(did) DO UPDATE SET amount=excluded.amount",
            (&account.to_string(), amount as i64),
        )
        .map_err(|e| CommonError::DatabaseError(format!("Failed to write balance: {e}")))?;
        Ok(())
    }

    fn read_balance(&self, account: &Did) -> Result<u64, CommonError> {
        let conn = Connection::open(&self.path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        let amt: Option<i64> = conn
            .query_row(
                "SELECT amount FROM mana_balances WHERE did=?1",
                [&account.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| CommonError::DatabaseError(format!("Failed to read balance: {e}")))?;
        Ok(amt.unwrap_or(0) as u64)
    }

    pub fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        let conn = Connection::open(&self.path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        let mut stmt = conn
            .prepare("SELECT did, amount FROM mana_balances")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to fetch balances: {e}")))?;
        let rows = stmt
            .query_map([], |row| {
                let did: String = row.get(0)?;
                let amount: i64 = row.get(1)?;
                Ok((did, amount))
            })
            .map_err(|e| CommonError::DatabaseError(format!("{e}")))?;
        for row in rows {
            let (did_str, bal) = row.map_err(|e| CommonError::DatabaseError(format!("{e}")))?;
            let did = Did::from_str(&did_str)
                .map_err(|e| CommonError::InvalidInputError(format!("{e}")))?;
            let new_bal = (bal as u64).saturating_add(amount);
            self.write_balance(&did, new_bal)?;
        }
        Ok(())
    }

    /// Fetch all account DIDs present in the ledger.
    pub fn all_accounts(&self) -> Vec<Did> {
        let conn = match Connection::open(&self.path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let mut stmt = match conn.prepare("SELECT did FROM mana_balances") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        // Collect the results immediately to avoid lifetime issues
        let rows_result = stmt.query_map([], |row| row.get::<_, String>(0));
        match rows_result {
            Ok(rows) => {
                let mut accounts = Vec::new();
                for did_string in rows.flatten() {
                    if let Ok(did) = Did::from_str(&did_string) {
                        accounts.push(did);
                    }
                }
                accounts
            }
            Err(_) => Vec::new(),
        }
    }
}

impl crate::ManaLedger for SqliteManaLedger {
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
        SqliteManaLedger::credit_all(self, amount)
    }

    fn all_accounts(&self) -> Vec<Did> {
        SqliteManaLedger::all_accounts(self)
    }
}

// --- SQLite based Resource Ledger --------------------------------------------

#[derive(Debug)]
pub struct SqliteResourceLedger {
    path: PathBuf,
}

impl SqliteResourceLedger {
    /// Create a new SQLite based resource ledger stored at `path`.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let conn = Connection::open(&path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS token_classes (id TEXT PRIMARY KEY, data TEXT)",
            [],
        )
        .map_err(|e| CommonError::DatabaseError(format!("Failed to create classes table: {e}")))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS token_balances (class_id TEXT, did TEXT, amount INTEGER, PRIMARY KEY(class_id,did))",
            [],
        )
        .map_err(|e| CommonError::DatabaseError(format!("Failed to create balances table: {e}")))?;
        Ok(Self { path })
    }

    fn write_class(&self, id: &TokenClassId, class: &TokenClass) -> Result<(), CommonError> {
        let conn = Connection::open(&self.path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        let data = serde_json::to_string(class)
            .map_err(|e| CommonError::SerializationError(format!("Failed to serialize class: {e}")))?;
        conn.execute(
            "INSERT INTO token_classes(id, data) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
            (&id, &data),
        )
        .map_err(|e| CommonError::DatabaseError(format!("Failed to write class: {e}")))?;
        Ok(())
    }

    fn read_class(&self, id: &TokenClassId) -> Result<Option<TokenClass>, CommonError> {
        let conn = Connection::open(&self.path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        let data: Option<String> = conn
            .query_row("SELECT data FROM token_classes WHERE id=?1", [id], |row| row.get(0))
            .optional()
            .map_err(|e| CommonError::DatabaseError(format!("Failed to read class: {e}")))?;
        if let Some(data) = data {
            Ok(Some(serde_json::from_str(&data).map_err(|e| {
                CommonError::DeserializationError(format!("Failed to deserialize class: {e}"))
            })?))
        } else {
            Ok(None)
        }
    }

    fn write_balance(&self, class: &TokenClassId, did: &Did, amount: u64) -> Result<(), CommonError> {
        let conn = Connection::open(&self.path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        conn.execute(
            "INSERT INTO token_balances(class_id, did, amount) VALUES (?1, ?2, ?3) ON CONFLICT(class_id,did) DO UPDATE SET amount=excluded.amount",
            (&class, &did.to_string(), amount as i64),
        )
        .map_err(|e| CommonError::DatabaseError(format!("Failed to write balance: {e}")))?;
        Ok(())
    }

    fn read_balance(&self, class: &TokenClassId, did: &Did) -> Result<u64, CommonError> {
        let conn = Connection::open(&self.path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sqlite DB: {e}")))?;
        let amt: Option<i64> = conn
            .query_row(
                "SELECT amount FROM token_balances WHERE class_id=?1 AND did=?2",
                (&class, &did.to_string()),
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| CommonError::DatabaseError(format!("Failed to read balance: {e}")))?;
        Ok(amt.unwrap_or(0) as u64)
    }
}

impl ResourceLedger for SqliteResourceLedger {
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
