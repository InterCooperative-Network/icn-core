use once_cell::sync::Lazy;
use prometheus_client::metrics::{counter::Counter, gauge::Gauge, histogram::Histogram};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Counts calls to `get_balance`.
pub static GET_BALANCE_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `credit_mana`.
pub static CREDIT_MANA_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `spend_mana`.
pub static SPEND_MANA_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts successful token mints.
pub static TOKEN_MINT_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts successful token burns.
pub static TOKEN_BURN_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts successful token transfers.
pub static TOKEN_TRANSFER_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts failed transactions due to policy violations.
pub static POLICY_VIOLATIONS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts cross-cooperative marketplace transactions.
pub static MARKETPLACE_TRANSACTIONS: Lazy<Counter> = Lazy::new(Counter::default);

/// Tracks total mana in circulation.
pub static TOTAL_MANA_CIRCULATION: Lazy<Gauge> = Lazy::new(Gauge::default);

/// Tracks number of active accounts.
pub static ACTIVE_ACCOUNTS: Lazy<Gauge> = Lazy::new(Gauge::default);

/// Tracks average mana regeneration rate.
pub static MANA_REGENERATION_RATE: Lazy<Gauge> = Lazy::new(Gauge::default);

/// Tracks economic health score.
pub static ECONOMIC_HEALTH_SCORE: Lazy<Gauge> = Lazy::new(Gauge::default);

/// Tracks resource utilization efficiency.
pub static RESOURCE_UTILIZATION: Lazy<Gauge> = Lazy::new(Gauge::default);

/// Tracks market liquidity level.
pub static MARKET_LIQUIDITY: Lazy<Gauge> = Lazy::new(Gauge::default);

/// Tracks price stability index.
pub static PRICE_STABILITY: Lazy<Gauge> = Lazy::new(Gauge::default);

/// Histogram of transaction processing times.
pub static TRANSACTION_DURATION: Lazy<Histogram> = Lazy::new(|| {
    let buckets = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0];
    Histogram::new(buckets.into_iter())
});

/// Histogram of mana allocation amounts.
pub static MANA_ALLOCATION_AMOUNTS: Lazy<Histogram> = Lazy::new(|| {
    let buckets = [1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0];
    Histogram::new(buckets.into_iter())
});

/// Histogram of resource allocation efficiency scores.
pub static ALLOCATION_EFFICIENCY: Lazy<Histogram> = Lazy::new(|| {
    let buckets = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    Histogram::new(buckets.into_iter())
});

/// Enhanced performance metrics tracker for cross-cooperative operations.
#[derive(Debug, Clone, Default)]
pub struct CrossCooperativeMetrics {
    /// Number of successful cross-cooperative resource shares
    pub successful_shares: u64,
    /// Number of failed cross-cooperative attempts
    pub failed_shares: u64,
    /// Total volume of resources shared across cooperatives
    pub total_shared_volume: u64,
    /// Average efficiency of cross-cooperative allocations
    pub avg_allocation_efficiency: f64,
    /// Number of active cooperatives
    pub active_cooperatives: u64,
    /// Cross-cooperative transaction latency statistics
    pub transaction_latencies: Vec<f64>,
}

/// Resource allocation performance tracker.
#[derive(Debug, Clone, Default)]
pub struct AllocationPerformanceMetrics {
    /// Total allocations processed
    pub total_allocations: u64,
    /// Successful allocations
    pub successful_allocations: u64,
    /// Failed allocations
    pub failed_allocations: u64,
    /// Average allocation time
    pub avg_allocation_time_ms: f64,
    /// Resource utilization after allocation
    pub post_allocation_utilization: f64,
    /// Allocation accuracy score (how well allocations match actual needs)
    pub allocation_accuracy: f64,
}

/// Economic validation metrics.
#[derive(Debug, Clone, Default)]
pub struct ValidationMetrics {
    /// Total validation checks performed
    pub total_validations: u64,
    /// Passed validations
    pub passed_validations: u64,
    /// Failed validations
    pub failed_validations: u64,
    /// Average validation time
    pub avg_validation_time_ms: f64,
    /// Policy enforcement actions taken
    pub policy_enforcements: u64,
    /// Transaction rollbacks due to validation failures
    pub rollbacks: u64,
}

