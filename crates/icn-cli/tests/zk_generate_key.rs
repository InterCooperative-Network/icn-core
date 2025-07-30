use serde_json::Value;
use std::process::Command;

#[tokio::test]
#[serial_test::serial]
async fn zk_generate_key_outputs_paths() {
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let output = tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["zk", "generate-key"])
            .output()
            .unwrap()
    })
    .await
    .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let v: Value = serde_json::from_str(&stdout).unwrap();
    let path = v["proving_key_path"].as_str().unwrap();
    assert!(std::path::Path::new(path).exists());
    assert!(!v["verifying_key_signature_hex"].as_str().unwrap().is_empty());
}
