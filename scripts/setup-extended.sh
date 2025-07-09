#!/bin/bash

# ==============================================================================
# ICN Development Environment Setup - Extended Edition
# ==============================================================================
# Comprehensive setup script for InterCooperative Network development
# Supports multi-profile configuration, enhanced error handling, and modular utilities

set -euo pipefail

# ==============================================================================
# Configuration and Constants
# ==============================================================================

# Script metadata
readonly SCRIPT_NAME="ICN Extended Setup"
readonly SCRIPT_VERSION="2.0.0"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Color codes for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly PURPLE='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# Profile configurations
declare -A PROFILES=(
    ["dev"]="Development environment with hot reload and debugging"
    ["test"]="Testing environment with mock services and validation"
    ["prod"]="Production environment with full security and monitoring"
    ["infra"]="Infrastructure management with deployment tools"
)

# Default configuration
DEFAULT_PROFILE="dev"
PROFILE="${ICN_PROFILE:-$DEFAULT_PROFILE}"
VERBOSE="${ICN_VERBOSE:-false}"
SKIP_CHECKS="${ICN_SKIP_CHECKS:-false}"
FORCE_REINSTALL="${ICN_FORCE_REINSTALL:-false}"

# ==============================================================================
# Utility Functions
# ==============================================================================

# Load modular utilities
load_utilities() {
    local utils_dir="${SCRIPT_DIR}"
    local required_utils=("env-utils.sh" "docker-utils.sh" "frontend-utils.sh" "remote-utils.sh")
    
    for util in "${required_utils[@]}"; do
        local util_path="${utils_dir}/${util}"
        if [[ -f "$util_path" ]]; then
            log "INFO" "Loading utility: $util"
            source "$util_path"
        else
            log "WARN" "Utility not found: $util_path"
        fi
    done
}

# Enhanced logging function
log() {
    local level="$1"
    local message="$2"
    local component="${3:-SETUP}"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case "$level" in
        "INFO")  echo -e "${GREEN}[${timestamp}] [${component}] INFO: ${message}${NC}" ;;
        "WARN")  echo -e "${YELLOW}[${timestamp}] [${component}] WARN: ${message}${NC}" ;;
        "ERROR") echo -e "${RED}[${timestamp}] [${component}] ERROR: ${message}${NC}" ;;
        "DEBUG") [[ "$VERBOSE" == "true" ]] && echo -e "${BLUE}[${timestamp}] [${component}] DEBUG: ${message}${NC}" ;;
        "SUCCESS") echo -e "${GREEN}[${timestamp}] [${component}] SUCCESS: ${message}${NC}" ;;
        *)       echo -e "${CYAN}[${timestamp}] [${component}] ${level}: ${message}${NC}" ;;
    esac
}

# Progress indicator
show_progress() {
    local current="$1"
    local total="$2"
    local task="$3"
    local percent=$((current * 100 / total))
    
    printf "\r${BLUE}[%3d%%] %s${NC}" "$percent" "$task"
    if [[ "$current" -eq "$total" ]]; then
        echo ""
    fi
}

# Error recovery with suggestions
handle_error() {
    local error_code="$1"
    local error_message="$2"
    local suggestions="${3:-}"
    
    log "ERROR" "$error_message"
    
    if [[ -n "$suggestions" ]]; then
        echo -e "${YELLOW}Recovery suggestions:${NC}"
        echo -e "$suggestions"
    fi
    
    echo -e "${RED}Setup failed with error code: $error_code${NC}"
    
    # Cleanup on error
    cleanup_on_error
    
    exit "$error_code"
}

# Cleanup function
cleanup_on_error() {
    log "INFO" "Performing cleanup after error..."
    
    # Remove incomplete installations
    if [[ -d "$PROJECT_ROOT/tmp" ]]; then
        rm -rf "$PROJECT_ROOT/tmp"
    fi
    
    # Stop any running services
    if command -v docker-compose &> /dev/null; then
        docker-compose -f "$PROJECT_ROOT/docker-compose.yml" down 2>/dev/null || true
    fi
}

# ==============================================================================
# System Requirements Check
# ==============================================================================

