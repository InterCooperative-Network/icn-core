// icn-ccl/src/bin/ccl_lsp.rs
//! CCL Language Server Protocol implementation
//! 
//! This binary provides LSP support for CCL (Cooperative Contract Language) files,
//! enabling features like autocompletion, go-to-definition, diagnostics, and hover
//! information in IDEs that support LSP.

use icn_ccl::lsp::CclLanguageServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| CclLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}