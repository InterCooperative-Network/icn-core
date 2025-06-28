use icn_node::app_router_with_options;
use reqwest::Client;
use tokio::time::{sleep, Duration};
use rcgen::generate_simple_self_signed;
use axum_server::tls_rustls::{from_tcp_rustls, RustlsConfig};

#[tokio::test]
async fn authentication_enforced() {
    let (router, _ctx) = app_router_with_options(
        Some("key123".into()),
        Some("secret".into()),
        None,
        None,
        None,
        None,
        None,
    )
    .await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap();
    });

    sleep(Duration::from_millis(100)).await;
    let base = format!("http://{}", addr);
    let client = Client::new();

    let resp = client.get(format!("{}/info", base)).send().await.unwrap();
    assert_eq!(resp.status(), 401);

    let resp = client
        .get(format!("{}/info", base))
        .header("x-api-key", "key123")
        .header("Authorization", "Bearer secret")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    server.abort();
}

#[tokio::test]
async fn https_serving() {
    let (router, _ctx) = app_router_with_options(None, None, None, None, None, None, None).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let std_listener = listener.into_std().unwrap();

    let cert = generate_simple_self_signed(["localhost".into()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    let key_pem = cert.serialize_private_key_pem();
    let dir = tempfile::tempdir().unwrap();
    let cert_path = dir.path().join("cert.pem");
    let key_path = dir.path().join("key.pem");
    std::fs::write(&cert_path, cert_pem).unwrap();
    std::fs::write(&key_path, key_pem).unwrap();

    let cfg = RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .unwrap();

    let server = tokio::spawn(async move {
        from_tcp_rustls(std_listener, cfg)
            .serve(router.into_make_service())
            .await
            .unwrap();
    });

    sleep(Duration::from_millis(100)).await;
    let url = format!("https://localhost:{}", addr.port());
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = client.get(format!("{}/info", url)).send().await.unwrap();
    assert!(resp.status().is_success());

    server.abort();
}
