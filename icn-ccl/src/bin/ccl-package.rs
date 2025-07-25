// icn-ccl/src/bin/ccl-package.rs
//! CCL Package Manager binary
//!
//! This binary provides package management for CCL contracts and governance patterns.

use icn_ccl::{create_test_package_manager, CclPackageManager, GovernancePatternType};
use icn_common::Did;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    // Get cache directory
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ccl-packages");

    let mut manager = create_test_package_manager(&cache_dir)?;

    match args[1].as_str() {
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: ccl-package search <query>");
                std::process::exit(1);
            }
            let query = &args[2];
            let results = manager.search(query)?;

            println!("Search results for '{}':", query);
            for package in results {
                println!("  {} v{} - {}", package.name, package.version, package.description);
            }
        }
        "install" => {
            if args.len() < 3 {
                eprintln!("Usage: ccl-package install <package>[@version]");
                std::process::exit(1);
            }

            let package_spec = &args[2];
            let (name, version) = if package_spec.contains('@') {
                let parts: Vec<&str> = package_spec.split('@').collect();
                (parts[0], parts[1])
            } else {
                (package_spec.as_str(), "latest")
            };

            manager.install(name, version)?;
            println!("Installed {} v{}", name, version);
        }
        "list" => {
            let installed = manager.list_installed();
            println!("Installed packages:");
            for package in installed {
                println!("  {} v{} - {}", package.name, package.version, package.description);
            }
        }
        "patterns" => {
            let patterns = manager.list_governance_patterns();
            println!("Available governance patterns:");
            for pattern in patterns {
                println!("  {} ({:?}) - {}", pattern.name, pattern.pattern_type, pattern.description);
            }
        }
        "generate" => {
            if args.len() < 3 {
                eprintln!("Usage: ccl-package generate <pattern_name>");
                std::process::exit(1);
            }

            let pattern_name = &args[2];
            let pattern = manager.get_governance_pattern(pattern_name);

            if let Some(pattern) = pattern {
                println!("Generating code from pattern: {}", pattern.name);
                println!("Description: {}", pattern.description);
                println!();

                // Get parameters from user
                let mut parameters = HashMap::new();
                for (param_name, param_spec) in &pattern.parameters {
                    print!("{} ({}): ", param_name, param_spec.description);
                    if let Some(default) = &param_spec.default_value {
                        print!("[{}] ", default);
                    }
                    io::stdout().flush()?;

                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let input = input.trim();

                    let value = if input.is_empty() {
                        param_spec.default_value.clone().unwrap_or_default()
                    } else {
                        input.to_string()
                    };

                    parameters.insert(param_name.clone(), value);
                }

                let generated_code = manager.generate_from_pattern(pattern_name, &parameters)?;
                println!("\nGenerated code:");
                println!("{}", generated_code);
            } else {
                eprintln!("Pattern '{}' not found", pattern_name);
                std::process::exit(1);
            }
        }
        "create" => {
            if args.len() < 4 {
                eprintln!("Usage: ccl-package create <name> <version> <source_file>");
                std::process::exit(1);
            }

            let name = args[2].clone();
            let version = args[3].clone();
            let source_file = &args[4];

            let source_code = std::fs::read_to_string(source_file)?;
            let author = Did::from_str("did:example:packager")?;

            let package = manager.create_package(
                name.clone(),
                version.clone(),
                "Generated package".to_string(),
                author,
                source_code,
                Vec::new(),
            )?;

            println!("Created package {} v{}", name, version);
            println!("Package info: {:?}", package.info);
        }
        "help" => {
            print_help();
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_help() {
    println!("CCL Package Manager");
    println!("Usage: ccl-package <command> [args]");
    println!();
    println!("Commands:");
    println!("  search <query>           Search for packages");
    println!("  install <pkg>[@version]  Install a package");
    println!("  list                     List installed packages");
    println!("  patterns                 List governance patterns");
    println!("  generate <pattern>       Generate code from pattern");
    println!("  create <name> <ver> <src> Create package from source");
    println!("  help                     Show this help");
}