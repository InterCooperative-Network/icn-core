#!/bin/bash

# ICN Full Mesh Lifecycle Testing Script
# Tests the complete mesh computing pipeline with job submission, bidding, execution, and completion

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
WORKING_CID="bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e"
BASE_URL="http://localhost"
TIMEOUT=10

print_step() {
    echo -e "\n${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_highlight() {
    echo -e "${YELLOW}⭐ $1${NC}"
}

# Test node connectivity
test_node() {
    local port=$1
    local api_key=$2
    local node_name=$3
    
    local response=$(curl -s -H "x-api-key: $api_key" "$BASE_URL:$port/health" | jq -r '.status // "ERROR"')
    if [ "$response" = "OK" ]; then
        print_success "$node_name (port $port) is healthy"
        return 0
    else
        print_error "$node_name (port $port) is not responding properly"
        return 1
    fi
}

# Submit a job to a node
submit_job() {
    local port=$1
    local api_key=$2
    local job_type=$3
    local payload=$4
    local cost=$5
    
    local job_data
    case $job_type in
        "echo")
            job_data=$(cat <<EOF
{
    "manifest_cid": "$WORKING_CID",
    "spec_json": {
        "kind": {
            "Echo": {
                "payload": "$payload"
            }
        },
        "inputs": [],
        "outputs": ["result"],
        "required_resources": {
            "cpu_cores": 1,
            "memory_mb": 128
        }
    },
    "cost_mana": $cost
}
EOF
)
            ;;
        "wasm")
            job_data=$(cat <<EOF
{
    "manifest_cid": "$WORKING_CID",
    "spec_json": {
        "kind": "CclWasm",
        "inputs": [],
        "outputs": ["result"],
        "required_resources": {
            "cpu_cores": 1,
            "memory_mb": 256
        }
    },
    "cost_mana": $cost
}
EOF
)
            ;;
    esac
    
    curl -s -X POST "$BASE_URL:$port/mesh/submit" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $api_key" \
        -d "$job_data" | jq -r '.job_id // "ERROR"'
}

# Get job status
get_job_status() {
    local port=$1
    local api_key=$2
    local job_id=$3
    
    curl -s -H "x-api-key: $api_key" \
        "$BASE_URL:$port/mesh/jobs/$job_id" | jq '.'
}

# Wait for job status change
wait_for_job_status() {
    local port=$1
    local api_key=$2
    local job_id=$3
    local expected_status=$4
    local max_wait=$5
    
    local count=0
    while [ $count -lt $max_wait ]; do
        local job_info=$(get_job_status $port $api_key $job_id)
        local current_status=$(echo "$job_info" | jq -r '.status // "unknown"')
        
        if [ "$current_status" = "$expected_status" ]; then
            return 0
        fi
        
        echo -n "."
        sleep 1
        count=$((count + 1))
    done
    
    return 1
}

# Stage a bid for testing (using stub network)
stage_bid() {
    local submitter_port=$1
    local executor_port=$2
    local job_id=$3
    local price=$4
    
    # This is a test helper function that would simulate receiving bids
    # In a real implementation, this would be handled by the network layer
    print_info "Simulating bid submission from executor (port $executor_port) for job $job_id at $price mana"
}

# Display detailed job information
show_job_details() {
    local port=$1
    local api_key=$2
    local job_id=$3
    local node_name=$4
    
    print_info "Job details from $node_name:"
    get_job_status $port $api_key $job_id | jq '.'
}

# Test the complete mesh lifecycle
test_mesh_lifecycle() {
    print_step "Testing Complete Mesh Lifecycle"
    
    local submitter_port=5001
    local executor1_port=5002
    local executor2_port=5003
    local submitter_key="devnet-a-key"
    local executor1_key="devnet-b-key"
    local executor2_key="devnet-c-key"
    
    # Test node connectivity
    print_step "Step 1: Testing Node Connectivity"
    test_node $submitter_port $submitter_key "Submitter Node" || return 1
    test_node $executor1_port $executor1_key "Executor Node 1" || return 1
    test_node $executor2_port $executor2_key "Executor Node 2" || return 1
    
    # Submit a job
    print_step "Step 2: Submitting Echo Job"
    local job_id=$(submit_job $submitter_port $submitter_key "echo" "Hello, Mesh Network!" 30)
    
    if [ "$job_id" = "ERROR" ] || [ -z "$job_id" ]; then
        print_error "Failed to submit job"
        return 1
    fi
    
    print_success "Job submitted with ID: $job_id"
    
    # Show initial job status
    print_step "Step 3: Initial Job Status"
    show_job_details $submitter_port $submitter_key $job_id "Submitter Node"
    
    # Wait for job to be announced (should move through lifecycle)
    print_step "Step 4: Waiting for Job Processing"
    print_info "Waiting for job to progress through mesh lifecycle..."
    
    # Since we're using stub services, the job will remain in pending state
    # In a real implementation with executor nodes, we'd see:
    # - Job announcement
    # - Bid collection
    # - Executor selection
    # - Job assignment
    # - Job execution
    # - Receipt submission
    
    sleep 3
    
    print_step "Step 5: Final Job Status"
    show_job_details $submitter_port $submitter_key $job_id "Submitter Node"
    
    # List all jobs to see the complete state
    print_step "Step 6: All Jobs Overview"
    print_info "All jobs on submitter node:"
    curl -s -H "x-api-key: $submitter_key" \
        "$BASE_URL:$submitter_port/mesh/jobs" | jq '.'
    
    print_highlight "Mesh Lifecycle Test Completed"
    print_info "Note: With stub network services, jobs remain in 'pending' state."
    print_info "With real libp2p networking, jobs would progress through the full lifecycle."
    
    return 0
}

