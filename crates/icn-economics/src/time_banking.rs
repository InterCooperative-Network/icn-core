use crate::{ResourceLedger, TokenClassId, TokenType};
use icn_common::{CommonError, Did, SystemTimeProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specialized functionality for time banking tokens.
/// Time banking allows communities to exchange labor hours on an equal basis.
///
/// Record of work performed in a time banking system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRecord {
    /// Unique identifier for this time record.
    pub record_id: String,
    /// Person who performed the work.
    pub worker: Did,
    /// Person or organization who received the benefit.
    pub beneficiary: Did,
    /// Type of work performed.
    pub work_type: String,
    /// Description of the work done.
    pub description: String,
    /// Number of hours worked (can be fractional).
    pub hours: f64,
    /// Skill level required (beginner, intermediate, advanced, expert).
    pub skill_level: String,
    /// Unix timestamp when work was performed.
    pub performed_at: u64,
    /// Unix timestamp when record was created.
    pub recorded_at: u64,
    /// Status of the time record.
    pub status: TimeRecordStatus,
    /// Optional metadata.
    pub metadata: HashMap<String, String>,
}

/// Status of a time banking record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeRecordStatus {
    /// Work has been performed and recorded.
    Recorded,
    /// Work has been verified by beneficiary.
    Verified,
    /// Work has been disputed.
    Disputed,
    /// Record has been cancelled.
    Cancelled,
}

/// Trait for managing time banking records.
pub trait TimeBankingStore: Send + Sync {
    /// Record time worked.
    fn record_time(&self, record: TimeRecord) -> Result<(), CommonError>;
    /// Get a time record by ID.
    fn get_time_record(&self, record_id: &str) -> Option<TimeRecord>;
    /// Update a time record.
    fn update_time_record(&self, record: TimeRecord) -> Result<(), CommonError>;
    /// Get all time records for a worker.
    fn get_worker_records(&self, worker: &Did) -> Vec<TimeRecord>;
    /// Get all time records for a beneficiary.
    fn get_beneficiary_records(&self, beneficiary: &Did) -> Vec<TimeRecord>;
    /// Get time records by work type.
    fn get_records_by_work_type(&self, work_type: &str) -> Vec<TimeRecord>;
}

/// In-memory time banking store for development and testing.
#[derive(Default)]
pub struct InMemoryTimeBankingStore {
    records: std::sync::Mutex<HashMap<String, TimeRecord>>,
}

impl InMemoryTimeBankingStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TimeBankingStore for InMemoryTimeBankingStore {
    fn record_time(&self, record: TimeRecord) -> Result<(), CommonError> {
        let mut records = self.records.lock().unwrap();
        if records.contains_key(&record.record_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Time record {} already exists",
                record.record_id
            )));
        }
        records.insert(record.record_id.clone(), record);
        Ok(())
    }

    fn get_time_record(&self, record_id: &str) -> Option<TimeRecord> {
        let records = self.records.lock().unwrap();
        records.get(record_id).cloned()
    }

    fn update_time_record(&self, record: TimeRecord) -> Result<(), CommonError> {
        let mut records = self.records.lock().unwrap();
        records.insert(record.record_id.clone(), record);
        Ok(())
    }

    fn get_worker_records(&self, worker: &Did) -> Vec<TimeRecord> {
        let records = self.records.lock().unwrap();
        let mut results: Vec<TimeRecord> = records
            .values()
            .filter(|record| &record.worker == worker)
            .cloned()
            .collect();

        // Sort by performance date (newest first)
        results.sort_by(|a, b| b.performed_at.cmp(&a.performed_at));
        results
    }

    fn get_beneficiary_records(&self, beneficiary: &Did) -> Vec<TimeRecord> {
        let records = self.records.lock().unwrap();
        let mut results: Vec<TimeRecord> = records
            .values()
            .filter(|record| &record.beneficiary == beneficiary)
            .cloned()
            .collect();

        // Sort by performance date (newest first)
        results.sort_by(|a, b| b.performed_at.cmp(&a.performed_at));
        results
    }

    fn get_records_by_work_type(&self, work_type: &str) -> Vec<TimeRecord> {
        let records = self.records.lock().unwrap();
        let mut results: Vec<TimeRecord> = records
            .values()
            .filter(|record| record.work_type == work_type)
            .cloned()
            .collect();

        // Sort by performance date (newest first)
        results.sort_by(|a, b| b.performed_at.cmp(&a.performed_at));
        results
    }
}

