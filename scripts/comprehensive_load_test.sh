#!/bin/bash

# ICN Core Production Load Testing Suite
# Comprehensive load testing with CI integration for production readiness validation

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TEST_RESULTS_DIR="${PROJECT_ROOT}/test-results"
LOAD_TEST_CONFIG="${PROJECT_ROOT}/configs/load-test-config.toml"

# Test parameters (can be overridden by environment variables)
FEDERATION_SIZE="${FEDERATION_SIZE:-3}"
TEST_DURATION="${TEST_DURATION:-300}" # 5 minutes
JOB_SUBMISSION_RATE="${JOB_SUBMISSION_RATE:-10}" # jobs per second
CONCURRENT_CONNECTIONS="${CONCURRENT_CONNECTIONS:-50}"
MAX_MEMORY_MB="${MAX_MEMORY_MB:-2048}"
MAX_CPU_PERCENT="${MAX_CPU_PERCENT:-80}"
MAX_MEMORY_PERCENT="${MAX_MEMORY_PERCENT:-85}"
PROMETHEUS_PORT="${PROMETHEUS_PORT:-9090}"
METRICS_SCRAPE_INTERVAL="${METRICS_SCRAPE_INTERVAL:-5}"

# CI Integration flags
CI_MODE="${CI_MODE:-false}"
FAIL_ON_PERFORMANCE_REGRESSION="${FAIL_ON_PERFORMANCE_REGRESSION:-true}"
BASELINE_RESULTS_FILE="${BASELINE_RESULTS_FILE:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1" >&2
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check dependencies
check_dependencies() {
    log "Checking dependencies..."
    
    local deps=("cargo" "docker" "docker-compose" "curl" "jq")
    local missing_deps=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing_deps+=("$dep")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        error "Missing dependencies: ${missing_deps[*]}"
        error "Please install the missing dependencies and try again."
        exit 1
    fi
    
    success "All dependencies found"
}

# Setup test environment
setup_test_environment() {
    log "Setting up test environment..."
    
    # Create test results directory
    mkdir -p "$TEST_RESULTS_DIR"
    
    # Generate load test configuration
    cat > "$LOAD_TEST_CONFIG" << EOF
[load_test]
federation_size = $FEDERATION_SIZE
test_duration_seconds = $TEST_DURATION
job_submission_rate = $JOB_SUBMISSION_RATE
concurrent_connections = $CONCURRENT_CONNECTIONS

[performance_limits]
max_memory_mb = $MAX_MEMORY_MB
max_cpu_percent = $MAX_CPU_PERCENT
max_response_time_ms = 1000
min_success_rate = 0.95

[monitoring]
prometheus_port = $PROMETHEUS_PORT
scrape_interval_seconds = $METRICS_SCRAPE_INTERVAL
collect_profiles = true
EOF

    success "Test environment configured"
}

# Build ICN Core with optimizations
build_icn_core() {
    log "Building ICN Core with optimizations..."
    
    cd "$PROJECT_ROOT"
    
    # Build with production optimizations
    cargo build --release --all-features
    
    if [ $? -ne 0 ]; then
        error "Build failed"
        exit 1
    fi
    
    success "ICN Core built successfully"
}

# Start monitoring infrastructure
start_monitoring() {
    log "Starting monitoring infrastructure..."
    
    cd "$PROJECT_ROOT/monitoring"
    
    # Start Prometheus and Grafana
    docker-compose -f docker-compose-monitoring.yml up -d
    
    # Wait for services to be ready
    local max_wait=60
    local count=0
    
    while ! curl -s "http://localhost:$PROMETHEUS_PORT/-/ready" > /dev/null; do
        if [ $count -ge $max_wait ]; then
            error "Prometheus failed to start within $max_wait seconds"
            exit 1
        fi
        sleep 1
        ((count++))
    done
    
    success "Monitoring infrastructure started"
}

# Start ICN federation
start_federation() {
    log "Starting ICN federation with $FEDERATION_SIZE nodes..."
    
    cd "$PROJECT_ROOT"
    
    # Clean any existing federation
    ./scripts/cleanup-devnet.sh 2>/dev/null || true
    
    # Start federation
    FEDERATION_SIZE="$FEDERATION_SIZE" ./scripts/run_federation_load_test.sh
    
    # Wait for federation to be ready
    local max_wait=120
    local count=0
    local ready_nodes=0
    
    while [ $ready_nodes -lt $FEDERATION_SIZE ]; do
        if [ $count -ge $max_wait ]; then
            error "Federation failed to start within $max_wait seconds"
            exit 1
        fi
        
        ready_nodes=$(curl -s "http://localhost:$PROMETHEUS_PORT/api/v1/query?query=up{job=\"icn-node\"}" | jq -r '.data.result | length' 2>/dev/null || echo "0")
        
        log "Ready nodes: $ready_nodes/$FEDERATION_SIZE"
        sleep 2
        ((count++))
    done
    
    success "Federation started with $FEDERATION_SIZE nodes"
}