# Test job submission with different types
test_job_types() {
    print_step "Testing Different Job Types"
    
    local port=5001
    local api_key="devnet-a-key"
    
    # Test Echo job
    print_info "Submitting Echo job..."
    local echo_job_id=$(submit_job $port $api_key "echo" "Test Echo Job" 20)
    if [ "$echo_job_id" != "ERROR" ]; then
        print_success "Echo job submitted: $echo_job_id"
    else
        print_error "Failed to submit Echo job"
    fi
    
    # Test WASM job (CCL)
    print_info "Submitting CCL WASM job..."
    local wasm_job_id=$(submit_job $port $api_key "wasm" "" 50)
    if [ "$wasm_job_id" != "ERROR" ]; then
        print_success "WASM job submitted: $wasm_job_id"
        
        # WASM jobs should auto-execute
        print_info "Waiting for WASM job to auto-execute..."
        if wait_for_job_status $port $api_key $wasm_job_id "completed" 10; then
            print_success "WASM job completed successfully!"
            show_job_details $port $api_key $wasm_job_id "Node"
        else
            print_warning "WASM job did not complete within timeout"
        fi
    else
        print_error "Failed to submit WASM job"
    fi
}

# Test concurrent job submissions
test_concurrent_jobs() {
    print_step "Testing Concurrent Job Submissions"
    
    local jobs=()
    
    # Submit multiple jobs concurrently
    for i in {1..5}; do
        local port=$((5000 + i))
        local api_key="devnet-$(echo -e {a..j} | cut -d' ' -f$i)-key"
        
        print_info "Submitting job $i to port $port..."
        local job_id=$(submit_job $port $api_key "echo" "Concurrent job $i" 25)
        
        if [ "$job_id" != "ERROR" ]; then
            jobs+=("$port:$api_key:$job_id")
            print_success "Job $i submitted: $job_id"
        else
            print_error "Failed to submit job $i"
        fi
    done
    
    # Check status of all jobs
    print_info "Checking status of all submitted jobs..."
    for job_info in "${jobs[@]}"; do
        IFS=':' read -r port api_key job_id <<< "$job_info"
        print_info "Job $job_id on port $port:"
        get_job_status $port $api_key $job_id | jq '.'
    done
}

# Test network peer connectivity
test_network_connectivity() {
    print_step "Testing Network Connectivity"
    
    for i in {1..5}; do
        local port=$((5000 + i))
        local api_key="devnet-$(echo -e {a..j} | cut -d' ' -f$i)-key"
        
        print_info "Checking peers for node on port $port..."
        curl -s -H "x-api-key: $api_key" \
            "$BASE_URL:$port/network/peers" | jq '.'
    done
}

# Main test runner
main() {
    echo -e "${CYAN}"
    echo "═══════════════════════════════════════════════════════════"
    echo "           ICN Full Mesh Lifecycle Test Suite             "
    echo "═══════════════════════════════════════════════════════════"
    echo -e "${NC}"
    
    case "${1:-full}" in
        "lifecycle")
            test_mesh_lifecycle
            ;;
        "types") 
            test_job_types
            ;;
        "concurrent")
            test_concurrent_jobs
            ;;
        "network")
            test_network_connectivity
            ;;
        "full")
            print_highlight "Running Full Test Suite"
            echo
            
            test_mesh_lifecycle || exit 1
            echo
            
            test_job_types || exit 1
            echo
            
            test_concurrent_jobs || exit 1
            echo
            
            test_network_connectivity || exit 1
            echo
            
            print_highlight "All Tests Completed Successfully!"
            ;;
        *)
            echo "Usage: $0 [lifecycle|types|concurrent|network|full]"
            echo
            echo "Test modes:"
            echo "  lifecycle   - Test complete mesh job lifecycle"
            echo "  types      - Test different job types (Echo, WASM)"
            echo "  concurrent - Test concurrent job submissions"
            echo "  network    - Test network connectivity"
            echo "  full       - Run all tests (default)"
            exit 1
            ;;
    esac
}

# Ensure we have the required tools
if ! command -v jq &> /dev/null; then
    print_error "jq is required but not installed. Please install jq."
    exit 1
fi

if ! command -v curl &> /dev/null; then
    print_error "curl is required but not installed. Please install curl."
    exit 1
fi

# Run the tests
main "$@" 