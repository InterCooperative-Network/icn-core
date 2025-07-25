# CCL Developer Tooling

This document describes the new developer tooling for CCL (Cooperative Contract Language) that implements modern IDE-like development experience.

## Overview

The CCL developer tooling provides three main features requested in issue #958:

1. **Language Server Protocol (LSP) support** for autocompletion, go-to-definition, and inline documentation
2. **Debugger for CCL/WASM contracts** with source mapping and breakpoint support  
3. **Package manager for CCL modules** and governance patterns

## Installation

### Install CCL Tools Globally

```bash
# Install both the LSP server and CLI tools
just install-ccl-tools

# Or install manually
cargo install --path icn-ccl --bin ccl-lsp
cargo install --path icn-ccl --bin ccl-cli
```

### VSCode Extension

1. Open VSCode
2. Install the ICN CCL Tools extension from the `vscode-extension/` directory
3. Configure the LSP server path if needed (auto-detected by default)

## Features

### 1. Language Server Protocol (LSP)

The CCL LSP server (`ccl-lsp`) provides:

- **Autocompletion**: CCL keywords, built-in types, standard library functions
- **Hover Information**: Detailed documentation for symbols at cursor
- **Diagnostics**: Real-time error reporting and syntax checking
- **Go-to-Definition**: Navigate to symbol definitions (infrastructure ready)
- **Find References**: Find all references to symbols (infrastructure ready)

#### Starting the LSP Server

```bash
# Start LSP server (usually started automatically by IDE)
ccl-lsp
```

#### Autocompletion Features

- **CCL Keywords**: `contract`, `proposal`, `role`, `policy`, `function`, `vote`, etc.
- **Built-in Types**: `u32`, `u64`, `string`, `bool`, `mana`, `did`, `address`, etc.
- **Standard Library**: `log()`, `require()`, `transfer()`, `balance()`, `now()`, etc.

### 2. Debugger Support

The CCL debugger provides source mapping between CCL source code and compiled WASM bytecode.

#### Generate Debug Information

```bash
# Compile with debug info
ccl-cli compile contract.ccl --debug

# Or use justfile command
just ccl-compile-debug contract.ccl
```

#### Interactive Debugging

```bash
# Start interactive debugger (coming soon)
ccl-cli debug contract.debug.json --interactive

# Or use justfile command  
just ccl-debug contract.debug.json
```

#### Debugger Features

- **Source Mapping**: Map WASM instructions back to CCL source lines
- **Breakpoints**: Set breakpoints in CCL source code
- **Stack Inspection**: View call stack and local variables
- **Step Debugging**: Step through code execution (infrastructure ready)

### 3. Package Manager

The CCL package manager enables sharing and reusing governance patterns and CCL modules.

#### Package Structure

A CCL package has this structure:
```
my-governance/
├── package.ccl          # Package manifest
├── src/
│   └── main.ccl         # Main contract
├── examples/            # Example usage
└── README.md           # Documentation
```

#### Package Manifest (`package.ccl`)

```toml
[package]
name = "my-governance"
version = "0.1.0"
description = "A CCL governance package"
license = "Apache-2.0"

[[package.authors]]
name = "Your Name"
email = "you@example.com"

[dependencies]
governance-lib = "^1.0.0"
voting-patterns = "~2.1.0"

[dev_dependencies]
test-utils = "^0.5.0"
```

#### Package Commands

```bash
# Create a new package
ccl-cli package init my-governance --author "Your Name" --email "you@example.com"

# Or with justfile
just ccl-init my-governance "Your Name"

# Install dependencies
ccl-cli package install

# Or with justfile  
just ccl-install

# Add a dependency
ccl-cli package add governance-lib "^1.0.0"

# Or with justfile
just ccl-add-dep governance-lib "^1.0.0"
```

## CLI Tools

### ccl-cli Commands

```bash
# Compile contracts
ccl-cli compile contract.ccl --output ./target/ccl --debug

# Package management
ccl-cli package init my-project --author "Your Name"
ccl-cli package install  
ccl-cli package add dependency-name "^1.0.0"

# Debugging
ccl-cli debug contract.debug.json --interactive

# Code quality
ccl-cli format src/*.ccl
ccl-cli lint src/*.ccl
```

### Justfile Integration

The justfile provides convenient commands for development:

