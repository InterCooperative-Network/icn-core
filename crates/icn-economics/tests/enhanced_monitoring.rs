use icn_economics::monitoring::{MonitoringConfig, SystemStatus, TrendAnalysis, TrendDirection};

#[test]
fn test_monitoring_config_defaults() {
    let config = MonitoringConfig::default();

    assert_eq!(config.health_check_interval.as_secs(), 30);
    assert_eq!(config.history_retention.as_secs(), 3600 * 24);
    assert!(config.enable_alerts);
    assert!(config.enable_trend_analysis);
    assert_eq!(config.max_history_size, 2880);
}

#[test]
fn test_trend_analysis_default() {
    let trend_analysis = TrendAnalysis::default();

    assert!(matches!(
        trend_analysis.mana_balance_trend,
        TrendDirection::Stable
    ));
    assert!(matches!(
        trend_analysis.transaction_volume_trend,
        TrendDirection::Stable
    ));
    assert!(matches!(
        trend_analysis.economic_health_trend,
        TrendDirection::Stable
    ));
    assert!(matches!(
        trend_analysis.cross_cooperative_trend,
        TrendDirection::Stable
    ));
    assert!(matches!(
        trend_analysis.network_performance_trend,
        TrendDirection::Stable
    ));
}

#[test]
fn test_trend_direction_enum() {
    // Test that all trend direction variants can be created
    let trends = [
        TrendDirection::StronglyImproving,
        TrendDirection::Improving,
        TrendDirection::Stable,
        TrendDirection::Declining,
        TrendDirection::StronglyDeclining,
    ];

    assert_eq!(trends.len(), 5);

    // Test that they can be matched
    for trend in trends {
        match trend {
            TrendDirection::StronglyImproving => {}
            TrendDirection::Improving => {}
            TrendDirection::Stable => {}
            TrendDirection::Declining => {}
            TrendDirection::StronglyDeclining => {}
        }
    }
}

#[test]
fn test_system_status_enum() {
    // Test that all system status variants can be created
    let statuses = [
        SystemStatus::Healthy,
        SystemStatus::Warning,
        SystemStatus::Critical,
        SystemStatus::Emergency,
    ];

    assert_eq!(statuses.len(), 4);

    // Test that they can be matched
    for status in statuses {
        match status {
            SystemStatus::Healthy => {}
            SystemStatus::Warning => {}
            SystemStatus::Critical => {}
            SystemStatus::Emergency => {}
        }
    }
}
