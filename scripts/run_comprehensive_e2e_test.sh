#!/bin/bash

# ICN Comprehensive End-to-End Test Runner
# This script sets up the federation, runs the comprehensive E2E test, and provides reporting

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_RESULTS_DIR="$PROJECT_ROOT/test_results"
FEDERATION_COMPOSE_FILE="$PROJECT_ROOT/icn-devnet/docker-compose.yml"

# Command line options
FRESH_START=true
KEEP_RUNNING=false
VERBOSE=false
TEST_TIMEOUT=600

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --use-existing     Use existing federation (don't start fresh)"
    echo "  --keep-running     Keep federation running after test"
    echo "  --verbose          Enable verbose output"
    echo "  --timeout SECONDS  Set test timeout (default: 600)"
    echo "  --help            Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  ICN_DEVNET_RUNNING  Set to skip federation startup"
    echo "  ICN_E2E_TEST_TIMEOUT  Override test timeout"
    echo "  RUST_LOG           Set logging level"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --use-existing)
            FRESH_START=false
            shift
            ;;
        --keep-running)
            KEEP_RUNNING=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --timeout)
            TEST_TIMEOUT="$2"
            shift 2
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Setup test environment
setup_test_environment() {
    print_info "Setting up test environment..."
    
    # Create test results directory
    mkdir -p "$TEST_RESULTS_DIR"
    
    # Set environment variables
    export ICN_E2E_TEST_TIMEOUT="$TEST_TIMEOUT"
    export ICN_TEST_MODE="true"
    
    if [[ "$VERBOSE" == "true" ]]; then
        export RUST_LOG="${RUST_LOG:-debug}"
    else
        export RUST_LOG="${RUST_LOG:-info}"
    fi
    
    print_success "Test environment configured"
}

# Pre-flight checks
run_preflight_checks() {
    print_info "Running pre-flight checks..."
    
    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    # Check if Docker Compose is available
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed or not in PATH"
        exit 1
    fi
    
    # Check if required ports are available
    local required_ports=(5001 5002 5003 9090 3000)
    for port in "${required_ports[@]}"; do
        if netstat -tuln 2>/dev/null | grep -q ":$port "; then
            print_warning "Port $port is already in use"
        fi
    done
    
    # Check if project can build
    print_info "Checking project build..."
    cd "$PROJECT_ROOT"
    if ! cargo check --workspace --all-features &>/dev/null; then
        print_error "Project build check failed"
        exit 1
    fi
    
    print_success "Pre-flight checks passed"
}

# Start federation with monitoring
start_federation() {
    if [[ "$FRESH_START" == "false" ]] || [[ -n "$ICN_DEVNET_RUNNING" ]]; then
        print_info "Using existing federation..."
        return 0
    fi
    
    print_info "Starting federation with monitoring stack..."
    
    cd "$PROJECT_ROOT"
    
    # Stop any existing federation
    docker-compose -f "$FEDERATION_COMPOSE_FILE" --profile monitoring down --volumes --remove-orphans &>/dev/null || true
    
    # Start federation with monitoring
    docker-compose -f "$FEDERATION_COMPOSE_FILE" --profile monitoring up -d
    
    print_info "Waiting for federation to stabilize..."
    sleep 60
    
    # Health check
    local nodes=("http://localhost:5001" "http://localhost:5002" "http://localhost:5003")
    for node in "${nodes[@]}"; do
        local retries=0
        while [[ $retries -lt 30 ]]; do
            if curl -s -f "$node/info" &>/dev/null; then
                print_success "Node $node is healthy"
                break
            fi
            ((retries++))
            sleep 2
        done
        
        if [[ $retries -eq 30 ]]; then
            print_error "Node $node failed to start"
            exit 1
        fi
    done
    
    print_success "Federation started successfully"
}

# Run the comprehensive E2E test
run_comprehensive_test() {
    print_info "Running comprehensive E2E test..."
    
    cd "$PROJECT_ROOT"
    
         local test_cmd="cargo test --release --features enable-libp2p -p icn-integration-tests --test comprehensive_e2e"
    
    if [[ "$VERBOSE" == "true" ]]; then
        test_cmd="$test_cmd -- --nocapture"
    fi
    
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local log_file="$TEST_RESULTS_DIR/comprehensive_e2e_${timestamp}.log"
    
    print_info "Test command: $test_cmd"
    print_info "Test log: $log_file"
    
    # Set test timeout
    export ICN_E2E_TEST_TIMEOUT="$TEST_TIMEOUT"
    
    # Run the test
    if timeout "$TEST_TIMEOUT" bash -c "$test_cmd 2>&1 | tee '$log_file'"; then
        print_success "Comprehensive E2E test PASSED"
        return 0
    else
        print_error "Comprehensive E2E test FAILED"
        return 1
    fi
}

