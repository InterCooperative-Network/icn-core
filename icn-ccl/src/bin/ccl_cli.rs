// icn-ccl/src/bin/ccl_cli.rs
//! CCL command-line interface for package management and debugging

use clap::{Parser, Subcommand};
use icn_ccl::{cli_commands, compile_ccl_file_to_wasm, error::CclError};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ccl-cli")]
#[command(
    about = "CCL Developer Tools - Package manager and debugger for Cooperative Contract Language"
)]
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
    /// Migration commands
    Migrate {
        #[command(subcommand)]
        command: MigrationCommands,
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

#[derive(Subcommand)]
enum MigrationCommands {
    /// Migrate a CCL contract to a newer version
    Upgrade {
        /// Input CCL file
        input: PathBuf,
        /// Output file (defaults to input file with .migrated.ccl extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Target CCL version (defaults to latest)
        #[arg(short, long)]
        target_version: Option<String>,
        /// Show migration report without applying changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Convert from other contract languages to CCL
    Convert {
        /// Input file (Solidity, JavaScript, etc.)
        input: PathBuf,
        /// Output CCL file
        #[arg(short, long)]
        output: PathBuf,
        /// Source language (solidity, javascript, typescript)
        #[arg(short, long)]
        source_language: String,
    },
    /// Detect the CCL version of a contract
    Detect {
        /// CCL file to analyze
        file: PathBuf,
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
        Commands::Compile {
            file,
            output,
            debug,
        } => compile_command(file, output, debug),
        Commands::Package { command } => match command {
            PackageCommands::Init {
                name,
                directory,
                author,
                email,
            } => cli_commands::init_package(name, &directory, author, email),
            PackageCommands::Install { directory } => {
                cli_commands::install_dependencies(&directory)
            }
            PackageCommands::Add {
                name,
                version,
                dev,
                directory,
            } => cli_commands::add_dependency(&directory, name, version, dev),
        },
        Commands::Debug { file, interactive } => debug_command(file, interactive),
        Commands::Format { files } => format_command(files),
        Commands::Lint { files } => lint_command(files),
        Commands::Migrate { command } => match command {
            MigrationCommands::Upgrade {
                input,
                output,
                target_version,
                dry_run,
            } => migration_upgrade_command(input, output, target_version, dry_run),
            MigrationCommands::Convert {
                input,
                output,
                source_language,
            } => migration_convert_command(input, output, source_language),
            MigrationCommands::Detect { file } => migration_detect_command(file),
        },
    }
}

fn compile_command(file: PathBuf, output: PathBuf, debug: bool) -> Result<(), CclError> {
    println!("🔨 Compiling CCL contract: {}", file.display());

    // Create output directory
    std::fs::create_dir_all(&output)
        .map_err(|e| CclError::IoError(format!("Failed to create output directory: {}", e)))?;

    // Compile the contract
    let (wasm_bytes, metadata) = compile_ccl_file_to_wasm(&file)?;

    // Get file stem safely
    let file_stem = file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CclError::IoError(format!("Invalid file path: {}", file.display())))?;

    // Write WASM output
    let wasm_path = output.join(format!("{}.wasm", file_stem));
    std::fs::write(&wasm_path, wasm_bytes)
        .map_err(|e| CclError::IoError(format!("Failed to write WASM file: {}", e)))?;

    // Write metadata
    let meta_path = output.join(format!("{}.json", file_stem));
    let meta_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| CclError::IoError(format!("Failed to serialize metadata: {}", e)))?;
    std::fs::write(&meta_path, meta_json)
        .map_err(|e| CclError::IoError(format!("Failed to write metadata file: {}", e)))?;

    println!("✅ Compiled successfully:");
    println!("   WASM: {}", wasm_path.display());
    println!("   Meta: {}", meta_path.display());

    // Generate debug info if requested
    if debug {
        let debug_path = output.join(format!("{}.debug.json", file_stem));
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
                "Debug command requires a .debug.json file or use --interactive flag".to_string(),
            ));
        }
    }
}

fn format_command(files: Vec<PathBuf>) -> Result<(), CclError> {
    if files.is_empty() {
        return Err(CclError::IoError(
            "No files specified for formatting".to_string(),
        ));
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
        return Err(CclError::IoError(
            "No files specified for linting".to_string(),
        ));
    }

    for file in files {
        println!("🔍 Linting: {}", file.display());
        // TODO: Implement CCL linting
        println!("   (linting not yet implemented)");
    }

    Ok(())
}