```bash
# CCL Development
just ccl-lsp                    # Start LSP server
just ccl-compile-debug file.ccl # Compile with debug info
just ccl-debug file.ccl         # Start debugger
just ccl-format                 # Format all CCL files
just ccl-lint                   # Lint all CCL files
just ccl-test                   # Run CCL tests

# Package Management
just ccl-init name author       # Create new package
just ccl-install               # Install dependencies
just ccl-add-dep name version  # Add dependency

# Installation
just install-ccl-tools         # Install tools globally
```

## VSCode Integration

The enhanced VSCode extension provides:

### Features

1. **Full LSP Integration**: All language server features
2. **Syntax Highlighting**: Enhanced CCL syntax highlighting  
3. **Build Tasks**: Compile CCL contracts from IDE
4. **Debug Integration**: Debug compiled contracts (coming soon)
5. **Package Commands**: Initialize and manage packages

### Configuration

```json
{
  "ccl.lsp.enabled": true,
  "ccl.lsp.serverPath": "",  // Auto-detected if empty
  "ccl.compile.outputDir": "./target/ccl",
  "ccl.debug.enabled": true
}
```

### Commands

- `CCL: Compile File` - Compile the current CCL file
- `CCL: Debug Contract` - Debug the current contract
- `CCL: Initialize Package` - Create a new CCL package
- `CCL: Install Dependencies` - Install package dependencies

## Development Workflow

### 1. Create a New Governance Contract

```bash
# Create new package
just ccl-init voting-system "Your Name"
cd voting-system

# Edit contract in src/main.ccl
code src/main.ccl

# Install dependencies if needed
just ccl-add-dep governance-lib "^1.0.0"
just ccl-install

# Compile with debug info
just ccl-compile-debug src/main.ccl

# Test and debug
just ccl-test
just ccl-debug src/main.ccl
```

### 2. IDE Development

1. Open VSCode in your project directory
2. Install the ICN CCL Tools extension
3. Edit CCL files with full LSP support:
   - Autocompletion while typing
   - Hover for documentation
   - Real-time error checking
   - Go-to-definition (Ctrl+Click)
4. Use `Ctrl+Shift+P` → "CCL: Compile File" to build
5. Use the integrated terminal for package management

### 3. Package Sharing

The package registry infrastructure is ready for sharing governance patterns:

```bash
# Search for packages (when registry is available)
ccl-cli search "voting patterns"

# Publish your package (when registry is available)  
ccl-cli publish --token your-auth-token
```

## Architecture

### LSP Server (`ccl-lsp`)

The Language Server Protocol implementation provides:

- **Document Management**: Track open CCL files and their state
- **Parsing Integration**: Use existing CCL parser for syntax analysis
- **Semantic Analysis**: Leverage semantic analyzer for type checking
- **Completion Engine**: Generate context-aware completions
- **Diagnostics**: Convert compiler errors to LSP diagnostics

### Debugger System

- **Source Maps**: JSON mapping between CCL source and WASM bytecode
- **Breakpoint Management**: Track and manage debugging breakpoints  
- **WASM Integration**: Interface with WASM runtime for execution control
- **Stack Inspection**: Provide call stack and variable information

### Package Manager

- **Manifest Format**: TOML-based package description format
- **Dependency Resolution**: Resolve version constraints and conflicts
- **Registry Integration**: HTTP API client for package registry
- **Local Storage**: Manage local package cache and dependencies

## Testing

Run the developer tooling tests:

```bash
# Test all new tooling
cargo test -p icn-ccl --test test_developer_tooling

# Test specific features
cargo test -p icn-ccl test_package_creation
cargo test -p icn-ccl test_lsp_completion  
cargo test -p icn-ccl test_source_map_creation
```

## Implementation Status

✅ **Completed:**
- LSP server with autocompletion, hover, and diagnostics
- Enhanced VSCode extension with LSP integration
- Package manager with dependency resolution
- CLI tools for compilation, debugging, and package management
- Source mapping infrastructure for debugging
- Comprehensive test suite
- Justfile integration for development workflows

🚧 **Future Enhancements:**
- Advanced go-to-definition and find references (infrastructure ready)
- Interactive step-through debugging (infrastructure ready)
- Package registry server and publishing
- Advanced code formatting and linting
- Real-time collaborative editing features
- Integration with ICN devnet for contract testing

## Contributing

The developer tooling follows the established ICN Core patterns:

1. **Replace Stub Implementations**: Convert TODO items to working code
2. **Comprehensive Testing**: Add tests for all new functionality  
3. **Documentation**: Update docs immediately with code changes
4. **Minimal Changes**: Make surgical, precise modifications
5. **Build Integration**: Ensure tools work with existing workflows

See the main project documentation for development guidelines and contribution processes.