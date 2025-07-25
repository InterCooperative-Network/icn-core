use icn_ccl_lsp::start_lsp_server;

#[tokio::main]
async fn main() {
    env_logger::init();
    start_lsp_server().await;
}