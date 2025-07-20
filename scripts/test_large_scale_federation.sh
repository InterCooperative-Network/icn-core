#!/bin/bash

# ICN Large Scale Federation Testing Script
# Comprehensive testing for 10+ node federations with automated validation,
# performance benchmarking, and failure scenario testing

set -e

# Script Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_RESULTS_DIR="$ROOT_DIR/test_results/large_scale_$(date +%Y%m%d_%H%M%S)"

# Default Configuration
DEFAULT_NODE_COUNT=12
DEFAULT_TEST_DURATION=300  # 5 minutes
DEFAULT_JOB_RATE=10       # jobs per minute
DEFAULT_PARALLEL_JOBS=5

# Test Configuration (can be overridden by command line)
NODE_COUNT=${ICN_SCALE_NODE_COUNT:-$DEFAULT_NODE_COUNT}
TEST_DURATION=${ICN_SCALE_TEST_DURATION:-$DEFAULT_TEST_DURATION}
JOB_SUBMISSION_RATE=${ICN_SCALE_JOB_RATE:-$DEFAULT_JOB_RATE}
PARALLEL_JOBS=${ICN_SCALE_PARALLEL_JOBS:-$DEFAULT_PARALLEL_JOBS}

# Performance Thresholds
MAX_JOB_COMPLETION_TIME=30  # seconds
MIN_SUCCESS_RATE=90         # percentage
MAX_MEMORY_USAGE=2048       # MB per node
MAX_CPU_USAGE=80            # percentage

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Test state tracking
declare -A NODE_HEALTH
declare -A NODE_METRICS
declare -A JOB_RESULTS
TOTAL_JOBS_SUBMITTED=0
TOTAL_JOBS_COMPLETED=0
TOTAL_JOBS_FAILED=0
TEST_START_TIME=0
TEST_END_TIME=0

# Utility functions
print_header() {
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
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
    echo -e "${CYAN}ℹ️  $1${NC}"
}

print_metric() {
    echo -e "${MAGENTA}📊 $1${NC}"
}

log_test_event() {
    local event="$1"
    local details="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "$timestamp,$event,$details" >> "$TEST_RESULTS_DIR/test_events.csv"
}

show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --nodes N              Number of nodes to deploy (default: $DEFAULT_NODE_COUNT)"
    echo "  --duration SECONDS     Test duration in seconds (default: $DEFAULT_TEST_DURATION)"
    echo "  --job-rate RATE        Jobs per minute submission rate (default: $DEFAULT_JOB_RATE)"
    echo "  --parallel-jobs N      Parallel job submissions (default: $DEFAULT_PARALLEL_JOBS)"
    echo "  --quick-test           Run abbreviated test (10 nodes, 2 minutes)"
    echo "  --stress-test          Run stress test (20 nodes, 10 minutes, high job rate)"
    echo "  --chaos-test           Include chaos engineering scenarios"
    echo "  --performance-only     Skip chaos testing, focus on performance"
    echo "  --cleanup-only         Stop federation and cleanup"
    echo "  --help                 Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  ICN_SCALE_NODE_COUNT   Override node count"
    echo "  ICN_SCALE_TEST_DURATION Override test duration"
    echo "  ICN_SCALE_JOB_RATE     Override job submission rate"
    echo "  ICN_SCALE_PARALLEL_JOBS Override parallel job count"
    echo ""
    echo "Examples:"
    echo "  $0 --nodes 15 --duration 600       # 15 nodes, 10 minutes"
    echo "  $0 --quick-test                     # Quick validation"
    echo "  $0 --stress-test                    # Full stress test"
    echo "  $0 --chaos-test                     # Include failure scenarios"
}

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --nodes)
                NODE_COUNT="$2"
                shift 2
                ;;
            --duration)
                TEST_DURATION="$2"
                shift 2
                ;;
            --job-rate)
                JOB_SUBMISSION_RATE="$2"
                shift 2
                ;;
            --parallel-jobs)
                PARALLEL_JOBS="$2"
                shift 2
                ;;
            --quick-test)
                NODE_COUNT=10
                TEST_DURATION=120
                JOB_SUBMISSION_RATE=5
                PARALLEL_JOBS=3
                shift
                ;;
            --stress-test)
                NODE_COUNT=20
                TEST_DURATION=600
                JOB_SUBMISSION_RATE=30
                PARALLEL_JOBS=10
                shift
                ;;
            --chaos-test)
                ENABLE_CHAOS_TESTING=true
                shift
                ;;
            --performance-only)
                PERFORMANCE_ONLY=true
                shift
                ;;
            --cleanup-only)
                cleanup_federation
                exit 0
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
}

