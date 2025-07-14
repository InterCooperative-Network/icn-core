use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn monitor_uptime_reports_value() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    let output = tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "monitor", "uptime"])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(output.status.success());
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Uptime"));

    server.abort();
}
