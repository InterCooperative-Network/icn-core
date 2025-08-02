//! ICN Runtime Configuration Builder Examples
//!
//! This example demonstrates advanced usage patterns for the ICN runtime configuration system,
//! including the builder pattern, templates, environment overrides, and complex configurations.

use icn_runtime::{templates, RuntimeConfigBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ICN Runtime Configuration Builder Examples");
    println!("=========================================\n");

    // Example 1: Basic Builder Usage
    println!("1. Basic Builder Usage:");
    let basic_config = RuntimeConfigBuilder::new()
        .environment_type("development")
        .debug(true)
        .log_level("debug")
        .node_did("did:key:z6MkExample123")
        .build_unchecked();

    println!(
        "   Environment: {}",
        basic_config.environment.environment_type
    );
    println!("   Debug mode: {}", basic_config.environment.debug);
    println!("   Log level: {}", basic_config.environment.log_level);
    println!("   Node DID: {}\n", basic_config.identity.node_did);

    // Example 2: Production Configuration
    println!("2. Production Configuration:");
    let production_config = RuntimeConfigBuilder::production()
        .node_did("did:key:z6MkProductionNode456")
        .listen_addresses(vec![
            "/ip4/0.0.0.0/tcp/4001".to_string(),
            "/ip6/::/tcp/4001".to_string(),
        ])
        .key_store_type("file")
        .key_file_path("~/.icn/keys/production.key")
        .dag_store_type("rocksdb")
        .dag_store_path("~/.icn/data/dag")
        .dag_cache_size_mb(512)
        .initial_mana(1000)
        .max_mana_capacity(10000)
        .governance_enabled(true)
        .build_unchecked();

    println!(
        "   Environment: {}",
        production_config.environment.environment_type
    );
    println!(
        "   Listen addresses: {:?}",
        production_config.network.listen_addresses
    );
    println!(
        "   DAG store: {}",
        production_config.storage.dag_store.store_type
    );
    println!(
        "   Initial mana: {}",
        production_config.storage.mana_ledger.initial_mana
    );
    println!(
        "   Governance enabled: {}\n",
        production_config.governance.enabled
    );

    // Example 3: Template Usage
    println!("3. Template-Based Configuration:");

    // Local development template
    let dev_config = RuntimeConfigBuilder::new()
        .apply_template(templates::local_development)
        .node_did("did:key:z6MkLocalDev789")
        .initial_mana(50000) // Override for faster development
        .build_unchecked();

    println!("   Local Development Template:");
    println!(
        "     Environment: {}",
        dev_config.environment.environment_type
    );
    println!(
        "     Listen addresses: {:?}",
        dev_config.network.listen_addresses
    );
    println!("     mDNS enabled: {}", dev_config.network.enable_mdns);
    println!(
        "     DAG store: {}",
        dev_config.storage.dag_store.store_type
    );

    // High performance template
    let perf_config = RuntimeConfigBuilder::new()
        .apply_template(templates::high_performance)
        .node_did("did:key:z6MkHighPerf999")
        .build_unchecked();

    println!("   High Performance Template:");
    println!(
        "     Max incoming connections: {}",
        perf_config
            .network
            .connection_limits
            .max_incoming_connections
    );
    println!(
        "     DAG cache size: {} MB",
        perf_config.storage.dag_store.cache_size_mb
    );
    println!(
        "     Max concurrent jobs: {}\n",
        perf_config.runtime.max_concurrent_jobs
    );

    // Example 4: Environment Overrides
    println!("4. Environment-Based Configuration:");
    let env_config = RuntimeConfigBuilder::new()
        .with_environment_overrides("production")
        .node_did("did:key:z6MkEnvOverride111")
        .dag_store_type("sled") // Override specific settings
        .connection_timeout_ms(15000)
        .build_unchecked();

    println!(
        "   Base environment: {}",
        env_config.environment.environment_type
    );
    println!("   Debug mode: {}", env_config.environment.debug);
    println!("   Log level: {}", env_config.environment.log_level);
    println!(
        "   Custom DAG store: {}",
        env_config.storage.dag_store.store_type
    );
    println!(
        "   Custom timeout: {} ms\n",
        env_config.network.timeouts.connection_timeout_ms
    );

    // Example 5: Configuration Merging
    println!("5. Configuration Merging:");
    let base_config = RuntimeConfigBuilder::development()
        .node_did("did:key:z6MkBase222")
        .initial_mana(10000)
        .build_unchecked();

    let override_config = RuntimeConfigBuilder::production()
        .dag_store_type("rocksdb")
        .governance_enabled(false) // Override governance settings
        .build_unchecked();

    let merged_config = RuntimeConfigBuilder::from_config(base_config)
        .merge_with(override_config)
        .build_unchecked();

    println!(
        "   Merged environment: {}",
        merged_config.environment.environment_type
    );
    println!(
        "   Merged DAG store: {}",
        merged_config.storage.dag_store.store_type
    );
    println!("   Merged governance: {}", merged_config.governance.enabled);
    println!(
        "   Original node DID: {}\n",
        merged_config.identity.node_did
    );

    // Example 6: Complex Fluent API Chain
    println!("6. Complex Fluent Configuration:");
    let complex_config = RuntimeConfigBuilder::testing()
        .node_did("did:key:z6MkComplex333")
        // Network configuration
        .listen_addresses(vec![
            "/ip4/127.0.0.1/tcp/4001".to_string(),
            "/ip4/127.0.0.1/tcp/4002".to_string(),
        ])
        .add_bootstrap_peer("12D3KooWPeer1", "/ip4/10.0.0.1/tcp/4001")
        .add_bootstrap_peer("12D3KooWPeer2", "/ip4/10.0.0.2/tcp/4001")
        .enable_mdns(false)
        .connection_timeout_ms(2000)
        .request_timeout_ms(5000)
        .max_incoming_connections(25)
        .max_outgoing_connections(25)
        // Storage configuration
        .data_dir("/tmp/complex-icn-test")
        .dag_store_type("memory")
        .dag_cache_size_mb(128)
        .mana_ledger_path("/tmp/complex-mana.db")
        .initial_mana(100000)
        .max_mana_capacity(500000)
        .mana_regeneration_rate(15.0)
        // Governance configuration
        .governance_enabled(true)
        .min_voting_power(10)
        .vote_cost_mana(5)
        .voting_period_seconds(300) // 5 minutes for testing
        // Runtime configuration
        .job_execution_timeout_ms(10000)
        .max_job_queue_size(100)
        .max_concurrent_jobs(5)
        .build_unchecked();

    println!("   Node DID: {}", complex_config.identity.node_did);
    println!(
        "   Bootstrap peers: {}",
        complex_config.network.bootstrap_peers.len()
    );
    println!("   Data directory: {:?}", complex_config.storage.data_dir);
    println!(
        "   Mana regeneration: {}/sec",
        complex_config.storage.mana_ledger.regeneration_rate
    );
    println!(
        "   Voting period: {}s",
        complex_config.governance.voting.voting_period_seconds
    );
    println!(
        "   Job timeout: {}ms\n",
        complex_config.runtime.job_execution_timeout_ms
    );

    // Example 7: Configuration Validation
    println!("7. Configuration Validation:");

    // Valid configuration
    let valid_config = RuntimeConfigBuilder::production()
        .node_did("did:key:z6MkValid444")
        .initial_mana(5000)
        .max_mana_capacity(50000);

    match valid_config.validate() {
        Ok(_) => println!("   Valid configuration: ✓"),
        Err(e) => println!("   Valid configuration failed: {e}"),
    }

    // Invalid configuration (initial mana > max capacity)
    let invalid_config = RuntimeConfigBuilder::production()
        .node_did("did:key:z6MkInvalid555")
        .initial_mana(100000)
        .max_mana_capacity(50000);

    match invalid_config.validate() {
        Ok(_) => println!("   Invalid configuration: ✗ (should have failed)"),
        Err(e) => println!("   Invalid configuration correctly rejected: {e}"),
    }

    // Example 8: File-based Configuration Integration
    println!("\n8. File-based Configuration:");
    println!("   (File operations would require proper paths and permissions)");
    println!("   Example usage:");
    println!("   let config = RuntimeConfig::from_file(\"production.toml\")?;");
    println!("   let modified = RuntimeConfigBuilder::from_config(config)");
    println!("       .initial_mana(2000)");
    println!("       .build()?;");
    println!("   modified.to_file(\"custom.toml\")?;");

    println!("\nConfiguration builder examples completed successfully!");
    println!("See the ICN documentation for more advanced usage patterns.");

    Ok(())
}