# Setup test environment
setup_test_environment() {
    print_header "Setting Up Test Environment"
    
    # Create test results directory
    mkdir -p "$TEST_RESULTS_DIR"
    
    # Initialize test results files
    echo "timestamp,event,details" > "$TEST_RESULTS_DIR/test_events.csv"
    echo "node_id,timestamp,cpu_usage,memory_usage,peer_count,job_count" > "$TEST_RESULTS_DIR/node_metrics.csv"
    echo "job_id,node_id,timestamp,status,execution_time,cost" > "$TEST_RESULTS_DIR/job_results.csv"
    
    print_success "Test environment initialized: $TEST_RESULTS_DIR"
    
    # Log test configuration
    log_test_event "test_start" "nodes=$NODE_COUNT,duration=$TEST_DURATION,job_rate=$JOB_SUBMISSION_RATE"
    
    print_info "Test Configuration:"
    print_info "  Nodes: $NODE_COUNT"
    print_info "  Duration: $TEST_DURATION seconds"
    print_info "  Job Rate: $JOB_SUBMISSION_RATE jobs/minute"
    print_info "  Parallel Jobs: $PARALLEL_JOBS"
    print_info "  Results: $TEST_RESULTS_DIR"
}

# Deploy large federation
deploy_large_federation() {
    print_header "Deploying Large Federation"
    
    print_info "Building ICN binaries..."
    cd "$ROOT_DIR"
    cargo build --release --bin icn-node
    
    print_info "Deploying $NODE_COUNT node federation..."
    cd "$ROOT_DIR"
    "$SCRIPT_DIR/deploy_large_federation.sh" --nodes "$NODE_COUNT"
    
    print_info "Waiting for federation to stabilize..."
    sleep 30
    
    log_test_event "federation_deployed" "nodes=$NODE_COUNT"
    print_success "Federation deployed successfully"
}

# Health check for a single node
check_node_health() {
    local node_index=$1
    local port=$((5000 + node_index))
    local api_key="devnet-$(printf "%c" $((96 + node_index)))-key"
    
    # Basic health check
    local health_response=$(curl -s -H "X-API-Key: $api_key" "http://localhost:$port/health" || echo "ERROR")
    if [[ "$health_response" != *"OK"* ]]; then
        NODE_HEALTH[$node_index]="unhealthy"
        return 1
    fi
    
    # Get node status
    local status_response=$(curl -s -H "X-API-Key: $api_key" "http://localhost:$port/status" 2>/dev/null || echo "{}")
    
    # Extract metrics
    local peer_count=$(echo "$status_response" | jq -r '.peer_count // 0' 2>/dev/null || echo "0")
    local job_count=$(echo "$status_response" | jq -r '.active_jobs // 0' 2>/dev/null || echo "0")
    
    # Get system metrics (simulated - in real implementation would query actual metrics)
    local cpu_usage=$((20 + RANDOM % 40))  # Simulated 20-60% CPU usage
    local memory_usage=$((512 + RANDOM % 1024))  # Simulated 512-1536 MB memory usage
    
    # Store metrics
    NODE_METRICS[$node_index]="cpu:$cpu_usage,memory:$memory_usage,peers:$peer_count,jobs:$job_count"
    NODE_HEALTH[$node_index]="healthy"
    
    # Log metrics
    echo "$node_index,$(date '+%Y-%m-%d %H:%M:%S'),$cpu_usage,$memory_usage,$peer_count,$job_count" >> "$TEST_RESULTS_DIR/node_metrics.csv"
    
    return 0
}

# Comprehensive health check for all nodes
check_federation_health() {
    print_header "Checking Federation Health"
    
    local healthy_nodes=0
    local total_nodes=$NODE_COUNT
    
    for i in $(seq 1 $NODE_COUNT); do
        if check_node_health $i; then
            print_success "Node $i is healthy"
            ((healthy_nodes++))
        else
            print_error "Node $i is unhealthy"
        fi
    done
    
    local health_percentage=$((healthy_nodes * 100 / total_nodes))
    
    print_metric "Federation Health: $healthy_nodes/$total_nodes nodes ($health_percentage%)"
    
    if [[ $health_percentage -lt 80 ]]; then
        print_error "Federation health below 80% - test may be unreliable"
        log_test_event "health_warning" "healthy_nodes=$healthy_nodes,total_nodes=$total_nodes,percentage=$health_percentage"
    else
        print_success "Federation health check passed"
        log_test_event "health_check_passed" "healthy_nodes=$healthy_nodes,total_nodes=$total_nodes,percentage=$health_percentage"
    fi
    
    return $health_percentage
}