# Run load test scenarios
run_load_tests() {
    log "Running load test scenarios..."
    
    local test_start_time=$(date +%s)
    local test_results_file="$TEST_RESULTS_DIR/load_test_$(date +%Y%m%d_%H%M%S).json"
    
    # Scenario 1: Sustained job submission
    log "Scenario 1: Sustained job submission at $JOB_SUBMISSION_RATE jobs/sec for ${TEST_DURATION}s"
    run_job_submission_test "$test_results_file"
    
    # Scenario 2: Burst traffic
    log "Scenario 2: Burst traffic test"
    run_burst_traffic_test "$test_results_file"
    
    # Scenario 3: Network partition simulation
    log "Scenario 3: Network partition simulation"
    run_network_partition_test "$test_results_file"
    
    # Scenario 4: Memory pressure test
    log "Scenario 4: Memory pressure test"
    run_memory_pressure_test "$test_results_file"
    
    # Scenario 5: Concurrent governance operations
    log "Scenario 5: Concurrent governance operations"
    run_governance_load_test "$test_results_file"
    
    local test_end_time=$(date +%s)
    local total_duration=$((test_end_time - test_start_time))
    
    log "Load tests completed in ${total_duration} seconds"
    
    # Generate final report
    generate_test_report "$test_results_file" "$total_duration"
}

# Job submission load test
run_job_submission_test() {
    local results_file="$1"
    local test_name="job_submission_sustained"
    
    log "Starting sustained job submission test..."
    
    # Start background job submission
    cargo run --release --bin load-test-client -- \
        --test-type job-submission \
        --rate "$JOB_SUBMISSION_RATE" \
        --duration "$TEST_DURATION" \
        --output "$results_file" \
        --test-name "$test_name" &
    
    local load_test_pid=$!
    
    # Monitor system resources during test
    monitor_system_resources "$test_name" "$TEST_DURATION" &
    local monitor_pid=$!
    
    # Wait for test completion
    wait $load_test_pid
    local exit_code=$?
    
    # Stop monitoring
    kill $monitor_pid 2>/dev/null || true
    
    if [ $exit_code -ne 0 ]; then
        error "Job submission test failed with exit code $exit_code"
        return 1
    fi
    
    success "Sustained job submission test completed"
}

# Burst traffic test
run_burst_traffic_test() {
    local results_file="$1"
    local test_name="burst_traffic"
    
    log "Starting burst traffic test..."
    
    # Run multiple short bursts
    for burst in {1..5}; do
        log "Running burst $burst/5"
        
        cargo run --release --bin load-test-client -- \
            --test-type burst \
            --rate $((JOB_SUBMISSION_RATE * 10)) \
            --duration 10 \
            --output "$results_file" \
            --test-name "${test_name}_burst_${burst}" &
        
        local burst_pid=$!
        wait $burst_pid
        
        # Cool down period
        sleep 5
    done
    
    success "Burst traffic test completed"
}

# Network partition simulation
run_network_partition_test() {
    local results_file="$1"
    local test_name="network_partition"
    
    log "Starting network partition test..."
    
    # Simulate network partition by blocking traffic between nodes
    ./scripts/simulate_network_partition.sh start &
    local partition_pid=$!
    
    # Continue job submission during partition
    cargo run --release --bin load-test-client -- \
        --test-type job-submission \
        --rate $((JOB_SUBMISSION_RATE / 2)) \
        --duration 60 \
        --output "$results_file" \
        --test-name "$test_name" &
    
    local load_test_pid=$!
    
    # Wait for test completion
    wait $load_test_pid
    
    # Restore network
    ./scripts/simulate_network_partition.sh stop
    wait $partition_pid 2>/dev/null || true
    
    # Allow recovery time
    sleep 30
    
    success "Network partition test completed"
}

# Memory pressure test
run_memory_pressure_test() {
    local results_file="$1"
    local test_name="memory_pressure"
    
    log "Starting memory pressure test..."
    
    # Submit large jobs to create memory pressure
    cargo run --release --bin load-test-client -- \
        --test-type large-jobs \
        --rate 5 \
        --duration 120 \
        --job-size-mb 50 \
        --output "$results_file" \
        --test-name "$test_name" &
    
    local load_test_pid=$!
    
    # Monitor memory usage
    monitor_memory_usage "$test_name" 120 &
    local monitor_pid=$!
    
    wait $load_test_pid
    kill $monitor_pid 2>/dev/null || true
    
    success "Memory pressure test completed"
}