/// Mana ledger performance metrics.
#[derive(Debug, Clone, Default)]
pub struct ManaLedgerMetrics {
    /// Total mana operations
    pub total_operations: u64,
    /// Average operation latency
    pub avg_operation_latency_ms: f64,
    /// Mana regeneration events
    pub regeneration_events: u64,
    /// Total mana regenerated
    pub total_mana_regenerated: u64,
    /// Number of accounts with insufficient mana
    pub insufficient_mana_events: u64,
    /// Average account balance
    pub avg_account_balance: f64,
}

/// Global metrics registry for the economics module.
pub static METRICS_REGISTRY: Lazy<Arc<RwLock<EconomicsMetricsRegistry>>> =
    Lazy::new(|| Arc::new(RwLock::new(EconomicsMetricsRegistry::new())));

/// Central registry for all economics metrics.
#[derive(Debug, Default)]
pub struct EconomicsMetricsRegistry {
    pub cross_cooperative: CrossCooperativeMetrics,
    pub allocation_performance: AllocationPerformanceMetrics,
    pub validation: ValidationMetrics,
    pub mana_ledger: ManaLedgerMetrics,
    pub custom_metrics: HashMap<String, f64>,
}

impl EconomicsMetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful cross-cooperative resource share.
    pub fn record_cross_cooperative_share(&mut self, volume: u64, efficiency: f64) {
        self.cross_cooperative.successful_shares += 1;
        self.cross_cooperative.total_shared_volume += volume;
        
        // Update average efficiency using exponential moving average
        let alpha = 0.1;
        self.cross_cooperative.avg_allocation_efficiency = 
            alpha * efficiency + (1.0 - alpha) * self.cross_cooperative.avg_allocation_efficiency;
    }

    /// Record a failed cross-cooperative attempt.
    pub fn record_cross_cooperative_failure(&mut self) {
        self.cross_cooperative.failed_shares += 1;
    }

    /// Record allocation performance metrics.
    pub fn record_allocation_performance(&mut self, success: bool, time_ms: f64, efficiency: f64) {
        self.allocation_performance.total_allocations += 1;
        if success {
            self.allocation_performance.successful_allocations += 1;
        } else {
            self.allocation_performance.failed_allocations += 1;
        }

        // Update average allocation time
        let n = self.allocation_performance.total_allocations as f64;
        self.allocation_performance.avg_allocation_time_ms = 
            (self.allocation_performance.avg_allocation_time_ms * (n - 1.0) + time_ms) / n;

        // Update allocation accuracy
        let alpha = 0.1;
        self.allocation_performance.allocation_accuracy = 
            alpha * efficiency + (1.0 - alpha) * self.allocation_performance.allocation_accuracy;

        // Update prometheus metrics
        ALLOCATION_EFFICIENCY.observe(efficiency);
    }

    /// Record validation metrics.
    pub fn record_validation(&mut self, passed: bool, time_ms: f64) {
        self.validation.total_validations += 1;
        if passed {
            self.validation.passed_validations += 1;
        } else {
            self.validation.failed_validations += 1;
            POLICY_VIOLATIONS.inc();
        }

        // Update average validation time
        let n = self.validation.total_validations as f64;
        self.validation.avg_validation_time_ms = 
            (self.validation.avg_validation_time_ms * (n - 1.0) + time_ms) / n;
    }

    /// Record mana ledger operation.
    pub fn record_mana_operation(&mut self, operation_time_ms: f64) {
        self.mana_ledger.total_operations += 1;
        
        // Update average operation latency
        let n = self.mana_ledger.total_operations as f64;
        self.mana_ledger.avg_operation_latency_ms = 
            (self.mana_ledger.avg_operation_latency_ms * (n - 1.0) + operation_time_ms) / n;

        // Update prometheus metrics
        TRANSACTION_DURATION.observe(operation_time_ms / 1000.0); // Convert to seconds
    }

    /// Record mana regeneration event.
    pub fn record_mana_regeneration(&mut self, amount: u64) {
        self.mana_ledger.regeneration_events += 1;
        self.mana_ledger.total_mana_regenerated += amount;
        MANA_REGENERATION_RATE.set((self.mana_ledger.total_mana_regenerated as f64 / self.mana_ledger.regeneration_events as f64) as i64);
    }

    /// Record insufficient mana event.
    pub fn record_insufficient_mana(&mut self) {
        self.mana_ledger.insufficient_mana_events += 1;
    }

    /// Update system-wide metrics.
    pub fn update_system_metrics(&mut self, total_mana: u64, active_accounts: u64, health_score: f64) {
        TOTAL_MANA_CIRCULATION.set(total_mana as i64);
        ACTIVE_ACCOUNTS.set(active_accounts as i64);
        ECONOMIC_HEALTH_SCORE.set(health_score as i64);
        
        if active_accounts > 0 {
            self.mana_ledger.avg_account_balance = total_mana as f64 / active_accounts as f64;
        }
    }

    /// Get comprehensive metrics summary.
    pub fn get_summary(&self) -> MetricsSummary {
        MetricsSummary {
            cross_cooperative_success_rate: if self.cross_cooperative.successful_shares + self.cross_cooperative.failed_shares > 0 {
                self.cross_cooperative.successful_shares as f64 / 
                (self.cross_cooperative.successful_shares + self.cross_cooperative.failed_shares) as f64
            } else { 1.0 },
            allocation_success_rate: if self.allocation_performance.total_allocations > 0 {
                self.allocation_performance.successful_allocations as f64 / 
                self.allocation_performance.total_allocations as f64
            } else { 1.0 },
            validation_success_rate: if self.validation.total_validations > 0 {
                self.validation.passed_validations as f64 / 
                self.validation.total_validations as f64
            } else { 1.0 },
            avg_operation_latency_ms: self.mana_ledger.avg_operation_latency_ms,
            total_shared_volume: self.cross_cooperative.total_shared_volume,
            avg_allocation_efficiency: self.allocation_performance.allocation_accuracy,
        }
    }
}

