use clap::{CommandFactory, FromArgMatches};
use icn_node::config::NodeConfig;
use icn_node::node::{build_network_service, Cli};

#[tokio::test]
async fn enable_p2p_uses_libp2p() {
    let args = ["icn-node", "--enable-p2p"];
    let cmd = Cli::command();
    let matches = cmd.get_matches_from(args);
    let cli = Cli::from_arg_matches(&matches).unwrap();
    let mut cfg = NodeConfig::default();
    cfg.apply_cli_overrides(&cli, &matches);

    let svc = build_network_service(&cfg).await.unwrap();
    #[cfg(feature = "enable-libp2p")]
    {
        use icn_network::NetworkService;
        assert!(NetworkService::as_any(&*svc)
            .is::<icn_network::libp2p_service::Libp2pNetworkService>());
    }
}

#[tokio::test]
async fn test_mode_forces_stub() {
    let args = ["icn-node", "--enable-p2p", "--test-mode"];
    let cmd = Cli::command();
    let matches = cmd.get_matches_from(args);
    let cli = Cli::from_arg_matches(&matches).unwrap();
    let mut cfg = NodeConfig::default();
    cfg.apply_cli_overrides(&cli, &matches);

    let svc = build_network_service(&cfg).await.unwrap();
    {
        use icn_network::NetworkService;
        assert!(NetworkService::as_any(&*svc).is::<icn_network::StubNetworkService>());
    }
}