/// Record time worked and issue time banking tokens.
pub fn record_and_mint_time_tokens<L: ResourceLedger, T: TimeBankingStore>(
    resource_ledger: &L,
    time_store: &T,
    time_token_class: &TokenClassId,
    worker: &Did,
    beneficiary: &Did,
    work_type: String,
    description: String,
    hours: f64,
    skill_level: String,
) -> Result<String, CommonError> {
    // Validate that the token class is for time banking
    let token_class = resource_ledger.get_class(time_token_class).ok_or_else(|| {
        CommonError::InvalidInputError(format!("Token class {time_token_class} not found"))
    })?;

    if token_class.token_type != TokenType::TimeBanking {
        return Err(CommonError::InvalidInputError(
            "Token class is not for time banking".into(),
        ));
    }

    // Create time record
    let record_id = format!("time_{}_{}", worker, SystemTimeProvider.unix_seconds());
    let time_record = TimeRecord {
        record_id: record_id.clone(),
        worker: worker.clone(),
        beneficiary: beneficiary.clone(),
        work_type,
        description,
        hours,
        skill_level,
        performed_at: SystemTimeProvider.unix_seconds(),
        recorded_at: SystemTimeProvider.unix_seconds(),
        status: TimeRecordStatus::Recorded,
        metadata: HashMap::new(),
    };

    // Record the time
    time_store.record_time(time_record)?;

    // Convert hours to token units (considering decimals)
    let token_amount = (hours * 100.0) as u64; // 2 decimal places

    // Mint time tokens to the worker
    resource_ledger.mint(time_token_class, worker, token_amount)?;

    Ok(record_id)
}

/// Verify time worked and update record status.
pub fn verify_time_record<T: TimeBankingStore>(
    time_store: &T,
    record_id: &str,
    verifier: &Did,
) -> Result<(), CommonError> {
    let mut record = time_store.get_time_record(record_id).ok_or_else(|| {
        CommonError::InvalidInputError(format!("Time record {record_id} not found"))
    })?;

    // Only beneficiary can verify
    if &record.beneficiary != verifier {
        return Err(CommonError::PolicyDenied(
            "Only beneficiary can verify time record".into(),
        ));
    }

    // Update status to verified
    record.status = TimeRecordStatus::Verified;
    time_store.update_time_record(record)?;

    Ok(())
}

/// Calculate total hours worked by a person in a time period.
pub fn calculate_total_hours<T: TimeBankingStore>(
    time_store: &T,
    worker: &Did,
    start_time: u64,
    end_time: u64,
) -> f64 {
    let records = time_store.get_worker_records(worker);
    records
        .into_iter()
        .filter(|record| {
            record.performed_at >= start_time
                && record.performed_at <= end_time
                && record.status == TimeRecordStatus::Verified
        })
        .map(|record| record.hours)
        .sum()
}

/// Get work statistics for a community.
pub fn get_community_work_stats<T: TimeBankingStore>(
    time_store: &T,
    workers: &[Did],
    start_time: u64,
    end_time: u64,
) -> WorkStatistics {
    let mut total_hours = 0.0;
    let mut work_types = HashMap::new();
    let mut skill_levels = HashMap::new();
    let mut record_count = 0;

    for worker in workers {
        let records = time_store.get_worker_records(worker);
        for record in records {
            if record.performed_at >= start_time
                && record.performed_at <= end_time
                && record.status == TimeRecordStatus::Verified
            {
                total_hours += record.hours;
                record_count += 1;

                *work_types.entry(record.work_type).or_insert(0.0) += record.hours;
                *skill_levels.entry(record.skill_level).or_insert(0.0) += record.hours;
            }
        }
    }

    WorkStatistics {
        total_hours,
        record_count,
        work_types,
        skill_levels,
        active_workers: workers.len(),
    }
}

/// Statistics about work done in a community.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkStatistics {
    /// Total hours worked.
    pub total_hours: f64,
    /// Number of work records.
    pub record_count: usize,
    /// Hours by work type.
    pub work_types: HashMap<String, f64>,
    /// Hours by skill level.
    pub skill_levels: HashMap<String, f64>,
    /// Number of active workers.
    pub active_workers: usize,
}

impl TimeRecord {
    /// Create a new time record.
    pub fn new(
        worker: Did,
        beneficiary: Did,
        work_type: String,
        description: String,
        hours: f64,
        skill_level: String,
    ) -> Self {
        let now = SystemTimeProvider.unix_seconds();
        Self {
            record_id: format!("time_{worker}_{now}"),
            worker,
            beneficiary,
            work_type,
            description,
            hours,
            skill_level,
            performed_at: now,
            recorded_at: now,
            status: TimeRecordStatus::Recorded,
            metadata: HashMap::new(),
        }
    }
}