# Governance load test
run_governance_load_test() {
    local results_file="$1"
    local test_name="governance_load"
    
    log "Starting governance load test..."
    
    # Submit multiple governance proposals and votes concurrently
    cargo run --release --bin load-test-client -- \
        --test-type governance \
        --rate 2 \
        --duration 180 \
        --output "$results_file" \
        --test-name "$test_name" &
    
    local load_test_pid=$!
    wait $load_test_pid
    
    success "Governance load test completed"
}

# Monitor system resources
monitor_system_resources() {
    local test_name="$1"
    local duration="$2"
    local output_file="$TEST_RESULTS_DIR/system_metrics_${test_name}.csv"
    
    echo "timestamp,cpu_percent,memory_mb,disk_io_mb,network_io_mb" > "$output_file"
    
    local end_time=$(($(date +%s) + duration))
    
    while [ $(date +%s) -lt $end_time ]; do
        local timestamp=$(date +%s)
        local cpu=$(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1}')
        local memory=$(free -m | awk 'NR==2{printf "%.2f", $3}')
        
        echo "${timestamp},${cpu},${memory},0,0" >> "$output_file"
        sleep 1
    done
}

# Monitor memory usage specifically
monitor_memory_usage() {
    local test_name="$1"
    local duration="$2"
    local output_file="$TEST_RESULTS_DIR/memory_usage_${test_name}.log"
    
    local end_time=$(($(date +%s) + duration))
    
    while [ $(date +%s) -lt $end_time ]; do
        echo "$(date): $(free -h)" >> "$output_file"
        
        # Check for memory limit breach
        local memory_usage=$(free | awk 'NR==2{printf "%.0f", $3/$2 * 100}')
        if [ "$memory_usage" -gt "$MAX_MEMORY_PERCENT" ]; then
            warn "Memory usage exceeded ${MAX_MEMORY_PERCENT}%: ${memory_usage}%"
        fi
        
        sleep 5
    done
}

# Collect metrics from Prometheus
collect_metrics() {
    local test_name="$1"
    local output_file="$TEST_RESULTS_DIR/metrics_${test_name}.json"
    
    log "Collecting metrics for $test_name..."
    
    # Define queries for key metrics
    local queries=(
        "icn:federation_health_ratio"
        "icn:job_success_rate_5m"
        "icn:federation_job_completion_rate"
        "icn:node_memory_usage_ratio"
        "icn:node_cpu_usage_ratio"
        "icn:job_throughput_5m"
        "icn:federation_total_mana"
        "rate(http_requests_total[5m])"
        "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))"
    )
    
    local metrics_json="{"
    local first=true
    
    for query in "${queries[@]}"; do
        if [ "$first" = false ]; then
            metrics_json+=","
        fi
        first=false
        
        local encoded_query=$(echo "$query" | sed 's/ /%20/g' | sed 's/:/%3A/g' | sed 's/\[/%5B/g' | sed 's/\]/%5D/g')
        local result=$(curl -s "http://localhost:$PROMETHEUS_PORT/api/v1/query?query=$encoded_query")
        
        metrics_json+="\"$(echo "$query" | sed 's/"/\\"/g')\":$result"
    done
    
    metrics_json+="}"
    
    echo "$metrics_json" > "$output_file"
    success "Metrics collected to $output_file"
}

