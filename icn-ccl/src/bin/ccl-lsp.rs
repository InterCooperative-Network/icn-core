// icn-ccl/src/bin/ccl-lsp.rs
//! CCL Language Server Protocol binary
//!
//! This binary starts the CCL Language Server for IDE integration.
//!
//! Usage:
//!   ccl-lsp [--stdio | --socket PORT]

use icn_ccl::lsp_server::start_lsp_server;
use std::env;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    // For now, only support stdio mode
    if args.len() > 1 && args[1] == "--help" {
        println!("CCL Language Server");
        println!("Usage: ccl-lsp [--stdio | --socket PORT]");
        println!();
        println!("Options:");
        println!("  --stdio     Use stdio for communication (default)");
        println!("  --socket    Use TCP socket for communication");
        println!("  --help      Show this help message");
        return;
    }

    // Start the LSP server
    eprintln!("Starting CCL Language Server...");
    start_lsp_server().await;
}