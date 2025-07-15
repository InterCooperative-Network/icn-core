use assert_cmd::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

#[tokio::test]
#[serial_test::serial]
async fn cooperative_wizard_creates_file() {
    let dir = tempdir().unwrap();
    let out = dir.path().to_path_buf();
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let path_str = out.to_str().unwrap().to_string();
    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new(bin);
        cmd.args(["wizard", "cooperative", "--output", &path_str])
            .stdin(Stdio::piped());
        let mut child = cmd.spawn().unwrap();
        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"TestCoop\n1\n").unwrap();
        }
        child.wait_with_output().unwrap()
    })
    .await
    .unwrap();
    assert!(output.status.success());
    assert!(out.join("TestCoop_governance.ccl").exists());
}
