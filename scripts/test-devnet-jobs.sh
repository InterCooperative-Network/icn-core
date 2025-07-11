#!/bin/bash

# ICN Devnet Job Testing Script
# Tests job submission, tracking, and validation across all nodes

set -e

# Configuration
NODES=(
    "http://localhost:5001"
    "http://localhost:5002"
    "http://localhost:5003"
    "http://localhost:5004"
    "http://localhost:5005"
    "http://localhost:5006"
    "http://localhost:5007"
    "http://localhost:5008"
    "http://localhost:5009"
    "http://localhost:5010"
)

API_KEYS=(
    "devnet-a-key"
    "devnet-b-key"
    "devnet-c-key"
    "devnet-d-key"
    "devnet-e-key"
    "devnet-f-key"
    "devnet-g-key"
    "devnet-h-key"
    "devnet-i-key"
    "devnet-j-key"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Valid test CIDs for manifests (working CID verified)
TEST_CIDS=(
    "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e"
    "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e"
    "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e"
    "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e"
    "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e"
)

# Test job specifications
declare -A JOB_SPECS

JOB_SPECS["echo"]='
{
  "kind": {"Echo": {"payload": "Hello ICN Devnet!"}},
  "inputs": [],
  "outputs": [],
  "required_resources": {"cpu_cores": 0, "memory_mb": 0}
}'

JOB_SPECS["compute"]='
{
  "kind": {"Compute": {"program": "fibonacci", "args": ["10"]}},
  "inputs": [],
  "outputs": [],
  "required_resources": {"cpu_cores": 1, "memory_mb": 128}
}'

JOB_SPECS["transform"]='
{
  "kind": {"Transform": {"input_format": "json", "output_format": "csv"}},
  "inputs": [{"name": "data", "format": "json"}],
  "outputs": [{"name": "result", "format": "csv"}],
  "required_resources": {"cpu_cores": 1, "memory_mb": 256}
}'

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Generate a valid CID for testing
generate_test_cid() {
    local index=$(($RANDOM % ${#TEST_CIDS[@]}))
    echo "${TEST_CIDS[$index]}"
}

# Submit a job to a specific node
submit_job() {
    local node_url=$1
    local api_key=$2
    local job_type=$3
    local manifest_cid=$4
    
    local spec_json="${JOB_SPECS[$job_type]}"
    
    curl -s -X POST "$node_url/mesh/submit" \
        -H 'Content-Type: application/json' \
        -H "x-api-key: $api_key" \
        -d "{
            \"manifest_cid\": \"$manifest_cid\",
            \"spec_json\": $spec_json,
            \"cost_mana\": 50
        }" 2>/dev/null
}

# Check job status
check_job_status() {
    local node_url=$1
    local api_key=$2
    local job_id=$3
    
    curl -s -X GET "$node_url/mesh/jobs/$job_id" \
        -H "x-api-key: $api_key" 2>/dev/null
}

# Test node health
test_node_health() {
    local node_url=$1
    local api_key=$2
    
    local health_response=$(curl -s -X GET "$node_url/health" \
        -H "x-api-key: $api_key" 2>/dev/null)
    
    if echo "$health_response" | grep -q '"status":"OK"'; then
        return 0
    else
        return 1
    fi
}

# Test network connectivity
test_network_connectivity() {
    local node_url=$1
    local api_key=$2
    
    local peers_response=$(curl -s -X GET "$node_url/network/peers" \
        -H "x-api-key: $api_key" 2>/dev/null)
    
    local peer_count=$(echo "$peers_response" | jq '.peers | length' 2>/dev/null || echo "0")
    echo "$peer_count"
}

# Monitor job until completion or timeout
monitor_job() {
    local node_url=$1
    local api_key=$2
    local job_id=$3
    local timeout_seconds=${4:-60}
    
    local start_time=$(date +%s)
    local end_time=$((start_time + timeout_seconds))
    
    while [ $(date +%s) -lt $end_time ]; do
        local status_response=$(check_job_status "$node_url" "$api_key" "$job_id")
        local status=$(echo "$status_response" | jq -r '.status' 2>/dev/null || echo "unknown")
        
        case "$status" in
            "completed")
                print_success "Job $job_id completed successfully"
                echo "$status_response" | jq '.'
                return 0
                ;;
            "failed")
                print_error "Job $job_id failed"
                echo "$status_response" | jq '.'
                return 1
                ;;
            "pending"|"bidding"|"assigned"|"running")
                print_info "Job $job_id status: $status"
                ;;
            *)
                print_warning "Job $job_id has unknown status: $status"
                ;;
        esac
        
        sleep 5
    done
    
    print_warning "Job $job_id monitoring timed out after ${timeout_seconds}s"
    return 2
}

# Test job submission and tracking
test_job_submission() {
    local node_index=$1
    local job_type=$2
    
    local node_url="${NODES[$node_index]}"
    local api_key="${API_KEYS[$node_index]}"
    local manifest_cid=$(generate_test_cid)
    
    print_info "Testing $job_type job submission on node $((node_index + 1))"
    print_info "Node: $node_url"
    print_info "Manifest CID: $manifest_cid"
    
    # Submit job
    local submit_response=$(submit_job "$node_url" "$api_key" "$job_type" "$manifest_cid")
    
    if echo "$submit_response" | grep -q '"job_id"'; then
        local job_id=$(echo "$submit_response" | jq -r '.job_id')
        print_success "Job submitted successfully: $job_id"
        
        # Monitor job
        monitor_job "$node_url" "$api_key" "$job_id" 120
        return $?
    else
        print_error "Job submission failed"
        echo "$submit_response" | jq '.' 2>/dev/null || echo "$submit_response"
        return 1
    fi
}