# Generate comprehensive test report
generate_test_report() {
    local results_file="$1"
    local total_duration="$2"
    local report_file="$TEST_RESULTS_DIR/load_test_report_$(date +%Y%m%d_%H%M%S).html"
    
    log "Generating test report..."
    
    # Collect final metrics
    collect_metrics "final"
    
    # Generate HTML report
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>ICN Core Load Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .success { color: #4CAF50; }
        .warning { color: #FF9800; }
        .error { color: #F44336; }
        table { border-collapse: collapse; width: 100%; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .metric-card { 
            display: inline-block; 
            background: #f5f5f5; 
            padding: 15px; 
            margin: 10px; 
            border-radius: 5px; 
            min-width: 200px;
        }
    </style>
</head>
<body>
    <h1>ICN Core Load Test Report</h1>
    <div class="metric-card">
        <h3>Test Configuration</h3>
        <p><strong>Federation Size:</strong> $FEDERATION_SIZE nodes</p>
        <p><strong>Test Duration:</strong> $total_duration seconds</p>
        <p><strong>Job Submission Rate:</strong> $JOB_SUBMISSION_RATE jobs/sec</p>
        <p><strong>Concurrent Connections:</strong> $CONCURRENT_CONNECTIONS</p>
    </div>
EOF

    # Add performance summary
    if [ -f "$results_file" ]; then
        local total_jobs=$(jq '.total_jobs // 0' "$results_file" 2>/dev/null || echo "0")
        local success_rate=$(jq '.success_rate // 0' "$results_file" 2>/dev/null || echo "0")
        local avg_response_time=$(jq '.avg_response_time_ms // 0' "$results_file" 2>/dev/null || echo "0")
        
        cat >> "$report_file" << EOF
    <div class="metric-card">
        <h3>Performance Summary</h3>
        <p><strong>Total Jobs:</strong> $total_jobs</p>
        <p><strong>Success Rate:</strong> $success_rate%</p>
        <p><strong>Avg Response Time:</strong> ${avg_response_time}ms</p>
    </div>
EOF
    fi
    
    cat >> "$report_file" << EOF
    <h2>Test Results</h2>
    <p>Detailed test results and metrics are available in the following files:</p>
    <ul>
        <li><a href="$(basename "$results_file")">Load Test Results (JSON)</a></li>
        <li><a href="metrics_final.json">Final Metrics (JSON)</a></li>
    </ul>
    
    <h2>Performance Analysis</h2>
    <p>Generated on: $(date)</p>
</body>
</html>
EOF

    success "Test report generated: $report_file"
    
    # Check for performance regressions
    if [ "$FAIL_ON_PERFORMANCE_REGRESSION" = "true" ] && [ -n "$BASELINE_RESULTS_FILE" ]; then
        check_performance_regression "$results_file" "$BASELINE_RESULTS_FILE"
    fi
}

# Check for performance regressions
check_performance_regression() {
    local current_results="$1"
    local baseline_results="$2"
    
    log "Checking for performance regressions..."
    
    if [ ! -f "$baseline_results" ]; then
        warn "Baseline results file not found: $baseline_results"
        return 0
    fi
    
    local current_success_rate=$(jq '.success_rate // 0' "$current_results" 2>/dev/null || echo "0")
    local baseline_success_rate=$(jq '.success_rate // 0' "$baseline_results" 2>/dev/null || echo "0")
    
    local current_response_time=$(jq '.avg_response_time_ms // 0' "$current_results" 2>/dev/null || echo "0")
    local baseline_response_time=$(jq '.avg_response_time_ms // 0' "$baseline_results" 2>/dev/null || echo "0")
    
    local regression_detected=false
    
    # Check success rate regression (more than 5% decrease)
    if (( $(echo "$current_success_rate < $baseline_success_rate - 5" | bc -l 2>/dev/null || echo "0") )); then
        error "Success rate regression detected: ${current_success_rate}% vs ${baseline_success_rate}% baseline"
        regression_detected=true
    fi
    
    # Check response time regression (more than 20% increase)
    if (( $(echo "$current_response_time > $baseline_response_time * 1.2" | bc -l 2>/dev/null || echo "0") )); then
        error "Response time regression detected: ${current_response_time}ms vs ${baseline_response_time}ms baseline"
        regression_detected=true
    fi
    
    if [ "$regression_detected" = "true" ]; then
        error "Performance regression detected!"
        if [ "$CI_MODE" = "true" ]; then
            exit 1
        fi
    else
        success "No performance regressions detected"
    fi
}

# Cleanup function
cleanup() {
    log "Cleaning up test environment..."
    
    # Stop federation
    cd "$PROJECT_ROOT"
    ./scripts/cleanup-devnet.sh 2>/dev/null || true
    
    # Stop monitoring
    cd "$PROJECT_ROOT/monitoring"
    docker-compose -f docker-compose-monitoring.yml down 2>/dev/null || true
    
    # Kill any remaining background processes
    jobs -p | xargs -r kill 2>/dev/null || true
    
    success "Cleanup completed"
}

# Trap cleanup on exit
trap cleanup EXIT

# Main execution
main() {
    log "Starting ICN Core production load testing suite..."
    log "Configuration: $FEDERATION_SIZE nodes, ${TEST_DURATION}s duration, ${JOB_SUBMISSION_RATE} jobs/sec"
    
    check_dependencies
    setup_test_environment
    build_icn_core
    start_monitoring
    start_federation
    run_load_tests
    
    success "Load testing suite completed successfully!"
    
    if [ "$CI_MODE" = "true" ]; then
        log "Uploading results to CI artifacts..."
        # CI-specific artifact upload commands would go here
    fi
}

# Handle script arguments
case "${1:-run}" in
    "run")
        main
        ;;
    "cleanup")
        cleanup
        ;;
    "report-only")
        generate_test_report "${2:-$TEST_RESULTS_DIR/latest_results.json}" "0"
        ;;
    *)
        echo "Usage: $0 [run|cleanup|report-only]"
        exit 1
        ;;
esac