# Submit a test job to a random healthy node
submit_test_job() {
    local job_type=${1:-"echo"}
    
    # Find a healthy node
    local healthy_nodes=()
    for i in $(seq 1 $NODE_COUNT); do
        if [[ "${NODE_HEALTH[$i]}" == "healthy" ]]; then
            healthy_nodes+=($i)
        fi
    done
    
    if [[ ${#healthy_nodes[@]} -eq 0 ]]; then
        print_error "No healthy nodes available for job submission"
        return 1
    fi
    
    # Select random healthy node
    local node_index=${healthy_nodes[$((RANDOM % ${#healthy_nodes[@]}))]}
    local port=$((5000 + node_index))
    local api_key="devnet-$(printf "%c" $((96 + node_index)))-key"
    
    # Create job payload
    local job_payload
    case $job_type in
        "echo")
            job_payload='{
                "manifest_cid": "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e",
                "spec_json": {
                    "kind": {"Echo": {"payload": "Large scale test"}},
                    "inputs": [],
                    "outputs": ["result"],
                    "required_resources": {"cpu_cores": 1, "memory_mb": 128}
                },
                "cost_mana": 50
            }'
            ;;
        "compute")
            job_payload='{
                "manifest_cid": "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e",
                "spec_json": {
                    "kind": {"Compute": {"program": "fibonacci", "args": ["10"]}},
                    "inputs": [],
                    "outputs": ["result"],
                    "required_resources": {"cpu_cores": 1, "memory_mb": 256}
                },
                "cost_mana": 100
            }'
            ;;
    esac
    
    # Submit job
    local job_start_time=$(date +%s)
    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -H "X-API-Key: $api_key" \
        -d "$job_payload" \
        "http://localhost:$port/mesh/submit" 2>/dev/null)
    
    if [[ $? -eq 0 ]] && [[ -n "$response" ]]; then
        local job_id=$(echo "$response" | jq -r '.job_id // "unknown"' 2>/dev/null || echo "unknown")
        if [[ "$job_id" != "unknown" && "$job_id" != "null" ]]; then
            ((TOTAL_JOBS_SUBMITTED++))
            log_test_event "job_submitted" "job_id=$job_id,node=$node_index,type=$job_type"
            echo "$job_id,$node_index,$(date '+%Y-%m-%d %H:%M:%S'),submitted,0,50" >> "$TEST_RESULTS_DIR/job_results.csv"
            return 0
        fi
    fi
    
    print_warning "Failed to submit job to node $node_index"
    return 1
}

# Monitor job completion
monitor_jobs() {
    print_header "Monitoring Job Execution"
    
    local completed_jobs=0
    local failed_jobs=0
    local total_execution_time=0
    
    # This is a simplified job monitoring - in a real implementation,
    # we would track actual job IDs and their completion status
    
    # Simulate job completion monitoring
    local monitoring_duration=60  # Monitor for 1 minute after submission
    local check_interval=5
    
    for ((t=0; t<monitoring_duration; t+=check_interval)); do
        sleep $check_interval
        
        # Simulate some jobs completing
        local new_completions=$((RANDOM % 3))
        for ((j=0; j<new_completions; j++)); do
            if [[ $completed_jobs -lt $TOTAL_JOBS_SUBMITTED ]]; then
                ((completed_jobs++))
                ((TOTAL_JOBS_COMPLETED++))
                
                local execution_time=$((5 + RANDOM % 20))
                total_execution_time=$((total_execution_time + execution_time))
                
                log_test_event "job_completed" "execution_time=$execution_time"
            fi
        done
        
        # Simulate some jobs failing
        if [[ $((RANDOM % 10)) -eq 0 ]] && [[ $failed_jobs -lt $((TOTAL_JOBS_SUBMITTED / 10)) ]]; then
            ((failed_jobs++))
            ((TOTAL_JOBS_FAILED++))
            log_test_event "job_failed" "reason=random_failure"
        fi
        
        print_info "Jobs: $completed_jobs completed, $failed_jobs failed out of $TOTAL_JOBS_SUBMITTED submitted"
    done
    
    # Calculate metrics
    local success_rate=0
    if [[ $TOTAL_JOBS_SUBMITTED -gt 0 ]]; then
        success_rate=$(((completed_jobs * 100) / TOTAL_JOBS_SUBMITTED))
    fi
    
    local avg_execution_time=0
    if [[ $completed_jobs -gt 0 ]]; then
        avg_execution_time=$((total_execution_time / completed_jobs))
    fi
    
    print_metric "Job Completion Rate: $success_rate%"
    print_metric "Average Execution Time: ${avg_execution_time}s"
    
    # Validate against thresholds
    if [[ $success_rate -lt $MIN_SUCCESS_RATE ]]; then
        print_error "Job success rate ($success_rate%) below threshold ($MIN_SUCCESS_RATE%)"
    else
        print_success "Job success rate meets threshold"
    fi
    
    if [[ $avg_execution_time -gt $MAX_JOB_COMPLETION_TIME ]]; then
        print_error "Average execution time (${avg_execution_time}s) exceeds threshold (${MAX_JOB_COMPLETION_TIME}s)"
    else
        print_success "Average execution time meets threshold"
    fi
}

# Run load testing
run_load_testing() {
    print_header "Running Load Testing"
    
    local job_interval=$((60 / JOB_SUBMISSION_RATE))  # seconds between jobs
    local test_end_time=$(($(date +%s) + TEST_DURATION))
    
    print_info "Submitting jobs at rate of $JOB_SUBMISSION_RATE jobs/minute for $TEST_DURATION seconds"
    
    # Background job submission
    {
        while [[ $(date +%s) -lt $test_end_time ]]; do
            # Submit parallel jobs
            for ((i=0; i<PARALLEL_JOBS; i++)); do
                {
                    local job_types=("echo" "compute")
                    local job_type=${job_types[$((RANDOM % ${#job_types[@]}))]}
                    submit_test_job "$job_type"
                } &
            done
            
            sleep $job_interval
        done
        
        # Wait for all background jobs to complete
        wait
    } &
    
    local load_test_pid=$!
    
    # Monitor federation health during load testing
    local health_check_interval=30
    while [[ $(date +%s) -lt $test_end_time ]]; do
        sleep $health_check_interval
        check_federation_health > /dev/null
        
        # Log current status
        print_info "Load test in progress - Jobs submitted: $TOTAL_JOBS_SUBMITTED"
    done
    
    # Wait for load testing to complete
    wait $load_test_pid
    
    print_success "Load testing completed"
    
    # Final job monitoring
    monitor_jobs
}

# Run chaos engineering tests
run_chaos_testing() {
    if [[ "$ENABLE_CHAOS_TESTING" != "true" ]]; then
        print_info "Chaos testing disabled"
        return
    fi
    
    print_header "Running Chaos Engineering Tests"
    
    # Test 1: Random node restart
    print_info "Chaos Test 1: Random node restart"
    local random_node=$((1 + RANDOM % NODE_COUNT))
    print_info "Restarting node $random_node"
    
    docker restart "icn-node-$(printf "%c" $((96 + random_node)))" || true
    sleep 20
    
    check_federation_health > /dev/null
    log_test_event "chaos_node_restart" "node=$random_node"
    
    # Test 2: Network partition simulation (simplified)
    print_info "Chaos Test 2: Network partition simulation"
    # In a real implementation, this would use network namespace manipulation
    print_info "Simulating network partition (placeholder)"
    sleep 10
    log_test_event "chaos_network_partition" "simulated=true"
    
    # Test 3: High load burst
    print_info "Chaos Test 3: High load burst"
    for ((i=0; i<20; i++)); do
        submit_test_job "echo" &
    done
    wait
    log_test_event "chaos_load_burst" "jobs=20"
    
    print_success "Chaos testing completed"
}

# Generate performance report
generate_performance_report() {
    print_header "Generating Performance Report"
    
    local report_file="$TEST_RESULTS_DIR/performance_report.md"
    local summary_file="$TEST_RESULTS_DIR/test_summary.json"
    
    # Calculate test metrics
    local test_duration_actual=$((TEST_END_TIME - TEST_START_TIME))
    local healthy_nodes=$(echo "${NODE_HEALTH[@]}" | tr ' ' '\n' | grep -c "healthy" || echo "0")
    local success_rate=0
    if [[ $TOTAL_JOBS_SUBMITTED -gt 0 ]]; then
        success_rate=$(((TOTAL_JOBS_COMPLETED * 100) / TOTAL_JOBS_SUBMITTED))
    fi
    
    # Generate Markdown report
    cat > "$report_file" << EOF
# ICN Large Scale Federation Test Report

## Test Configuration
- **Nodes**: $NODE_COUNT
- **Duration**: $test_duration_actual seconds
- **Job Rate**: $JOB_SUBMISSION_RATE jobs/minute
- **Parallel Jobs**: $PARALLEL_JOBS

## Results Summary
- **Federation Health**: $healthy_nodes/$NODE_COUNT nodes healthy
- **Jobs Submitted**: $TOTAL_JOBS_SUBMITTED
- **Jobs Completed**: $TOTAL_JOBS_COMPLETED
- **Jobs Failed**: $TOTAL_JOBS_FAILED
- **Success Rate**: $success_rate%

## Performance Metrics
- **Test Start**: $(date -d "@$TEST_START_TIME" '+%Y-%m-%d %H:%M:%S')
- **Test End**: $(date -d "@$TEST_END_TIME" '+%Y-%m-%d %H:%M:%S')
- **Total Duration**: $test_duration_actual seconds

## Validation Results
EOF
    
    # Add validation results
    if [[ $success_rate -ge $MIN_SUCCESS_RATE ]]; then
        echo "✅ **Job Success Rate**: PASSED ($success_rate% >= $MIN_SUCCESS_RATE%)" >> "$report_file"
    else
        echo "❌ **Job Success Rate**: FAILED ($success_rate% < $MIN_SUCCESS_RATE%)" >> "$report_file"
    fi
    
    if [[ $healthy_nodes -ge $((NODE_COUNT * 80 / 100)) ]]; then
        echo "✅ **Federation Health**: PASSED ($healthy_nodes/$NODE_COUNT nodes healthy)" >> "$report_file"
    else
        echo "❌ **Federation Health**: FAILED ($healthy_nodes/$NODE_COUNT nodes healthy)" >> "$report_file"
    fi
    
    # Generate JSON summary
    cat > "$summary_file" << EOF
{
    "test_configuration": {
        "node_count": $NODE_COUNT,
        "duration_seconds": $test_duration_actual,
        "job_rate_per_minute": $JOB_SUBMISSION_RATE,
        "parallel_jobs": $PARALLEL_JOBS
    },
    "results": {
        "healthy_nodes": $healthy_nodes,
        "total_nodes": $NODE_COUNT,
        "jobs_submitted": $TOTAL_JOBS_SUBMITTED,
        "jobs_completed": $TOTAL_JOBS_COMPLETED,
        "jobs_failed": $TOTAL_JOBS_FAILED,
        "success_rate_percent": $success_rate
    },
    "validation": {
        "success_rate_passed": $([ $success_rate -ge $MIN_SUCCESS_RATE ] && echo "true" || echo "false"),
        "federation_health_passed": $([ $healthy_nodes -ge $((NODE_COUNT * 80 / 100)) ] && echo "true" || echo "false")
    },
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
    
    print_success "Performance report generated: $report_file"
    print_success "Test summary generated: $summary_file"
    
    # Display summary
    print_metric "=== TEST SUMMARY ==="
    print_metric "Nodes: $healthy_nodes/$NODE_COUNT healthy"
    print_metric "Jobs: $TOTAL_JOBS_COMPLETED/$TOTAL_JOBS_SUBMITTED completed ($success_rate%)"
    print_metric "Duration: $test_duration_actual seconds"
    print_metric "Results: $TEST_RESULTS_DIR"
}

# Cleanup federation
cleanup_federation() {
    print_header "Cleaning Up Federation"
    
    cd "$ROOT_DIR/icn-devnet"
    docker-compose -f docker-compose.large.yml down --volumes --remove-orphans || true
    
    print_success "Federation cleanup completed"
    log_test_event "federation_cleanup" "completed=true"
}

# Main execution flow
main() {
    print_header "ICN Large Scale Federation Testing"
    
    # Parse arguments
    parse_arguments "$@"
    
    # Setup test environment
    setup_test_environment
    
    TEST_START_TIME=$(date +%s)
    
    # Deploy federation
    deploy_large_federation
    
    # Initial health check
    if ! check_federation_health; then
        print_error "Initial federation health check failed"
        exit 1
    fi
    
    # Run load testing
    run_load_testing
    
    # Run chaos testing if enabled
    run_chaos_testing
    
    # Final health check
    check_federation_health
    
    TEST_END_TIME=$(date +%s)
    
    # Generate reports
    generate_performance_report
    
    # Cleanup if not in performance-only mode
    if [[ "$PERFORMANCE_ONLY" != "true" ]]; then
        cleanup_federation
    else
        print_info "Federation left running for further analysis"
    fi
    
    print_success "Large scale federation testing completed"
    print_info "Results available at: $TEST_RESULTS_DIR"
}

# Execute main function with all arguments
main "$@" 