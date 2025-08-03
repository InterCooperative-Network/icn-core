use crate::{ManaLedger, ResourceLedger};
use icn_common::{CommonError, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Comprehensive economic monitoring and alerting service
pub struct EconomicMonitoringService {
    /// Mana ledger for balance monitoring
    mana_ledger: Arc<dyn ManaLedger>,
    /// Resource ledger for token monitoring
    resource_ledger: Arc<dyn ResourceLedger>,
    /// Time provider for timestamps
    time_provider: Arc<dyn TimeProvider>,
    /// Monitoring configuration
    config: MonitoringConfig,
    /// Current health metrics
    current_metrics: Arc<std::sync::RwLock<SystemHealthMetrics>>,
    /// Alert thresholds
    alert_thresholds: AlertThresholds,
    /// Historical data for trend analysis
    history: Arc<std::sync::RwLock<Vec<HealthSnapshot>>>,
}

/// Configuration for economic monitoring
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// How often to run health checks
    pub health_check_interval: Duration,
    /// How long to keep historical data
    pub history_retention: Duration,
    /// Enable alerting
    pub enable_alerts: bool,
    /// Enable trend analysis
    pub enable_trend_analysis: bool,
    /// Maximum number of historical snapshots to keep
    pub max_history_size: usize,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(30),
            history_retention: Duration::from_secs(3600 * 24), // 24 hours
            enable_alerts: true,
            enable_trend_analysis: true,
            max_history_size: 2880, // 24 hours at 30s intervals
        }
    }
}

/// Comprehensive system health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthMetrics {
    /// Overall system health score (0.0 to 1.0)
    pub overall_health: f64,
    /// Mana system health
    pub mana_health: ManaHealthMetrics,
    /// Token system health
    pub token_health: TokenHealthMetrics,
    /// Economic activity metrics
    pub economic_activity: EconomicActivityMetrics,
    /// Network performance metrics
    pub network_performance: NetworkPerformanceMetrics,
    /// Cross-cooperative metrics
    pub cross_cooperative: CrossCooperativeMetrics,
    /// Timestamp of when metrics were collected
    pub timestamp: u64,
}

/// Mana system health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaHealthMetrics {
    /// Total mana in circulation
    pub total_circulation: u64,
    /// Number of active accounts
    pub active_accounts: u64,
    /// Average account balance
    pub average_balance: f64,
    /// Balance distribution (Gini coefficient)
    pub balance_inequality: f64,
    /// Mana regeneration rate
    pub regeneration_rate: f64,
    /// System utilization (spending rate vs regeneration)
    pub utilization_rate: f64,
}

/// Token system health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHealthMetrics {
    /// Number of active token classes
    pub active_token_classes: u64,
    /// Total token supply across all classes
    pub total_token_supply: u64,
    /// Transaction volume (last hour)
    pub transaction_volume: u64,
    /// Failed transaction rate
    pub failed_transaction_rate: f64,
    /// Average transaction size
    pub average_transaction_size: f64,
}

/// Economic activity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicActivityMetrics {
    /// Marketplace activity level
    pub marketplace_activity: f64,
    /// Resource allocation efficiency
    pub allocation_efficiency: f64,
    /// Price stability index
    pub price_stability: f64,
    /// Market liquidity
    pub market_liquidity: f64,
    /// Economic growth rate
    pub growth_rate: f64,
}

/// Network performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPerformanceMetrics {
    /// Transaction processing latency (ms)
    pub transaction_latency: f64,
    /// System throughput (transactions per second)
    pub throughput: f64,
    /// Error rate
    pub error_rate: f64,
    /// Resource utilization
    pub resource_utilization: f64,
}

/// Cross-cooperative metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCooperativeMetrics {
    /// Number of active federations
    pub active_federations: u64,
    /// Cross-federation transaction volume
    pub cross_federation_volume: u64,
    /// Trust level distribution
    pub trust_distribution: HashMap<u32, u64>,
    /// Federation health scores
    pub federation_health: HashMap<String, f64>,
}

/// Alert thresholds for monitoring
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Minimum overall health score before alert
    pub min_overall_health: f64,
    /// Maximum balance inequality before alert
    pub max_balance_inequality: f64,
    /// Maximum failed transaction rate before alert
    pub max_failed_transaction_rate: f64,
    /// Minimum market liquidity before alert
    pub min_market_liquidity: f64,
    /// Maximum transaction latency before alert (ms)
    pub max_transaction_latency: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            min_overall_health: 0.3,
            max_balance_inequality: 0.8,
            max_failed_transaction_rate: 0.1,
            min_market_liquidity: 0.2,
            max_transaction_latency: 5000.0,
        }
    }
}

