//! Generate TypeScript Client SDK
//!
//! This binary generates TypeScript client SDK files from the ICN API definitions.
//! Usage: cargo run --bin generate-ts-sdk [output-directory]

use icn_api::client_sdk::{sdk_files, TypeScriptGenerator};
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let output_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("./client-sdk")
    };

    println!("Generating TypeScript SDK files in: {:?}", output_dir);

    // Generate all SDK files
    sdk_files::generate_sdk_files(&output_dir)?;

    println!("✅ TypeScript SDK generated successfully!");
    println!("📁 Files created in: {:?}", output_dir);
    println!();
    println!("To use the generated SDK:");
    println!("1. cd {}", output_dir.display());
    println!("2. npm install");
    println!("3. npm run build");
    println!();
    println!("The compiled SDK will be in the 'dist' folder.");

    Ok(())
} 