# Generate test report
generate_test_report() {
    local test_status=$1
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local report_file="$TEST_RESULTS_DIR/comprehensive_e2e_report_${timestamp}.html"
    
    print_info "Generating test report..."
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>ICN Comprehensive E2E Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 20px; border-radius: 5px; }
        .success { color: #008000; }
        .error { color: #ff0000; }
        .warning { color: #ffa500; }
        .info { color: #0066cc; }
        .metrics { background: #f9f9f9; padding: 15px; border-radius: 5px; margin: 10px 0; }
        pre { background: #f5f5f5; padding: 10px; border-radius: 3px; overflow-x: auto; }
    </style>
</head>
<body>
    <div class="header">
        <h1>ICN Comprehensive E2E Test Report</h1>
        <p><strong>Timestamp:</strong> $(date)</p>
        <p><strong>Test Status:</strong> <span class="$([ $test_status -eq 0 ] && echo 'success' || echo 'error')">$([ $test_status -eq 0 ] && echo 'PASSED' || echo 'FAILED')</span></p>
        <p><strong>Test Duration:</strong> ${TEST_TIMEOUT}s timeout</p>
    </div>
    
    <h2>Test Configuration</h2>
    <ul>
        <li><strong>Fresh Start:</strong> $FRESH_START</li>
        <li><strong>Keep Running:</strong> $KEEP_RUNNING</li>
        <li><strong>Verbose:</strong> $VERBOSE</li>
        <li><strong>Timeout:</strong> ${TEST_TIMEOUT}s</li>
    </ul>
    
    <h2>Federation Status</h2>
    <div class="metrics">
        <p><strong>Nodes:</strong> 3-node federation</p>
        <p><strong>Monitoring:</strong> Prometheus + Grafana</p>
        <p><strong>Ports:</strong> 5001-5003 (nodes), 9090 (Prometheus), 3000 (Grafana)</p>
    </div>
    
    <h2>Test Results</h2>
    <p>$([ $test_status -eq 0 ] && echo 'All test phases completed successfully.' || echo 'Test failed. Check logs for details.')</p>
    
    <h2>Monitoring Links</h2>
    <ul>
        <li><a href="http://localhost:9090" target="_blank">Prometheus</a></li>
        <li><a href="http://localhost:3000" target="_blank">Grafana</a> (admin/icnfederation)</li>
        <li><a href="http://localhost:5001/info" target="_blank">Node A</a></li>
        <li><a href="http://localhost:5002/info" target="_blank">Node B</a></li>
        <li><a href="http://localhost:5003/info" target="_blank">Node C</a></li>
    </ul>
    
    <h2>Log Files</h2>
    <p>Test logs are available in: <code>$TEST_RESULTS_DIR/</code></p>
    
    <h2>Next Steps</h2>
    $([ $test_status -eq 0 ] && echo '<p class="success">✅ Test passed! The ICN federation is working correctly.</p>' || echo '<p class="error">❌ Test failed. Review the logs and check federation status.</p>')
    
</body>
</html>
EOF
    
    print_success "Test report generated: $report_file"
}

# Collect diagnostics on failure
collect_diagnostics() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local diagnostics_dir="$TEST_RESULTS_DIR/diagnostics_$timestamp"
    
    print_info "Collecting diagnostics..."
    
    mkdir -p "$diagnostics_dir"
    
    # Collect container logs
    if command -v docker-compose &> /dev/null; then
        docker-compose -f "$FEDERATION_COMPOSE_FILE" --profile monitoring logs > "$diagnostics_dir/docker_logs.txt" 2>&1 || true
    fi
    
    # Collect container status
    docker ps -a > "$diagnostics_dir/docker_ps.txt" 2>&1 || true
    
    # Collect system info
    {
        echo "=== System Information ==="
        uname -a
        echo ""
        echo "=== Memory Usage ==="
        free -h
        echo ""
        echo "=== Disk Usage ==="
        df -h
        echo ""
        echo "=== Network Ports ==="
        netstat -tuln | grep -E ":(5001|5002|5003|9090|3000)"
    } > "$diagnostics_dir/system_info.txt" 2>&1 || true
    
    # Try to collect metrics
    for port in 5001 5002 5003; do
        curl -s "http://localhost:$port/metrics" > "$diagnostics_dir/node_${port}_metrics.txt" 2>&1 || true
    done
    
    print_success "Diagnostics collected in: $diagnostics_dir"
}

# Cleanup function
cleanup() {
    if [[ "$KEEP_RUNNING" == "false" ]] && [[ -z "$ICN_DEVNET_RUNNING" ]]; then
        print_info "Cleaning up federation..."
        cd "$PROJECT_ROOT"
        docker-compose -f "$FEDERATION_COMPOSE_FILE" --profile monitoring down --volumes --remove-orphans &>/dev/null || true
        print_success "Federation stopped"
    else
        print_info "Keeping federation running as requested"
    fi
}

# Main execution
main() {
    print_header "ICN Comprehensive E2E Test Runner"
    
    setup_test_environment
    run_preflight_checks
    start_federation
    
    local test_status=0
    
    if run_comprehensive_test; then
        print_success "Comprehensive E2E test completed successfully"
        test_status=0
    else
        print_error "Comprehensive E2E test failed"
        collect_diagnostics
        test_status=1
    fi
    
    generate_test_report $test_status
    cleanup
    
    exit $test_status
}

# Handle signals
trap cleanup EXIT

# Run main function
main "$@" 