use assert_cmd::prelude::*;
use base64::Engine;
use icn_node::app_router;
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::task;

/// Test response time for basic info command
#[tokio::test]
#[serial_test::serial]
async fn test_info_command_performance() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Measure response time for info command
    let start = Instant::now();

    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "info"])
            .assert()
            .success();
    })
    .await
    .unwrap();

    let duration = start.elapsed();

    // Info command should complete within 5 seconds
    assert!(
        duration < Duration::from_secs(5),
        "Info command took too long: {duration:?}"
    );

    println!("Info command completed in: {duration:?}");

    server.abort();
}

/// Test concurrent command execution
#[tokio::test]
#[serial_test::serial]
async fn test_concurrent_commands() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Test concurrent execution of multiple commands
    let num_concurrent = 10;
    let mut handles = Vec::new();

    let start = Instant::now();

    for i in 0..num_concurrent {
        let bin = bin.to_string(); // Convert to owned String
        let base = base.to_string();

        let handle = tokio::task::spawn_blocking(move || {
            let cmd_start = Instant::now();
            let output = Command::new(bin)
                .args(["--api-url", &base, "info"])
                .output()
                .unwrap();

            (i, output.status.success(), cmd_start.elapsed())
        });

        handles.push(handle);
    }

    // Wait for all commands to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    let total_duration = start.elapsed();

    // All commands should succeed
    for (i, success, duration) in &results {
        assert!(*success, "Command {i} failed");
        assert!(
            *duration < Duration::from_secs(10),
            "Command {i} took too long: {duration:?}"
        );
    }

    // Total time should be reasonable (concurrent execution)
    assert!(
        total_duration < Duration::from_secs(30),
        "Concurrent commands took too long: {total_duration:?}"
    );

    println!("Concurrent commands completed in: {total_duration:?}");

    server.abort();
}

/// Test memory usage with large JSON payloads
#[tokio::test]
#[serial_test::serial]
async fn test_large_payload_handling() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Create a moderately large job payload
    let large_payload = "x".repeat(100000); // 100KB payload
    let job_request = serde_json::json!({
        "manifest_cid": "bafytest",
        "spec_bytes": base64::engine::general_purpose::STANDARD.encode(large_payload.as_bytes()),
        "spec_json": null,
        "cost_mana": 10
    })
    .to_string();

    let start = Instant::now();

    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "mesh", "submit", &job_request])
            .assert()
            .success();
    })
    .await
    .unwrap();

    let duration = start.elapsed();

    // Large payload should still complete within reasonable time
    assert!(
        duration < Duration::from_secs(30),
        "Large payload command took too long: {duration:?}"
    );

    println!("Large payload command completed in: {duration:?}");

    server.abort();
}

/// Test CLI startup time
#[tokio::test]
#[serial_test::serial]
async fn test_cli_startup_time() {
    let bin = env!("CARGO_BIN_EXE_icn-cli");

    let start = Instant::now();

    tokio::task::spawn_blocking(move || {
        Command::new(bin).args(["--help"]).assert().success();
    })
    .await
    .unwrap();

    let duration = start.elapsed();

    // CLI should start up quickly
    assert!(
        duration < Duration::from_secs(2),
        "CLI startup took too long: {duration:?}"
    );

    println!("CLI startup completed in: {duration:?}");
}

/// Test command parsing performance
#[tokio::test]
#[serial_test::serial]
async fn test_command_parsing_performance() {
    let bin = env!("CARGO_BIN_EXE_icn-cli");

    // Test various command combinations
    let commands = vec![
        vec!["--help"],
        vec!["info", "--help"],
        vec!["dag", "--help"],
        vec!["mesh", "--help"],
        vec!["governance", "--help"],
        vec!["network", "--help"],
        vec!["identity", "--help"],
        vec!["ccl", "--help"],
        vec!["zk", "--help"],
        vec!["federation", "--help"],
    ];

    let start = Instant::now();

    for cmd_args in commands {
        let bin = bin.to_string(); // Convert to owned String

        tokio::task::spawn_blocking(move || {
            Command::new(bin).args(cmd_args).assert().success();
        })
        .await
        .unwrap();
    }

    let duration = start.elapsed();

    // All help commands should complete quickly
    assert!(
        duration < Duration::from_secs(10),
        "Command parsing took too long: {duration:?}"
    );

    println!("Command parsing completed in: {duration:?}");
}