check_system_requirements() {
    log "INFO" "Checking system requirements for profile: $PROFILE"
    
    local requirements_met=true
    local total_checks=8
    local current_check=0
    
    # Check OS compatibility
    ((current_check++))
    show_progress "$current_check" "$total_checks" "Checking OS compatibility..."
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        log "DEBUG" "Linux system detected"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        log "DEBUG" "macOS system detected"
    else
        log "WARN" "Unsupported OS type: $OSTYPE"
        requirements_met=false
    fi
    
    # Check required commands
    local required_commands=("git" "curl" "wget" "jq")
    
    case "$PROFILE" in
        "dev"|"test"|"prod")
            required_commands+=("rustc" "cargo")
            ;;
        "infra")
            required_commands+=("docker" "docker-compose" "ssh")
            ;;
    esac
    
    for cmd in "${required_commands[@]}"; do
        ((current_check++))
        show_progress "$current_check" "$total_checks" "Checking command: $cmd"
        
        if ! command -v "$cmd" &> /dev/null; then
            log "ERROR" "Required command not found: $cmd"
            requirements_met=false
        else
            log "DEBUG" "Found command: $cmd"
        fi
    done
    
    # Check Rust version for development profiles
    if [[ "$PROFILE" == "dev" || "$PROFILE" == "test" || "$PROFILE" == "prod" ]]; then
        ((current_check++))
        show_progress "$current_check" "$total_checks" "Checking Rust version..."
        
        if command -v rustc &> /dev/null; then
            local rust_version=$(rustc --version | cut -d' ' -f2)
            log "DEBUG" "Rust version: $rust_version"
            
            # Check for nightly as specified in MSRV
            if ! rustc --version | grep -q "nightly"; then
                log "WARN" "ICN requires Rust nightly. Consider running: rustup default nightly"
            fi
        fi
    fi
    
    if [[ "$requirements_met" == "false" ]]; then
        handle_error 1 "System requirements not met" "
        Please install missing dependencies:
        - For Ubuntu/Debian: apt-get install git curl wget jq
        - For macOS: brew install git curl wget jq
        - For Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        "
    fi
    
    log "SUCCESS" "System requirements check passed"
}

# ==============================================================================
# Profile-Specific Setup
# ==============================================================================

setup_profile_configuration() {
    log "INFO" "Setting up profile configuration: $PROFILE"
    
    # Create profile-specific directories
    local profile_dirs=(
        "$PROJECT_ROOT/.icn/$PROFILE"
        "$PROJECT_ROOT/.icn/$PROFILE/config"
        "$PROJECT_ROOT/.icn/$PROFILE/logs"
        "$PROJECT_ROOT/.icn/$PROFILE/data"
    )
    
    for dir in "${profile_dirs[@]}"; do
        mkdir -p "$dir"
        log "DEBUG" "Created directory: $dir"
    done
    
    # Generate profile-specific configuration
    if command -v generate_profile_config &> /dev/null; then
        generate_profile_config "$PROFILE"
    else
        log "WARN" "Profile configuration generator not available"
    fi
    
    # Set up environment variables
    export ICN_PROFILE="$PROFILE"
    export ICN_ROOT="$PROJECT_ROOT"
    export ICN_CONFIG_DIR="$PROJECT_ROOT/.icn/$PROFILE/config"
    export ICN_LOG_DIR="$PROJECT_ROOT/.icn/$PROFILE/logs"
    export ICN_DATA_DIR="$PROJECT_ROOT/.icn/$PROFILE/data"
    
    log "SUCCESS" "Profile configuration completed"
}

# ==============================================================================
# Core ICN Setup
# ==============================================================================

setup_icn_core() {
    log "INFO" "Setting up ICN core workspace"
    
    # Verify we're in the correct directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        handle_error 2 "Not in ICN core workspace root" "
        Please run this script from the icn-core repository root directory.
        "
    fi
    
    # Check workspace structure
    local expected_crates=(
        "icn-common" "icn-protocol" "icn-identity" "icn-dag"
        "icn-economics" "icn-mesh" "icn-governance" "icn-reputation"
        "icn-network" "icn-runtime" "icn-api" "icn-cli" "icn-node"
    )
    
    for crate in "${expected_crates[@]}"; do
        if [[ ! -d "$PROJECT_ROOT/crates/$crate" ]]; then
            log "WARN" "Expected crate not found: $crate"
        else
            log "DEBUG" "Found crate: $crate"
        fi
    done
    
    # Build workspace based on profile
    case "$PROFILE" in
        "dev")
            log "INFO" "Building ICN core in development mode"
            cargo build --workspace --all-features
            ;;
        "test")
            log "INFO" "Building ICN core for testing"
            cargo build --workspace --all-features
            cargo test --workspace --all-features --no-run
            ;;
        "prod")
            log "INFO" "Building ICN core for production"
            cargo build --workspace --all-features --release
            ;;
        "infra")
            log "INFO" "Skipping build for infrastructure profile"
            ;;
    esac
    
    log "SUCCESS" "ICN core setup completed"
}