/// Historical health snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// Health metrics at this point in time
    pub metrics: SystemHealthMetrics,
    /// Alerts generated at this time
    pub alerts: Vec<HealthAlert>,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert category
    pub category: AlertCategory,
    /// Human-readable message
    pub message: String,
    /// Metric value that triggered the alert
    pub metric_value: f64,
    /// Threshold that was exceeded
    pub threshold: f64,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational
    Info,
    /// Warning - attention needed
    Warning,
    /// Critical - immediate action required
    Critical,
    /// Emergency - system stability at risk
    Emergency,
}

/// Alert categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertCategory {
    /// Mana system issues
    ManaSystem,
    /// Token system issues
    TokenSystem,
    /// Economic health issues
    EconomicHealth,
    /// Network performance issues
    NetworkPerformance,
    /// Cross-cooperative issues
    CrossCooperative,
    /// Security issues
    Security,
}

impl EconomicMonitoringService {
    /// Create a new economic monitoring service
    pub fn new(
        mana_ledger: Arc<dyn ManaLedger>,
        resource_ledger: Arc<dyn ResourceLedger>,
        time_provider: Arc<dyn TimeProvider>,
        config: MonitoringConfig,
    ) -> Self {
        Self {
            mana_ledger,
            resource_ledger,
            time_provider,
            config,
            current_metrics: Arc::new(std::sync::RwLock::new(SystemHealthMetrics::default())),
            alert_thresholds: AlertThresholds::default(),
            history: Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }

    /// Start the monitoring service
    pub async fn start(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let mana_ledger = self.mana_ledger.clone();
        let resource_ledger = self.resource_ledger.clone();
        let time_provider = self.time_provider.clone();
        let config = self.config.clone();
        let current_metrics = self.current_metrics.clone();
        let alert_thresholds = self.alert_thresholds.clone();
        let history = self.history.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.health_check_interval);

            loop {
                interval.tick().await;

                if let Err(e) = Self::run_health_check(
                    &mana_ledger,
                    &resource_ledger,
                    &time_provider,
                    &config,
                    &current_metrics,
                    &alert_thresholds,
                    &history,
                )
                .await
                {
                    log::error!("Health check failed: {}", e);
                }
            }
        });

        Ok(handle)
    }

    /// Run a comprehensive health check
    async fn run_health_check(
        mana_ledger: &Arc<dyn ManaLedger>,
        resource_ledger: &Arc<dyn ResourceLedger>,
        time_provider: &Arc<dyn TimeProvider>,
        config: &MonitoringConfig,
        current_metrics: &Arc<std::sync::RwLock<SystemHealthMetrics>>,
        alert_thresholds: &AlertThresholds,
        history: &Arc<std::sync::RwLock<Vec<HealthSnapshot>>>,
    ) -> Result<(), CommonError> {
        let start_time = Instant::now();

        // Collect comprehensive metrics
        let metrics = Self::collect_system_metrics(
            mana_ledger,
            resource_ledger,
            time_provider,
        ).await?;

        // Generate alerts based on thresholds
        let alerts = Self::generate_alerts(&metrics, alert_thresholds);

        // Update current metrics
        {
            let mut current = current_metrics.write().unwrap();
            *current = metrics.clone();
        }

        // Store historical snapshot
        if config.enable_trend_analysis {
            let snapshot = HealthSnapshot { metrics: metrics.clone(), alerts };
            Self::store_historical_snapshot(snapshot, history, config);
        }

        // Update Prometheus metrics
        Self::update_prometheus_metrics(&metrics);

        let duration = start_time.elapsed();
        log::debug!("Health check completed in {:?}", duration);

        Ok(())
    }

    /// Collect comprehensive system metrics
    async fn collect_system_metrics(
        _mana_ledger: &Arc<dyn ManaLedger>,
        _resource_ledger: &Arc<dyn ResourceLedger>,
        time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<SystemHealthMetrics, CommonError> {
        let timestamp = time_provider.unix_seconds();

        // Mock data for demonstration - in real implementation, these would collect actual metrics
        let mana_health = ManaHealthMetrics {
            total_circulation: 1_000_000,
            active_accounts: 500,
            average_balance: 2000.0,
            balance_inequality: 0.3, // Gini coefficient
            regeneration_rate: 0.1,
            utilization_rate: 0.6,
        };

        let token_health = TokenHealthMetrics {
            active_token_classes: 15,
            total_token_supply: 50_000,
            transaction_volume: 100,
            failed_transaction_rate: 0.02,
            average_transaction_size: 25.0,
        };

        let economic_activity = EconomicActivityMetrics {
            marketplace_activity: 0.7,
            allocation_efficiency: 0.8,
            price_stability: 0.9,
            market_liquidity: 0.6,
            growth_rate: 0.05,
        };

        let network_performance = NetworkPerformanceMetrics {
            transaction_latency: 150.0,
            throughput: 50.0,
            error_rate: 0.01,
            resource_utilization: 0.4,
        };

        let cross_cooperative = CrossCooperativeMetrics {
            active_federations: 3,
            cross_federation_volume: 25,
            trust_distribution: [(1, 50), (2, 150), (3, 200)].into_iter().collect(),
            federation_health: [
                ("federation-a".to_string(), 0.9),
                ("federation-b".to_string(), 0.8),
                ("federation-c".to_string(), 0.85),
            ].into_iter().collect(),
        };

        // Calculate overall health score
        let overall_health = Self::calculate_overall_health(
            &mana_health,
            &token_health,
            &economic_activity,
            &network_performance,
            &cross_cooperative,
        );

        Ok(SystemHealthMetrics {
            overall_health,
            mana_health,
            token_health,
            economic_activity,
            network_performance,
            cross_cooperative,
            timestamp,
        })
    }

    /// Calculate overall system health score
    fn calculate_overall_health(
        mana_health: &ManaHealthMetrics,
        token_health: &TokenHealthMetrics,
        economic_activity: &EconomicActivityMetrics,
        network_performance: &NetworkPerformanceMetrics,
        cross_cooperative: &CrossCooperativeMetrics,
    ) -> f64 {
        // Weighted health score calculation
        let mana_score = (1.0 - mana_health.balance_inequality) * 0.7 + mana_health.regeneration_rate * 0.3;
        let token_score = (1.0 - token_health.failed_transaction_rate) * 0.6 + 
                         (token_health.transaction_volume as f64 / 1000.0).min(1.0) * 0.4;
        let economic_score = (economic_activity.marketplace_activity + 
                             economic_activity.allocation_efficiency + 
                             economic_activity.price_stability + 
                             economic_activity.market_liquidity) / 4.0;
        let network_score = (1.0 - network_performance.error_rate) * 0.5 + 
                           (1.0 - (network_performance.transaction_latency / 5000.0).min(1.0)) * 0.5;
        let federation_score = if cross_cooperative.active_federations > 0 {
            cross_cooperative.federation_health.values().sum::<f64>() / cross_cooperative.federation_health.len() as f64
        } else {
            1.0
        };

        // Weighted average
        (mana_score * 0.25 + token_score * 0.25 + economic_score * 0.25 + network_score * 0.15 + federation_score * 0.1)
            .max(0.0).min(1.0)
    }

    /// Generate alerts based on current metrics and thresholds
    fn generate_alerts(
        metrics: &SystemHealthMetrics,
        thresholds: &AlertThresholds,
    ) -> Vec<HealthAlert> {
        let mut alerts = Vec::new();

        // Overall health alert
        if metrics.overall_health < thresholds.min_overall_health {
            alerts.push(HealthAlert {
                severity: if metrics.overall_health < 0.2 { AlertSeverity::Emergency } else { AlertSeverity::Critical },
                category: AlertCategory::EconomicHealth,
                message: format!("System health critically low: {:.2}", metrics.overall_health),
                metric_value: metrics.overall_health,
                threshold: thresholds.min_overall_health,
                suggested_actions: vec![
                    "Check mana regeneration rates".to_string(),
                    "Review token system performance".to_string(),
                    "Examine network performance".to_string(),
                ],
            });
        }

        // Balance inequality alert
        if metrics.mana_health.balance_inequality > thresholds.max_balance_inequality {
            alerts.push(HealthAlert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::ManaSystem,
                message: format!("High mana balance inequality: {:.2}", metrics.mana_health.balance_inequality),
                metric_value: metrics.mana_health.balance_inequality,
                threshold: thresholds.max_balance_inequality,
                suggested_actions: vec![
                    "Review mana allocation policies".to_string(),
                    "Consider redistribution mechanisms".to_string(),
                ],
            });
        }

        // Failed transaction rate alert
        if metrics.token_health.failed_transaction_rate > thresholds.max_failed_transaction_rate {
            alerts.push(HealthAlert {
                severity: AlertSeverity::Critical,
                category: AlertCategory::TokenSystem,
                message: format!("High failed transaction rate: {:.2}%", metrics.token_health.failed_transaction_rate * 100.0),
                metric_value: metrics.token_health.failed_transaction_rate,
                threshold: thresholds.max_failed_transaction_rate,
                suggested_actions: vec![
                    "Check token ledger system".to_string(),
                    "Review transaction validation".to_string(),
                    "Examine network connectivity".to_string(),
                ],
            });
        }

        // Market liquidity alert
        if metrics.economic_activity.market_liquidity < thresholds.min_market_liquidity {
            alerts.push(HealthAlert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::EconomicHealth,
                message: format!("Low market liquidity: {:.2}", metrics.economic_activity.market_liquidity),
                metric_value: metrics.economic_activity.market_liquidity,
                threshold: thresholds.min_market_liquidity,
                suggested_actions: vec![
                    "Encourage marketplace participation".to_string(),
                    "Review pricing mechanisms".to_string(),
                    "Check cross-cooperative trading".to_string(),
                ],
            });
        }

        // Transaction latency alert
        if metrics.network_performance.transaction_latency > thresholds.max_transaction_latency {
            alerts.push(HealthAlert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::NetworkPerformance,
                message: format!("High transaction latency: {:.1}ms", metrics.network_performance.transaction_latency),
                metric_value: metrics.network_performance.transaction_latency,
                threshold: thresholds.max_transaction_latency,
                suggested_actions: vec![
                    "Check network performance".to_string(),
                    "Review system load".to_string(),
                    "Optimize transaction processing".to_string(),
                ],
            });
        }

        alerts
    }

    /// Store historical snapshot
    fn store_historical_snapshot(
        snapshot: HealthSnapshot,
        history: &Arc<std::sync::RwLock<Vec<HealthSnapshot>>>,
        config: &MonitoringConfig,
    ) {
        let mut hist = history.write().unwrap();
        hist.push(snapshot);

        // Cleanup old entries
        let hist_len = hist.len();
        if hist_len > config.max_history_size {
            hist.drain(0..hist_len - config.max_history_size);
        }
    }

    /// Update Prometheus metrics
    fn update_prometheus_metrics(metrics: &SystemHealthMetrics) {
        // Update static metrics with proper type casting
        crate::metrics::ECONOMIC_HEALTH_SCORE.set((metrics.overall_health * 100.0) as i64);
        crate::metrics::TOTAL_MANA_CIRCULATION.set(metrics.mana_health.total_circulation as i64);
        crate::metrics::ACTIVE_ACCOUNTS.set(metrics.mana_health.active_accounts as i64);
        crate::metrics::RESOURCE_UTILIZATION.set((metrics.network_performance.resource_utilization * 100.0) as i64);
        crate::metrics::MARKET_LIQUIDITY.set((metrics.economic_activity.market_liquidity * 100.0) as i64);
        crate::metrics::PRICE_STABILITY.set((metrics.economic_activity.price_stability * 100.0) as i64);
    }

    /// Get current health metrics
    pub fn get_current_metrics(&self) -> SystemHealthMetrics {
        self.current_metrics.read().unwrap().clone()
    }

    /// Get historical snapshots
    pub fn get_history(&self, hours: u64) -> Vec<HealthSnapshot> {
        let cutoff_time = self.time_provider.unix_seconds() - (hours * 3600);
        let history = self.history.read().unwrap();
        history
            .iter()
            .filter(|snapshot| snapshot.metrics.timestamp >= cutoff_time)
            .cloned()
            .collect()
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<HealthAlert> {
        let metrics = self.get_current_metrics();
        Self::generate_alerts(&metrics, &self.alert_thresholds)
    }
}

impl Default for SystemHealthMetrics {
    fn default() -> Self {
        Self {
            overall_health: 1.0,
            mana_health: ManaHealthMetrics {
                total_circulation: 0,
                active_accounts: 0,
                average_balance: 0.0,
                balance_inequality: 0.0,
                regeneration_rate: 0.0,
                utilization_rate: 0.0,
            },
            token_health: TokenHealthMetrics {
                active_token_classes: 0,
                total_token_supply: 0,
                transaction_volume: 0,
                failed_transaction_rate: 0.0,
                average_transaction_size: 0.0,
            },
            economic_activity: EconomicActivityMetrics {
                marketplace_activity: 0.0,
                allocation_efficiency: 0.0,
                price_stability: 1.0,
                market_liquidity: 0.0,
                growth_rate: 0.0,
            },
            network_performance: NetworkPerformanceMetrics {
                transaction_latency: 0.0,
                throughput: 0.0,
                error_rate: 0.0,
                resource_utilization: 0.0,
            },
            cross_cooperative: CrossCooperativeMetrics {
                active_federations: 0,
                cross_federation_volume: 0,
                trust_distribution: HashMap::new(),
                federation_health: HashMap::new(),
            },
            timestamp: 0,
        }
    }
}