/// Test CLI with repeated commands (stress test)
#[tokio::test]
#[serial_test::serial]
async fn test_repeated_commands_stress() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Execute the same command multiple times
    let num_iterations = 20;
    let mut durations = Vec::new();

    for i in 0..num_iterations {
        let bin = bin.to_string(); // Convert to owned String
        let base_clone = base.clone();

        let start = Instant::now();

        tokio::task::spawn_blocking(move || {
            Command::new(bin)
                .args(["--api-url", &base_clone, "info"])
                .assert()
                .success();
        })
        .await
        .unwrap();

        let duration = start.elapsed();
        durations.push(duration);

        // Each iteration should complete reasonably quickly
        assert!(
            duration < Duration::from_secs(10),
            "Iteration {i} took too long: {duration:?}"
        );
    }

    // Calculate average duration
    let avg_duration = durations.iter().sum::<Duration>() / num_iterations as u32;

    println!("Average command duration over {num_iterations} iterations: {avg_duration:?}");

    // Average should be reasonable
    assert!(
        avg_duration < Duration::from_secs(5),
        "Average command duration too high: {avg_duration:?}"
    );

    server.abort();
}

/// Test CLI memory usage patterns
#[tokio::test]
#[serial_test::serial]
async fn test_memory_usage_patterns() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Test different command types that might use different amounts of memory
    let commands = vec![
        vec!["--api-url".to_string(), base.clone(), "info".to_string()],
        vec!["--api-url".to_string(), base.clone(), "status".to_string()],
        vec!["--api-url".to_string(), base.clone(), "metrics".to_string()],
        vec![
            "--api-url".to_string(),
            base.clone(),
            "network".to_string(),
            "stats".to_string(),
        ],
        vec![
            "--api-url".to_string(),
            base.clone(),
            "mesh".to_string(),
            "jobs".to_string(),
        ],
        vec![
            "--api-url".to_string(),
            base.clone(),
            "governance".to_string(),
            "proposals".to_string(),
        ],
    ];

    for cmd_args in commands {
        let bin = bin.to_string(); // Convert to owned String

        let start = Instant::now();

        tokio::task::spawn_blocking(move || {
            Command::new(bin).args(cmd_args).assert().success();
        })
        .await
        .unwrap();

        let duration = start.elapsed();

        // Each command should complete within reasonable time
        assert!(
            duration < Duration::from_secs(15),
            "Command took too long: {duration:?}"
        );
    }

    server.abort();
}

/// Test CLI error handling performance
#[tokio::test]
#[serial_test::serial]
async fn test_error_handling_performance() {
    let bin = env!("CARGO_BIN_EXE_icn-cli");

    // Test error cases to ensure they fail quickly
    let error_cases = vec![
        vec!["--api-url", "http://127.0.0.1:9999", "info"], // Unreachable server
        vec!["--api-url", "invalid-url", "info"],           // Invalid URL
        vec!["invalid-command"],                            // Invalid command
        vec!["mesh", "submit"],                             // Missing required argument
    ];

    for cmd_args in error_cases {
        let bin = bin.to_string(); // Convert to owned String

        let start = Instant::now();

        tokio::task::spawn_blocking(move || {
            Command::new(bin).args(cmd_args).assert().failure();
        })
        .await
        .unwrap();

        let duration = start.elapsed();

        // Error cases should fail quickly (not hang)
        assert!(
            duration < Duration::from_secs(30),
            "Error case took too long: {duration:?}"
        );
    }

    println!("Error handling performance test completed");
}
