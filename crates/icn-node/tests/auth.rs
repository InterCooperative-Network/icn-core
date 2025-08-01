use base64::Engine;
use icn_node::app_router_with_options;
use rcgen::generate_simple_self_signed;
use reqwest::Client;
use tempfile::NamedTempFile;
use tokio::task;

#[tokio::test]
async fn api_key_required_for_requests() {
    let (router, _ctx) = app_router_with_options(
        icn_node::RuntimeMode::Development,
        Some("secret".into()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = Client::new();
    let url = format!("http://{addr}/info");

    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    let resp = client
        .get(&url)
        .header("x-api-key", "wrong")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    let resp = client
        .get(&url)
        .header("x-api-key", "secret")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    server.abort();
}

#[tokio::test]
async fn bearer_token_required_for_requests() {
    let (router, _ctx) = app_router_with_options(
        icn_node::RuntimeMode::Development,
        None,
        Some("s3cr3t".into()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = Client::new();
    let url = format!("http://{addr}/info");

    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    let resp = client
        .get(&url)
        .header("Authorization", "Bearer wrong")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    let resp = client
        .get(&url)
        .header("Authorization", "Bearer s3cr3t")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    server.abort();
}

#[tokio::test]
async fn tls_api_key_and_bearer_token() {
    let cert = generate_simple_self_signed(["localhost".to_string()]).unwrap();
    let cert_pem = cert.cert.serialize_pem();
    let key_pem = cert.signing_key.serialize_pem();

    let cert_file = NamedTempFile::new().unwrap();
    let key_file = NamedTempFile::new().unwrap();
    std::fs::write(cert_file.path(), cert_pem).unwrap();
    std::fs::write(key_file.path(), key_pem).unwrap();

    let (router, _ctx) = app_router_with_options(
        icn_node::RuntimeMode::Development,
        Some("secret".into()),
        Some("token".into()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await;

    let std_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = std_listener.local_addr().unwrap();
    let tls_config =
        axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_file.path(), key_file.path())
            .await
            .unwrap();
    let server = task::spawn(async move {
        axum_server::tls_rustls::from_tcp_rustls(std_listener, tls_config)
            .serve(router.into_make_service())
            .await
            .unwrap();
    });

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let url = format!("https://{addr}/info");

    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    let resp = client
        .get(&url)
        .header("x-api-key", "secret")
        .header("Authorization", "Bearer wrong")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    let resp = client
        .get(&url)
        .header("x-api-key", "secret")
        .header("Authorization", "Bearer token")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    server.abort();
}