# Test cross-node job visibility
test_cross_node_visibility() {
    local submit_node=$1
    local query_node=$2
    local job_id=$3
    
    local submit_url="${NODES[$submit_node]}"
    local query_url="${NODES[$query_node]}"
    local query_key="${API_KEYS[$query_node]}"
    
    print_info "Testing job visibility: submitted on node $((submit_node + 1)), querying on node $((query_node + 1))"
    
    sleep 2  # Allow time for network propagation
    
    local query_response=$(check_job_status "$query_url" "$query_key" "$job_id")
    
    if echo "$query_response" | grep -q '"job_id"'; then
        print_success "Job $job_id visible on node $((query_node + 1))"
        return 0
    else
        print_warning "Job $job_id not visible on node $((query_node + 1))"
        return 1
    fi
}

# Main test runner
main() {
    print_header "ICN Devnet Job Testing Suite"
    
    # Test 1: Node Health Checks
    print_header "Testing Node Health"
    local healthy_nodes=0
    for i in "${!NODES[@]}"; do
        if test_node_health "${NODES[$i]}" "${API_KEYS[$i]}"; then
            print_success "Node $((i + 1)) is healthy"
            ((healthy_nodes++))
        else
            print_error "Node $((i + 1)) is unhealthy"
        fi
    done
    
    print_info "Healthy nodes: $healthy_nodes/${#NODES[@]}"
    
    if [ $healthy_nodes -eq 0 ]; then
        print_error "No healthy nodes found. Exiting."
        exit 1
    fi
    
    # Test 2: Network Connectivity
    print_header "Testing Network Connectivity"
    for i in "${!NODES[@]}"; do
        local peer_count=$(test_network_connectivity "${NODES[$i]}" "${API_KEYS[$i]}")
        if [ "$peer_count" -gt 0 ]; then
            print_success "Node $((i + 1)) has $peer_count peers"
        else
            print_warning "Node $((i + 1)) has no peers"
        fi
    done
    
    # Test 3: Job Submission Tests
    print_header "Testing Job Submissions"
    
    # Test different job types on different nodes
    local test_cases=(
        "0:echo"
        "1:compute"
        "2:transform"
        "3:echo"
        "4:compute"
    )
    
    local successful_jobs=0
    local job_ids=()
    
    for test_case in "${test_cases[@]}"; do
        local node_index="${test_case%:*}"
        local job_type="${test_case#*:}"
        
        if test_job_submission "$node_index" "$job_type"; then
            ((successful_jobs++))
        fi
        
        echo ""  # Add spacing between tests
    done
    
    print_info "Successful job submissions: $successful_jobs/${#test_cases[@]}"
    
    # Test 4: Cross-Node Visibility (if we have multiple healthy nodes)
    if [ $healthy_nodes -gt 1 ]; then
        print_header "Testing Cross-Node Job Visibility"
        
        # Submit a job on node 0 and check visibility on node 1
        local manifest_cid=$(generate_test_cid)
        local submit_response=$(submit_job "${NODES[0]}" "${API_KEYS[0]}" "echo" "$manifest_cid")
        
        if echo "$submit_response" | grep -q '"job_id"'; then
            local job_id=$(echo "$submit_response" | jq -r '.job_id')
            print_success "Test job submitted: $job_id"
            
            test_cross_node_visibility 0 1 "$job_id"
        else
            print_error "Failed to submit test job for cross-node visibility test"
        fi
    fi
    
    # Test 5: Performance Summary
    print_header "Performance Summary"
    
    local total_jobs=${#test_cases[@]}
    local success_rate=$((successful_jobs * 100 / total_jobs))
    
    print_info "Total jobs submitted: $total_jobs"
    print_info "Successful jobs: $successful_jobs"
    print_info "Success rate: $success_rate%"
    
    if [ $success_rate -ge 80 ]; then
        print_success "Devnet is performing well!"
    elif [ $success_rate -ge 60 ]; then
        print_warning "Devnet performance is moderate"
    else
        print_error "Devnet performance is poor"
    fi
    
    print_header "Test Complete"
}

# Handle command line arguments
case "${1:-all}" in
    "health")
        print_header "Node Health Check"
        for i in "${!NODES[@]}"; do
            if test_node_health "${NODES[$i]}" "${API_KEYS[$i]}"; then
                print_success "Node $((i + 1)) is healthy"
            else
                print_error "Node $((i + 1)) is unhealthy"
            fi
        done
        ;;
    "submit")
        job_type="${2:-echo}"
        node_index="${3:-0}"
        test_job_submission "$node_index" "$job_type"
        ;;
    "monitor")
        job_id="$2"
        node_index="${3:-0}"
        if [ -z "$job_id" ]; then
            print_error "Job ID required for monitoring"
            exit 1
        fi
        monitor_job "${NODES[$node_index]}" "${API_KEYS[$node_index]}" "$job_id"
        ;;
    "all"|*)
        main
        ;;
esac 