fn migration_upgrade_command(
    input: PathBuf,
    output: Option<PathBuf>,
    target_version: Option<String>,
    dry_run: bool,
) -> Result<(), CclError> {
    use icn_ccl::migration::{CclVersion, MigrationEngine, CURRENT_CCL_VERSION};
    use std::fs;

    println!("🔄 Migrating CCL contract: {}", input.display());

    // Read input file
    let content = fs::read_to_string(&input)
        .map_err(|e| CclError::IoError(format!("Failed to read input file: {}", e)))?;

    // Initialize migration engine
    let engine = MigrationEngine::new();

    // Detect current version
    let detected_version = engine.detect_version(&content)?;
    println!("📋 Detected CCL version: {}", detected_version.to_string());

    // Parse target version
    let target = if let Some(version_str) = target_version {
        CclVersion::parse(&version_str)?
    } else {
        CURRENT_CCL_VERSION.clone()
    };
    println!("🎯 Target CCL version: {}", target.to_string());

    // Check if migration is needed
    if detected_version >= target {
        println!("✅ Contract is already at or newer than target version. No migration needed.");
        return Ok(());
    }

    // Generate migration report
    let report = engine.generate_migration_report(&content, &detected_version, &target);
    println!("\n{}", report.to_string());

    if dry_run {
        println!("🔍 Dry run completed. Use without --dry-run to apply changes.");
        return Ok(());
    }

    // Perform migration
    let migrated_content = engine.migrate(&content, &detected_version, &target)?;

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut path = input.clone();
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        path.set_file_name(format!("{}.migrated.ccl", file_stem));
        path
    });

    // Write output
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CclError::IoError(format!("Failed to create output directory: {}", e)))?;
    }

    fs::write(&output_path, migrated_content)
        .map_err(|e| CclError::IoError(format!("Failed to write output file: {}", e)))?;

    println!("✅ Migration completed: {}", output_path.display());
    Ok(())
}

fn migration_convert_command(
    input: PathBuf,
    output: PathBuf,
    source_language: String,
) -> Result<(), CclError> {
    use icn_ccl::migration::{convert_from_javascript, convert_from_solidity};
    use std::fs;

    println!(
        "🔄 Converting {} to CCL: {}",
        source_language,
        input.display()
    );

    // Read input file
    let content = fs::read_to_string(&input)
        .map_err(|e| CclError::IoError(format!("Failed to read input file: {}", e)))?;

    // Convert based on source language
    let ccl_content = match source_language.to_lowercase().as_str() {
        "solidity" | "sol" => convert_from_solidity(&content)?,
        "javascript" | "js" | "typescript" | "ts" => convert_from_javascript(&content)?,
        _ => {
            return Err(CclError::CliArgumentError(format!(
                "Unsupported source language: {}. Supported: solidity, javascript, typescript",
                source_language
            )));
        }
    };

    // Write output
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CclError::IoError(format!("Failed to create output directory: {}", e)))?;
    }

    fs::write(&output, ccl_content)
        .map_err(|e| CclError::IoError(format!("Failed to write output file: {}", e)))?;

    println!("✅ Conversion completed: {}", output.display());
    println!("⚠️  Note: Automated conversion requires manual review for correctness.");
    Ok(())
}

fn migration_detect_command(file: PathBuf) -> Result<(), CclError> {
    use icn_ccl::migration::MigrationEngine;
    use std::fs;

    println!("🔍 Detecting CCL version: {}", file.display());

    // Read file
    let content = fs::read_to_string(&file)
        .map_err(|e| CclError::IoError(format!("Failed to read file: {}", e)))?;

    // Detect version
    let engine = MigrationEngine::new();
    let detected_version = engine.detect_version(&content)?;

    println!("📋 Detected CCL version: {}", detected_version.to_string());

    // Analyze syntax patterns for additional insights
    let patterns = analyze_syntax_patterns(&content);
    if !patterns.is_empty() {
        println!("\n🔍 Syntax analysis:");
        for pattern in patterns {
            println!("   {}", pattern);
        }
    }

    Ok(())
}

fn analyze_syntax_patterns(content: &str) -> Vec<String> {
    let mut patterns = Vec::new();

    if content.contains("rule ") {
        patterns.push("✓ Contains 'rule' keyword (v0.1.x syntax)".to_string());
    }
    if content.contains("policy ") {
        patterns.push("✓ Contains 'policy' keyword (v0.2.x syntax)".to_string());
    }
    if content.contains("when ") && content.contains("then ") {
        patterns.push("✓ Uses when-then syntax (v0.1.x style)".to_string());
    }
    if content.contains("charge(") {
        patterns.push("✓ Uses charge() function (deprecated in v0.2.x)".to_string());
    }
    if content.contains("require_payment(") {
        patterns.push("✓ Uses require_payment() function (v0.2.x style)".to_string());
    }
    if content.contains("contract ") {
        patterns.push("✓ Modern contract definition".to_string());
    }
    if content.contains("// CCL Version:") {
        patterns.push("✓ Contains explicit version declaration".to_string());
    }

    patterns
}