# ==============================================================================
# Frontend and Additional Tools Setup
# ==============================================================================

setup_frontend_tools() {
    if [[ "$PROFILE" == "infra" ]]; then
        log "INFO" "Skipping frontend setup for infrastructure profile"
        return
    fi
    
    log "INFO" "Setting up frontend development tools"
    
    # Setup Node.js and pnpm
    if command -v setup_node_environment &> /dev/null; then
        setup_node_environment
    else
        log "WARN" "Node.js environment setup not available"
    fi
    
    # Setup frontend repositories
    if command -v setup_frontend_repositories &> /dev/null; then
        setup_frontend_repositories
    else
        log "WARN" "Frontend repositories setup not available"
    fi
    
    log "SUCCESS" "Frontend tools setup completed"
}

# ==============================================================================
# Docker and Containerization Setup
# ==============================================================================

setup_docker_environment() {
    log "INFO" "Setting up Docker environment"
    
    # Check Docker availability
    if ! command -v docker &> /dev/null; then
        log "WARN" "Docker not available, skipping containerization setup"
        return
    fi
    
    # Setup Docker environment
    if command -v setup_docker_services &> /dev/null; then
        setup_docker_services "$PROFILE"
    else
        log "WARN" "Docker services setup not available"
    fi
    
    log "SUCCESS" "Docker environment setup completed"
}

# ==============================================================================
# Remote Infrastructure Setup
# ==============================================================================

setup_remote_infrastructure() {
    if [[ "$PROFILE" != "infra" ]]; then
        log "INFO" "Skipping remote infrastructure setup for non-infra profile"
        return
    fi
    
    log "INFO" "Setting up remote infrastructure"
    
    # Setup SSH configuration
    if command -v setup_ssh_config &> /dev/null; then
        setup_ssh_config
    else
        log "WARN" "SSH configuration setup not available"
    fi
    
    # Setup remote nodes
    if command -v setup_remote_nodes &> /dev/null; then
        setup_remote_nodes
    else
        log "WARN" "Remote nodes setup not available"
    fi
    
    log "SUCCESS" "Remote infrastructure setup completed"
}

# ==============================================================================
# Aliases and Shortcuts Generation
# ==============================================================================

generate_development_aliases() {
    log "INFO" "Generating development aliases and shortcuts"
    
    local aliases_file="$PROJECT_ROOT/.icn/$PROFILE/aliases.sh"
    
    cat > "$aliases_file" << EOF
#!/bin/bash
# ICN Development Aliases - Profile: $PROFILE
# Generated by: $SCRIPT_NAME v$SCRIPT_VERSION
# Generated on: $(date)

# Core ICN aliases
alias icn-build='cargo build --workspace --all-features'
alias icn-test='cargo test --workspace --all-features'
alias icn-clippy='cargo clippy --all-targets --all-features -- -D warnings'
alias icn-fmt='cargo fmt --all'
alias icn-doc='cargo doc --workspace --all-features --no-deps --open'

# Profile-specific aliases
EOF
    
    case "$PROFILE" in
        "dev")
            cat >> "$aliases_file" << EOF
alias icn-dev='cargo run --bin icn-node --features dev'
alias icn-watch='cargo watch -x "run --bin icn-node --features dev"'
alias icn-debug='RUST_LOG=debug cargo run --bin icn-node --features dev'
EOF
            ;;
        "test")
            cat >> "$aliases_file" << EOF
alias icn-test-all='cargo test --workspace --all-features'
alias icn-test-unit='cargo test --workspace --lib'
alias icn-test-integration='cargo test --workspace --test "*"'
alias icn-coverage='cargo tarpaulin --workspace --all-features --out Html'
EOF
            ;;
        "prod")
            cat >> "$aliases_file" << EOF
alias icn-prod='cargo run --bin icn-node --release'
alias icn-build-release='cargo build --workspace --all-features --release'
alias icn-audit='cargo audit'
EOF
            ;;
        "infra")
            cat >> "$aliases_file" << EOF
alias icn-deploy='./scripts/deploy.sh'
alias icn-status='./scripts/status.sh'
alias icn-logs='./scripts/logs.sh'
EOF
            ;;
    esac
    
    # Add utility aliases if available
    if command -v generate_utility_aliases &> /dev/null; then
        generate_utility_aliases >> "$aliases_file"
    fi
    
    log "SUCCESS" "Development aliases generated: $aliases_file"
}

# ==============================================================================
# Health Check and Validation
# ==============================================================================

