use icn_common::{CommonError, Did};
use rusqlite::{Connection, OptionalExtension};
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
        conn.execute(
            "UPDATE mana_balances SET amount = amount + ?1",
            [amount as i64],
        )
        .map_err(|e| CommonError::DatabaseError(format!("{e}")))?;
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
        match stmt.query_map([], |row| row.get::<_, String>(0)) {
            Ok(rows) => rows
                .filter_map(|r| r.ok().and_then(|s| Did::from_str(&s).ok()))
                .collect(),
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
