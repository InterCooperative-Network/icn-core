use icn_ccl_packages::{Package, PackageManager, PackageCategory, PackageRegistry};
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return Ok(());
    }
    
    let mut package_manager = PackageManager::new(PathBuf::from("./packages"));
    
    match args[1].as_str() {
        "init" => {
            if args.len() < 3 {
                println!("❌ Usage: ccl-pkg init <package-name>");
                return Ok(());
            }
            
            let name = args[2].clone();
            let category = if args.len() > 3 {
                match args[3].as_str() {
                    "governance" => PackageCategory::Governance,
                    "economics" => PackageCategory::Economics,
                    "identity" => PackageCategory::Identity,
                    "utility" => PackageCategory::Utility,
                    _ => PackageCategory::Template,
                }
            } else {
                PackageCategory::Template
            };
            
            package_manager.init_package(
                name,
                "Unknown Author".to_string(),
                "A new CCL package".to_string(),
                category,
            )?;
        }
        
        "install" => {
            if args.len() < 3 {
                println!("❌ Usage: ccl-pkg install <package-name> [version]");
                return Ok(());
            }
            
            let package_name = &args[2];
            let version = args.get(3).map(|s| s.as_str());
            
            package_manager.install_package(package_name, version).await?;
        }
        
        "add" => {
            if args.len() < 4 {
                println!("❌ Usage: ccl-pkg add <package-name> <version>");
                return Ok(());
            }
            
            let name = args[2].clone();
            let version = args[3].clone();
            
            package_manager.add_dependency(name, version)?;
        }
        
        "list" => {
            let packages = package_manager.list_packages()?;
            if packages.is_empty() {
                println!("No packages installed");
            } else {
                println!("📦 Installed packages:");
                for package in packages {
                    println!("  {} v{} - {}", package.name, package.version, package.description);
                }
            }
        }
        
        "build" => {
            package_manager.build_package()?;
        }
        
        "search" => {
            if args.len() < 3 {
                println!("❌ Usage: ccl-pkg search <query>");
                return Ok(());
            }
            
            let query = &args[2];
            let mut registry = PackageRegistry::new();
            
            if let Ok(Some(entry)) = registry.search_package(query).await {
                println!("📦 Found package:");
                println!("  Name: {}", entry.name);
                println!("  Version: {}", entry.version);
                println!("  Description: {}", entry.description);
                println!("  Author: {}", entry.author);
            } else {
                println!("❌ Package '{}' not found", query);
            }
        }
        
        "registry" => {
            let mut registry = PackageRegistry::new();
            let packages = registry.list_packages().await?;
            
            println!("📚 Available packages in registry:");
            for package in packages {
                println!("  {} v{} - {}", package.name, package.version, package.description);
                println!("    by {} ({})", package.author, package.published_at);
            }
        }
        
        "help" | "--help" | "-h" => {
            print_help();
        }
        
        _ => {
            println!("❓ Unknown command: {}", args[1]);
            print_help();
        }
    }
    
    Ok(())
}

fn print_help() {
    println!(r#"
📦 ICN CCL Package Manager

Usage: ccl-pkg <command> [arguments]

Commands:
  init <name> [category]     Initialize a new CCL package
                            Categories: governance, economics, identity, utility, template
  
  install <name> [version]   Install a package from the registry
  add <name> <version>       Add a dependency to current package
  list                       List installed packages
  build                      Build the current package
  search <query>             Search for packages in the registry
  registry                   List all available packages
  help                       Show this help message

Examples:
  ccl-pkg init my-governance governance
  ccl-pkg install liquid-democracy
  ccl-pkg add quadratic-voting 0.8.0
  ccl-pkg build
  ccl-pkg search governance

Package Categories:
  🏛️  governance  - Governance and voting mechanisms
  💰 economics   - Economic and financial primitives  
  🆔 identity    - Identity and access management
  🔧 utility     - General utility functions
  📄 template    - Basic contract templates
"#);
}