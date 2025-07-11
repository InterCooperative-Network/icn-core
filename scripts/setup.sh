#!/bin/bash
# ICN Core full environment setup script (works locally and in CI)

set -euo pipefail
IFS=$'\n\t'

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_FILE="/tmp/icn-setup.log"
ERROR_COUNT=0
WARNING_COUNT=0

# Color codes
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'

log()      { echo -e "${BLUE}ℹ️  $1${NC}" | tee -a "$LOG_FILE"; }
success()  { echo -e "${GREEN}✅ $1${NC}" | tee -a "$LOG_FILE"; }
warn()     { echo -e "${YELLOW}⚠️  $1${NC}" | tee -a "$LOG_FILE"; ((WARNING_COUNT++)); }
error()    { echo -e "${RED}❌ $1${NC}" | tee -a "$LOG_FILE"; ((ERROR_COUNT++)); }

trap 'error "Script failed at line $LINENO: $BASH_COMMAND"; exit 1' ERR

echo "ICN Setup Log — $(date)" > "$LOG_FILE"
cd "$REPO_ROOT"

log "🔍 Starting ICN setup in $REPO_ROOT"

# Step 1: Check Rust toolchain
log "🔧 Verifying Rust toolchain..."
if ! command -v rustc >/dev/null; then
  error "Rust is not installed. Install with rustup."
  exit 1
fi
rustup component add rustfmt clippy || warn "Failed to install rustfmt or clippy"

# Step 2: Fetch dependencies
log "📦 Fetching dependencies..."
if ! cargo fetch --locked; then
  warn "Cargo.lock may be stale, retrying..."
  cargo fetch || { error "cargo fetch failed"; exit 1; }
fi

# Step 3: Check project structure
log "🗂 Checking project structure..."
for dir in crates/icn-common crates/icn-node crates/icn-runtime icn-ccl icn-devnet; do
  [ -d "$dir" ] && success "Found $dir" || warn "Missing: $dir"
done

# Step 4: Build test
log "🔨 Building crates..."
cargo check -p icn-common || warn "icn-common failed to build"
cargo check -p icn-node || warn "icn-node failed to build"

# Step 5: Formatting and linting
log "🎨 Checking formatting..."
cargo fmt --all -- --check || {
  warn "Formatting issues found, fixing..."
  cargo fmt --all
}

log "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings || warn "Clippy warnings"

# Step 6: Docker check
log "🐳 Checking Docker (optional)..."
if command -v docker >/dev/null; then
  docker info >/dev/null 2>&1 && success "Docker available" || warn "Docker daemon not running"
else
  warn "Docker not installed — devnet unavailable"
fi

# Step 7: Final Summary
echo ""
log "📊 ICN setup complete"
echo "  ✅ Errors: $ERROR_COUNT"
echo "  ⚠️  Warnings: $WARNING_COUNT"
echo "  📄 Log file: $LOG_FILE"

[ $ERROR_COUNT -eq 0 ] && success "🎉 Setup completed successfully!" || exit 1