/// Summary of key metrics for monitoring.
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub cross_cooperative_success_rate: f64,
    pub allocation_success_rate: f64,
    pub validation_success_rate: f64,
    pub avg_operation_latency_ms: f64,
    pub total_shared_volume: u64,
    pub avg_allocation_efficiency: f64,
}

/// Helper functions for recording metrics throughout the economics module.
pub mod helpers {
    use super::*;
    use std::time::Instant;

    /// Time a function execution and record the metrics.
    pub fn time_operation<F, R>(operation: F) -> (R, f64)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration_ms = start.elapsed().as_millis() as f64;
        (result, duration_ms)
    }

    /// Record a marketplace transaction.
    pub fn record_marketplace_transaction(volume: u64) {
        MARKETPLACE_TRANSACTIONS.inc();
        if let Ok(mut registry) = METRICS_REGISTRY.write() {
            registry.record_cross_cooperative_share(volume, 1.0);
        }
    }

    /// Record a token operation.
    pub fn record_token_operation(operation_type: &str, amount: u64) {
        match operation_type {
            "mint" => { TOKEN_MINT_CALLS.inc(); },
            "burn" => { TOKEN_BURN_CALLS.inc(); },
            "transfer" => { TOKEN_TRANSFER_CALLS.inc(); },
            _ => {}
        }
        MANA_ALLOCATION_AMOUNTS.observe(amount as f64);
    }

    /// Update resource utilization metrics.
    pub fn update_resource_utilization(utilization: f64) {
        RESOURCE_UTILIZATION.set(utilization as i64);
    }

    /// Update market metrics.
    pub fn update_market_metrics(liquidity: f64, stability: f64) {
        MARKET_LIQUIDITY.set(liquidity as i64);
        PRICE_STABILITY.set(stability as i64);
    }
}