perform_health_check() {
    log "INFO" "Performing post-setup health check"
    
    local health_score=0
    local max_score=10
    
    # Check workspace build
    if cargo check --workspace --all-features &> /dev/null; then
        ((health_score++))
        log "SUCCESS" "Workspace builds successfully"
    else
        log "ERROR" "Workspace build failed"
    fi
    
    # Check test compilation
    if cargo test --workspace --all-features --no-run &> /dev/null; then
        ((health_score++))
        log "SUCCESS" "Tests compile successfully"
    else
        log "ERROR" "Test compilation failed"
    fi
    
    # Profile-specific health checks
    case "$PROFILE" in
        "dev")
            if [[ -f "$PROJECT_ROOT/.icn/$PROFILE/aliases.sh" ]]; then
                ((health_score++))
                log "SUCCESS" "Development aliases available"
            fi
            ;;
        "test")
            if cargo test --workspace --lib --quiet &> /dev/null; then
                ((health_score++))
                log "SUCCESS" "Unit tests pass"
            fi
            ;;
        "prod")
            if cargo build --workspace --all-features --release &> /dev/null; then
                ((health_score++))
                log "SUCCESS" "Production build successful"
            fi
            ;;
        "infra")
            if command -v docker &> /dev/null; then
                ((health_score++))
                log "SUCCESS" "Docker available for infrastructure"
            fi
            ;;
    esac
    
    # Calculate health percentage
    local health_percent=$((health_score * 100 / max_score))
    
    if [[ "$health_percent" -ge 80 ]]; then
        log "SUCCESS" "Health check passed ($health_percent%)"
    elif [[ "$health_percent" -ge 60 ]]; then
        log "WARN" "Health check partially passed ($health_percent%)"
    else
        log "ERROR" "Health check failed ($health_percent%)"
    fi
    
    return 0
}

# ==============================================================================
# Main Setup Function
# ==============================================================================

main() {
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                     ICN Development Environment Setup                       ║${NC}"
    echo -e "${CYAN}║                              Extended Edition                               ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
    echo
    
    log "INFO" "Starting $SCRIPT_NAME v$SCRIPT_VERSION"
    log "INFO" "Profile: $PROFILE"
    log "INFO" "Project Root: $PROJECT_ROOT"
    echo
    
    # Load utilities
    load_utilities
    
    # Setup steps
    local setup_steps=(
        "check_system_requirements"
        "setup_profile_configuration"
        "setup_icn_core"
        "setup_frontend_tools"
        "setup_docker_environment"
        "setup_remote_infrastructure"
        "generate_development_aliases"
        "perform_health_check"
    )
    
    local total_steps=${#setup_steps[@]}
    local current_step=0
    
    for step in "${setup_steps[@]}"; do
        ((current_step++))
        echo
        log "INFO" "Step $current_step/$total_steps: $step"
        
        if [[ "$SKIP_CHECKS" == "true" && "$step" == "check_system_requirements" ]]; then
            log "WARN" "Skipping system requirements check"
            continue
        fi
        
        if ! "$step"; then
            handle_error 3 "Setup step failed: $step"
        fi
    done
    
    echo
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                            Setup Complete!                                  ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
    echo
    
    # Show next steps
    echo -e "${CYAN}Next steps:${NC}"
    echo "1. Source the aliases: source .icn/$PROFILE/aliases.sh"
    echo "2. Run initial tests: icn-test"
    echo "3. Start development: icn-dev"
    echo
    
    log "SUCCESS" "ICN development environment setup completed successfully!"
}

# ==============================================================================
# Script Entry Point
# ==============================================================================

# Handle command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--profile)
            PROFILE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --skip-checks)
            SKIP_CHECKS=true
            shift
            ;;
        --force)
            FORCE_REINSTALL=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Options:"
            echo "  -p, --profile PROFILE    Set profile (dev|test|prod|infra)"
            echo "  -v, --verbose           Enable verbose output"
            echo "  --skip-checks           Skip system requirements check"
            echo "  --force                 Force reinstallation"
            echo "  -h, --help              Show this help message"
            echo
            echo "Available profiles:"
            for profile in "${!PROFILES[@]}"; do
                echo "  $profile - ${PROFILES[$profile]}"
            done
            exit 0
            ;;
        *)
            log "ERROR" "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate profile
if [[ -z "${PROFILES[$PROFILE]:-}" ]]; then
    log "ERROR" "Invalid profile: $PROFILE"
    echo "Available profiles: ${!PROFILES[*]}"
    exit 1
fi

# Set up signal handlers
trap 'handle_error 130 "Script interrupted by user"' INT TERM

# Run main setup
main "$@" 