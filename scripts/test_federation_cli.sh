#!/bin/bash

# Test script for new federation CLI commands
set -e

echo "🧪 Testing Federation CLI Commands"
echo "=================================="

# Build the CLI
echo "Building CLI..."
cargo build -p icn-cli --quiet

CLI_BIN="./target/debug/icn-cli"

echo "✅ CLI built successfully"

# Test federation trust commands
echo ""
echo "📋 Testing federation trust commands..."

# Test help for trust commands
echo "Testing federation trust help..."
$CLI_BIN federation trust --help > /dev/null 2>&1
echo "✅ Federation trust help works"

# Test help for metadata commands  
echo "Testing federation metadata help..."
$CLI_BIN federation metadata --help > /dev/null 2>&1
echo "✅ Federation metadata help works"

# Test help for DID commands
echo "Testing federation DID help..."
$CLI_BIN federation did --help > /dev/null 2>&1
echo "✅ Federation DID help works"

echo ""
echo "🎉 All federation CLI tests passed!"
echo "New commands are available:"
echo "  - icn-cli federation trust [configure|add|remove|list|validate|bridge|bootstrap]"
echo "  - icn-cli federation metadata [get|set|scope|quorum|members|add-member|remove-member]"
echo "  - icn-cli federation did [generate|verify|publish|resolve]"