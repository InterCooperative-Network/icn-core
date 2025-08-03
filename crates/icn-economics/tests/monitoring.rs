use icn_common::SystemTimeProvider;
use icn_economics::{
    monitoring::{
        EconomicMonitoringService, MonitoringConfig, AlertSeverity, AlertCategory,
        SystemHealthMetrics,
    },
};
// Simple stub implementations for testing
struct TestManaLedger;
struct TestResourceLedger;

impl icn_economics::ManaLedger for TestManaLedger {
    fn get_balance(&self, _did: &icn_common::Did) -> u64 { 0 }
    fn set_balance(&self, _did: &icn_common::Did, _amount: u64) -> Result<(), icn_common::CommonError> { Ok(()) }
    fn spend(&self, _did: &icn_common::Did, _amount: u64) -> Result<(), icn_common::CommonError> { Ok(()) }
    fn credit(&self, _did: &icn_common::Did, _amount: u64) -> Result<(), icn_common::CommonError> { Ok(()) }
}

impl icn_economics::ResourceLedger for TestResourceLedger {
    fn create_class(&self, _class_id: &String, _class: icn_economics::TokenClass) -> Result<(), icn_common::CommonError> { Ok(()) }
    fn get_class(&self, _class_id: &String) -> Option<icn_economics::TokenClass> { None }
    fn update_class(&self, _class_id: &String, _class: icn_economics::TokenClass) -> Result<(), icn_common::CommonError> { Ok(()) }
    fn list_classes(&self) -> Vec<(String, icn_economics::TokenClass)> { Vec::new() }
    fn mint(&self, _class_id: &String, _owner: &icn_common::Did, _amount: u64) -> Result<(), icn_common::CommonError> { Ok(()) }
    fn burn(&self, _class_id: &String, _owner: &icn_common::Did, _amount: u64) -> Result<(), icn_common::CommonError> { Ok(()) }
    fn transfer(&self, _class_id: &String, _from: &icn_common::Did, _to: &icn_common::Did, _amount: u64) -> Result<(), icn_common::CommonError> { Ok(()) }
    fn get_balance(&self, _class_id: &String, _owner: &icn_common::Did) -> u64 { 0 }
    fn can_transfer(&self, _class_id: &String, _from: &icn_common::Did, _to: &icn_common::Did, _amount: u64) -> Result<bool, icn_common::CommonError> { Ok(true) }
    fn get_transfer_history(&self, _class_id: &String, _did: &icn_common::Did) -> Vec<icn_economics::TransferRecord> { Vec::new() }
}
use std::sync::Arc;
use std::time::Duration;

#[test]
fn monitoring_service_creation() {
    let mana_ledger = Arc::new(TestManaLedger);
    let resource_ledger = Arc::new(TestResourceLedger);
    let time_provider = Arc::new(SystemTimeProvider);
    
    let config = MonitoringConfig {
        health_check_interval: Duration::from_secs(10),
        enable_alerts: true,
        enable_trend_analysis: true,
        ..Default::default()
    };
    
    let monitoring_service = EconomicMonitoringService::new(
        mana_ledger,
        resource_ledger,
        time_provider,
        config,
    );
    
    // Test getting initial metrics
    let metrics = monitoring_service.get_current_metrics();
    assert_eq!(metrics.overall_health, 1.0); // Default healthy state
    
    // Test getting active alerts
    let alerts = monitoring_service.get_active_alerts();
    // With default healthy metrics, there might be alerts for 0 values which are below thresholds
    // This is expected for a newly initialized system
    println!("Alerts generated: {}", alerts.len());
    for alert in &alerts {
        println!("Alert: {:?} - {}", alert.severity, alert.message);
    }
    
    // Test getting history
    let history = monitoring_service.get_history(1); // Last 1 hour
    assert!(history.is_empty()); // No history yet
}

#[test]
fn health_metrics_default() {
    let metrics = SystemHealthMetrics::default();
    
    assert_eq!(metrics.overall_health, 1.0);
    assert_eq!(metrics.mana_health.total_circulation, 0);
    assert_eq!(metrics.token_health.active_token_classes, 0);
    assert_eq!(metrics.economic_activity.price_stability, 1.0);
    assert_eq!(metrics.network_performance.error_rate, 0.0);
    assert_eq!(metrics.cross_cooperative.active_federations, 0);
}

#[test]
fn alert_categorization() {
    // Test alert severity ordering
    assert!(AlertSeverity::Emergency > AlertSeverity::Critical);
    assert!(AlertSeverity::Critical > AlertSeverity::Warning);
    assert!(AlertSeverity::Warning > AlertSeverity::Info);
    
    // Test alert categories
    let categories = vec![
        AlertCategory::ManaSystem,
        AlertCategory::TokenSystem,
        AlertCategory::EconomicHealth,
        AlertCategory::NetworkPerformance,
        AlertCategory::CrossCooperative,
        AlertCategory::Security,
    ];
    
    // Ensure all categories are distinct
    for (i, cat1) in categories.iter().enumerate() {
        for (j, cat2) in categories.iter().enumerate() {
            if i != j {
                assert_ne!(cat1, cat2);
            }
        }
    }
}

#[tokio::test]
async fn monitoring_service_start() {
    let mana_ledger = Arc::new(TestManaLedger);
    let resource_ledger = Arc::new(TestResourceLedger);
    let time_provider = Arc::new(SystemTimeProvider);
    
    let config = MonitoringConfig {
        health_check_interval: Duration::from_millis(100), // Fast for testing
        enable_alerts: true,
        enable_trend_analysis: true,
        max_history_size: 10,
        ..Default::default()
    };
    
    let monitoring_service = EconomicMonitoringService::new(
        mana_ledger,
        resource_ledger,
        time_provider,
        config,
    );
    
    // Start the monitoring service
    let handle = monitoring_service.start().await.unwrap();
    
    // Let it run for a short time
    tokio::time::sleep(Duration::from_millis(250)).await;
    
    // Stop the service
    handle.abort();
    
    // Check that metrics were collected
    let metrics = monitoring_service.get_current_metrics();
    assert!(metrics.timestamp > 0); // Should have been updated
}

#[test]
fn monitoring_config_defaults() {
    let config = MonitoringConfig::default();
    
    assert_eq!(config.health_check_interval, Duration::from_secs(30));
    assert_eq!(config.history_retention, Duration::from_secs(3600 * 24));
    assert!(config.enable_alerts);
    assert!(config.enable_trend_analysis);
    assert_eq!(config.max_history_size, 2880);
}