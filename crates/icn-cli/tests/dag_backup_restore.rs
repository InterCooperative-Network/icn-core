use assert_cmd::prelude::*;
use icn_common::{compute_merkle_cid, DagBlock, Did};
use icn_dag::{FileDagStore, StorageService};
use predicates::str::contains;
use std::process::Command;
use tempfile::tempdir;

fn create_block() -> DagBlock {
    let data = b"data".to_vec();
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig = None;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &author, &sig, &None);
    DagBlock {
        cid,
        data,
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig,
        scope: None,
    }
}

#[tokio::test]
#[serial_test::serial]
async fn dag_backup_restore_verify_commands() {
    let dir = tempdir().unwrap();
    let workdir = dir.path();
    let store_path = workdir.join("icn_data/node_store");
    std::fs::create_dir_all(&store_path).unwrap();
    let mut store = FileDagStore::new(store_path.clone()).unwrap();
    let block = create_block();
    let cid = block.cid.clone();
    store.put(&block).unwrap();

    let backup_dir = workdir.join("backup");
    let backup_str = backup_dir.to_str().unwrap().to_string();
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let wd = workdir.to_path_buf();
    let backup_clone = backup_str.clone();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .current_dir(&wd)
            .args(["dag", "backup", &backup_clone])
            .assert()
            .success()
            .stdout(contains("Backup"));
    })
    .await
    .unwrap();

    assert!(backup_dir.join(cid.to_string()).exists());

    std::fs::remove_dir_all(&store_path).unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let wd = workdir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .current_dir(&wd)
            .args(["dag", "restore", &backup_str])
            .assert()
            .success()
            .stdout(contains("Restored"));
    })
    .await
    .unwrap();

    let store2 = FileDagStore::new(store_path.clone()).unwrap();
    assert!(store2.get(&cid).unwrap().is_some());

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let wd = workdir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .current_dir(&wd)
            .args(["dag", "verify", "--full"])
            .assert()
            .success()
            .stdout(contains("Verified"));
    })
    .await
    .unwrap();
}
