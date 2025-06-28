use clap::{CommandFactory, FromArgMatches};
use icn_node::config::{NodeConfig, StorageBackendType};
use icn_node::node::Cli;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn merge_file_env_cli() {
    // create a temp config file with nested sections
    let file = NamedTempFile::new().unwrap();
    fs::write(
        &file,
        r#"[storage]
backend = "file"
path = "file_path"
[http]
listen_addr = "1.2.3.4:1111"
"#,
    )
    .unwrap();

    // set env vars overriding some values
    std::env::set_var("ICN_STORAGE_BACKEND", "sqlite");
    std::env::set_var("ICN_HTTP_LISTEN_ADDR", "5.6.7.8:2222");

    // CLI overrides storage_path
    let args = [
        "icn-node",
        "--storage-path",
        "cli_path",
        "--config",
        file.path().to_str().unwrap(),
    ];
    let cmd = Cli::command();
    let matches = cmd.get_matches_from(args);
    let cli = Cli::from_arg_matches(&matches).unwrap();

    let mut cfg = NodeConfig::from_file(file.path()).unwrap();
    cfg.apply_env_overrides();
    cfg.apply_cli_overrides(&cli, &matches);

    assert_eq!(cfg.storage_backend, StorageBackendType::Sqlite);
    assert_eq!(cfg.storage_path.to_str().unwrap(), "cli_path");
    assert_eq!(cfg.http_listen_addr, "5.6.7.8:2222");

    // cleanup
    std::env::remove_var("ICN_STORAGE_BACKEND");
    std::env::remove_var("ICN_HTTP_LISTEN_ADDR");
}
