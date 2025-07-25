// icn-ccl/src/bin/ccl_cli.rs
//! CCL command-line interface for package management and debugging

use clap::{Parser, Subcommand};
use icn_ccl::{
    cli_commands,
    compile_ccl_file_to_wasm,
    error::CclError,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ccl-cli")]
#[command(about = "CCL Developer Tools - Package manager and debugger for Cooperative Contract Language")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a CCL contract
    Compile {
        /// CCL source file to compile
        file: PathBuf,
        /// Output directory for compiled files
        #[arg(short, long, default_value = "./target/ccl")]
        output: PathBuf,
        /// Generate debug information
        #[arg(long)]
        debug: bool,
    },
    /// Package management commands
    Package {
        #[command(subcommand)]
        command: PackageCommands,
    },
    /// Debugging commands
    Debug {
        /// Debug info file or CCL source file
        file: PathBuf,
        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },
    /// Format CCL source files
    Format {
        /// CCL files to format
        files: Vec<PathBuf>,
    },
    /// Lint CCL source files
    Lint {
        /// CCL files to lint
        files: Vec<PathBuf>,
    },
}

#[derive(Subcommand)]
enum PackageCommands {
    /// Initialize a new CCL package
    Init {
        /// Package name
        name: String,
        /// Package directory (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        directory: PathBuf,
        /// Author name
        #[arg(long)]
        author: String,
        /// Author email
        #[arg(long)]
        email: Option<String>,
    },
    /// Install package dependencies
    Install {
        /// Package directory (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        directory: PathBuf,
    },
    /// Add a dependency
    Add {
        /// Dependency name
        name: String,
        /// Version requirement
        version: String,
        /// Add as development dependency
        #[arg(long)]
        dev: bool,
        /// Package directory (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        directory: PathBuf,
    },
}

fn main() {
    env_logger::init();
    
    let cli = Cli::parse();

    if let Err(e) = run_command(cli.command) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_command(command: Commands) -> Result<(), CclError> {
    match command {
        Commands::Compile { file, output, debug } => {
            compile_command(file, output, debug)
        }
        Commands::Package { command } => {
            match command {
                PackageCommands::Init { name, directory, author, email } => {
                    cli_commands::init_package(name, &directory, author, email)
                }
                PackageCommands::Install { directory } => {
                    cli_commands::install_dependencies(&directory)
                }
                PackageCommands::Add { name, version, dev, directory } => {
                    cli_commands::add_dependency(&directory, name, version, dev)
                }
            }
        }
        Commands::Debug { file, interactive } => {
            debug_command(file, interactive)
        }
        Commands::Format { files } => {
            format_command(files)
        }
        Commands::Lint { files } => {
            lint_command(files)
        }
    }
}

fn compile_command(file: PathBuf, output: PathBuf, debug: bool) -> Result<(), CclError> {
    println!("🔨 Compiling CCL contract: {}", file.display());
    
    // Create output directory
    std::fs::create_dir_all(&output).map_err(|e| {
        CclError::IoError(format!("Failed to create output directory: {}", e))
    })?;

    // Compile the contract
    let (wasm_bytes, metadata) = compile_ccl_file_to_wasm(&file)?;

    // Write WASM output
    let wasm_path = output.join(format!("{}.wasm", 
        file.file_stem().unwrap().to_string_lossy()));
    std::fs::write(&wasm_path, wasm_bytes).map_err(|e| {
        CclError::IoError(format!("Failed to write WASM file: {}", e))
    })?;

    // Write metadata
    let meta_path = output.join(format!("{}.json", 
        file.file_stem().unwrap().to_string_lossy()));
    let meta_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
        CclError::IoError(format!("Failed to serialize metadata: {}", e))
    })?;
    std::fs::write(&meta_path, meta_json).map_err(|e| {
        CclError::IoError(format!("Failed to write metadata file: {}", e))
    })?;

    println!("✅ Compiled successfully:");
    println!("   WASM: {}", wasm_path.display());
    println!("   Meta: {}", meta_path.display());

    // Generate debug info if requested
    if debug {
        let debug_path = output.join(format!("{}.debug.json", 
            file.file_stem().unwrap().to_string_lossy()));
        cli_commands::generate_debug_info(&file, &wasm_path, &debug_path)?;
    }

    Ok(())
}

fn debug_command(file: PathBuf, interactive: bool) -> Result<(), CclError> {
    if interactive {
        cli_commands::start_debug_session(&file)
    } else {
        // Non-interactive debugging - just validate the debug file
        if file.extension().and_then(|s| s.to_str()) == Some("json") {
            println!("🔍 Debug file: {}", file.display());
            // TODO: Show debug file information
            Ok(())
        } else {
            return Err(CclError::IoError(
                "Debug command requires a .debug.json file or use --interactive flag".to_string()
            ));
        }
    }
}

fn format_command(files: Vec<PathBuf>) -> Result<(), CclError> {
    if files.is_empty() {
        return Err(CclError::IoError("No files specified for formatting".to_string()));
    }

    for file in files {
        println!("📝 Formatting: {}", file.display());
        // TODO: Implement CCL code formatting
        println!("   (formatting not yet implemented)");
    }

    Ok(())
}

fn lint_command(files: Vec<PathBuf>) -> Result<(), CclError> {
    if files.is_empty() {
        return Err(CclError::IoError("No files specified for linting".to_string()));
    }

    for file in files {
        println!("🔍 Linting: {}", file.display());
        // TODO: Implement CCL linting
        println!("   (linting not yet implemented)");
    }

    Ok(())
}