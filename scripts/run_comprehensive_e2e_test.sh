#!/bin/bash

# Comprehensive E2E Test Runner for ICN
# This script sets up the environment and runs the full end-to-end test

set -e

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_NAME="comprehensive_e2e"
LOG_FILE="$PROJECT_ROOT/test_results/${TEST_NAME}_$(date +%Y%m%d_%H%M%S).log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cleanup() {
    print_status "Cleaning up..."
    
    # Stop federation if we started it
    if [ -z "$ICN_DEVNET_RUNNING" ]; then
        cd "$PROJECT_ROOT"
        docker-compose -f icn-devnet/docker-compose.yml --profile monitoring down --volumes --remove-orphans || true
    fi
    
    # Clean up test artifacts
    rm -rf "$PROJECT_ROOT"/*.sled || true
    rm -rf "$PROJECT_ROOT"/test_data || true
    
    print_status "Cleanup completed"
}

# Set up signal handlers for cleanup
trap cleanup EXIT
trap 'print_error "Test interrupted"; exit 1' INT TERM

# Main execution
main() {
    print_status "Starting ICN Comprehensive E2E Test"
    print_status "Log file: $LOG_FILE"
    
    # Create log directory
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Pre-flight checks
    print_status "Running pre-flight checks..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    if ! docker info &> /dev/null; then
        print_error "Docker daemon is not running"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed or not in PATH"
        exit 1
    fi
    
    # Check Rust toolchain
    if ! command -v cargo &> /dev/null; then
        print_error "Rust toolchain is not installed"
        exit 1
    fi
    
    print_success "Pre-flight checks passed"
    
    # Build the project
    print_status "Building ICN project..."
    if ! cargo build --release 2>&1 | tee -a "$LOG_FILE"; then
        print_error "Failed to build project"
        exit 1
    fi
    print_success "Project built successfully"
    
    # Check if federation is already running
    if [ -n "$ICN_DEVNET_RUNNING" ]; then
        print_warning "Federation is already running (ICN_DEVNET_RUNNING is set)"
        print_status "Will use existing federation"
    else
        print_status "Starting fresh federation with monitoring stack..."
        
        # Clean up any existing containers
        docker-compose -f icn-devnet/docker-compose.yml --profile monitoring down --volumes --remove-orphans || true
        
        # Start federation with monitoring
        if ! docker-compose -f icn-devnet/docker-compose.yml --profile monitoring up -d 2>&1 | tee -a "$LOG_FILE"; then
            print_error "Failed to start federation"
            exit 1
        fi
        
        print_status "Waiting for federation to stabilize..."
        sleep 60
        
        # Check if all services are healthy
        services=("icn-node-a" "icn-node-b" "icn-node-c" "prometheus" "grafana")
        for service in "${services[@]}"; do
            if ! docker-compose -f icn-devnet/docker-compose.yml --profile monitoring ps | grep -q "$service.*Up"; then
                print_error "Service $service is not running"
                docker-compose -f icn-devnet/docker-compose.yml --profile monitoring logs "$service" | tail -20
                exit 1
            fi
        done
        
        print_success "Federation started successfully"
    fi
    
    # Run the comprehensive E2E test
    print_status "Running comprehensive E2E test..."
    
    # Set test environment
    export RUST_LOG=info,icn_node=debug,icn_runtime=debug
    export ICN_TEST_MODE=true
    export ICN_E2E_TEST_TIMEOUT=600  # 10 minutes
    
    # Run the test
    if cargo test --release comprehensive_mesh_job_e2e_test --features="enable-libp2p" -- --nocapture 2>&1 | tee -a "$LOG_FILE"; then
        print_success "Comprehensive E2E test completed successfully!"
        
        # Generate test report
        print_status "Generating test report..."
        generate_test_report
        
    else
        print_error "Comprehensive E2E test failed"
        
        # Collect diagnostic information
        print_status "Collecting diagnostic information..."
        collect_diagnostics
        
        exit 1
    fi
}

# Generate test report
generate_test_report() {
    local report_file="$PROJECT_ROOT/test_results/${TEST_NAME}_report_$(date +%Y%m%d_%H%M%S).html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>ICN Comprehensive E2E Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .success { color: green; }
        .error { color: red; }
        .warning { color: orange; }
        .info { color: blue; }
        .section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
        .metrics { background-color: #f9f9f9; }
        pre { background-color: #f0f0f0; padding: 10px; border-radius: 3px; overflow-x: auto; }
    </style>
</head>
<body>
    <h1>ICN Comprehensive E2E Test Report</h1>
    <p><strong>Test Date:</strong> $(date)</p>
    <p><strong>Test Duration:</strong> View log file for details</p>
    
    <div class="section">
        <h2>Test Overview</h2>
        <p>This comprehensive end-to-end test validates the complete ICN mesh job lifecycle including:</p>
        <ul>
            <li>Multi-node federation setup and convergence</li>
            <li>Complete mesh job lifecycle (submit → bid → execute → complete)</li>
            <li>DAG receipt anchoring and queries</li>
            <li>Mana balance tracking and automatic refunds</li>
            <li>Prometheus metrics collection</li>
            <li>Performance under load</li>
        </ul>
    </div>
    
    <div class="section success">
        <h2>✅ Test Result: SUCCESS</h2>
        <p>All test phases completed successfully.</p>
    </div>
    
    <div class="section metrics">
        <h2>📊 Key Metrics</h2>
        <p>Access the following URLs to view detailed metrics:</p>
        <ul>
            <li><a href="http://localhost:9090" target="_blank">Prometheus</a> - Raw metrics and queries</li>
            <li><a href="http://localhost:3000" target="_blank">Grafana</a> - Dashboards and visualizations</li>
            <li><a href="http://localhost:5001/metrics" target="_blank">Node A Metrics</a> - Node-specific metrics</li>
        </ul>
    </div>
    
    <div class="section">
        <h2>🔍 Detailed Results</h2>
        <p>See the full test log: <code>$LOG_FILE</code></p>
        <p>Test artifacts are available in: <code>$PROJECT_ROOT/test_results/</code></p>
    </div>
    
    <div class="section">
        <h2>🐛 Troubleshooting</h2>
        <p>If you encounter issues:</p>
        <ul>
            <li>Check the Docker containers are running: <code>docker-compose -f icn-devnet/docker-compose.yml --profile monitoring ps</code></li>
            <li>View container logs: <code>docker-compose -f icn-devnet/docker-compose.yml --profile monitoring logs [service-name]</code></li>
            <li>Ensure ports are available: 5001-5003 (nodes), 9090 (prometheus), 3000 (grafana)</li>
            <li>Check the troubleshooting guide in the documentation</li>
        </ul>
    </div>
    
</body>
</html>
EOF
    
    print_success "Test report generated: $report_file"
    
    # Try to open the report in a browser
    if command -v xdg-open &> /dev/null; then
        xdg-open "$report_file" &
    elif command -v open &> /dev/null; then
        open "$report_file" &
    fi
}

# Collect diagnostic information on failure
collect_diagnostics() {
    local diag_dir="$PROJECT_ROOT/test_results/diagnostics_$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$diag_dir"
    
    print_status "Collecting diagnostic information in $diag_dir..."
    
    # Container status
    docker-compose -f icn-devnet/docker-compose.yml --profile monitoring ps > "$diag_dir/container_status.txt" 2>&1
    
    # Container logs
    for service in icn-node-a icn-node-b icn-node-c prometheus grafana; do
        docker-compose -f icn-devnet/docker-compose.yml --profile monitoring logs "$service" > "$diag_dir/${service}_logs.txt" 2>&1
    done
    
    # System information
    docker system info > "$diag_dir/docker_system_info.txt" 2>&1
    df -h > "$diag_dir/disk_usage.txt" 2>&1
    free -h > "$diag_dir/memory_usage.txt" 2>&1
    
    # Network information
    docker network ls > "$diag_dir/docker_networks.txt" 2>&1
    docker network inspect icn-devnet_icn-federation > "$diag_dir/federation_network.txt" 2>&1
    
    # Try to get metrics if available
    curl -s http://localhost:9090/api/v1/label/__name__/values > "$diag_dir/prometheus_metrics.txt" 2>/dev/null || true
    
    print_success "Diagnostic information collected in $diag_dir"
}

# Help function
show_help() {
    echo "ICN Comprehensive E2E Test Runner"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -k, --keep-running  Keep federation running after test completion"
    echo "  -v, --verbose       Enable verbose logging"
    echo ""
    echo "Environment Variables:"
    echo "  ICN_DEVNET_RUNNING  If set, use existing federation instead of starting new one"
    echo "  ICN_E2E_TEST_TIMEOUT  Test timeout in seconds (default: 600)"
    echo ""
    echo "Examples:"
    echo "  $0                  # Run full test with fresh federation"
    echo "  ICN_DEVNET_RUNNING=1 $0  # Use existing federation"
    echo "  $0 --keep-running   # Keep federation running after test"
}

# Parse command line arguments
KEEP_RUNNING=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -k|--keep-running)
            KEEP_RUNNING=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Set verbose logging if requested
if [ "$VERBOSE" = true ]; then
    set -x
fi

# Modify cleanup behavior if keep-running is set
if [ "$KEEP_RUNNING" = true ]; then
    cleanup() {
        print_status "Keeping federation running as requested"
        print_status "To stop manually: docker-compose -f icn-devnet/docker-compose.yml --profile monitoring down --volumes"
    }
fi

# Run main function
main "$@" 