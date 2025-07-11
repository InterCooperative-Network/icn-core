#!/bin/bash

# ICN Devnet Job Execution Enabler
# Shows how to get jobs executing instead of staying pending

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Working CID
WORKING_CID="bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e"

print_step() {
    echo -e "\n${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

print_highlight() {
    echo -e "${YELLOW}⭐ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Test if job manager is running
test_job_manager() {
    local node_url=$1
    local api_key=$2
    
    # Submit a test job and see if it gets processed
    local response=$(curl -s -X POST "$node_url/mesh/submit" \
        -H 'Content-Type: application/json' \
        -H "x-api-key: $api_key" \
        -d "{
            \"manifest_cid\": \"$WORKING_CID\",
            \"spec_json\": {
                \"kind\": \"Echo\",
                \"payload\": \"Job Manager Test\"
            },
            \"cost_mana\": 10
        }" 2>/dev/null)
    
    local job_id=$(echo "$response" | jq -r '.job_id // empty')
    
    if [ -n "$job_id" ]; then
        echo "$job_id"
        return 0
    else
        return 1
    fi
}

# Monitor job status for changes
monitor_job_status() {
    local node_url=$1
    local api_key=$2
    local job_id=$3
    local max_attempts=${4:-20}
    
    print_info "Monitoring job $job_id for status changes..."
    
    local attempts=0
    local last_status=""
    
    while [ $attempts -lt $max_attempts ]; do
        local response=$(curl -s -X GET "$node_url/mesh/jobs/$job_id" -H "x-api-key: $api_key" 2>/dev/null)
        local status=$(echo "$response" | jq -r '.status // "unknown"')
        
        if [ "$status" != "$last_status" ]; then
            case "$status" in
                "pending")
                    print_info "Status: ⏳ Pending (waiting for job manager)"
                    ;;
                "assigned")
                    print_info "Status: 📋 Assigned (executor selected)"
                    ;;
                "completed")
                    print_success "Status: ✅ Completed!"
                    return 0
                    ;;
                "failed")
                    echo -e "${RED}❌ Status: Failed${NC}"
                    return 1
                    ;;
                *)
                    print_info "Status: $status"
                    ;;
            esac
            last_status="$status"
        fi
        
        sleep 2
        ((attempts++))
    done
    
    print_info "Monitoring timeout after $max_attempts attempts"
    return 2
}

main() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════════════════════════════╗"
    echo "║                                                                          ║"
    echo "║                🔧 ICN JOB EXECUTION ENABLER 🔧                         ║"
    echo "║                 From Pending to Completed                               ║"
    echo "║                                                                          ║"
    echo "╚══════════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    # Step 1: Explain the problem
    print_step "1. Understanding Why Jobs Stay Pending"
    
    print_info "Jobs stay pending because:"
    echo "  • Job Manager background task is not running"
    echo "  • Nodes are not acting as executors (bidding on jobs)"
    echo "  • Full mesh lifecycle is not active"
    
    print_info "Solutions:"
    echo "  • Solution A: Submit CclWasm jobs (auto-execute)"
    echo "  • Solution B: Enable full mesh lifecycle with executors"
    
    # Step 2: Test current job submission
    print_step "2. Testing Current Job Submission"
    
    local node_url="http://localhost:5001"
    local api_key="devnet-a-key"
    
    print_info "Submitting regular Echo job (will stay pending)..."
    
    local test_job_id=$(test_job_manager "$node_url" "$api_key")
    if [ -n "$test_job_id" ]; then
        print_success "Job submitted: $test_job_id"
        
        # Check initial status
        sleep 1
        local status_response=$(curl -s -X GET "$node_url/mesh/jobs/$test_job_id" -H "x-api-key: $api_key")
        local status=$(echo "$status_response" | jq -r '.status // "unknown"')
        print_info "Initial status: $status (stays pending without job manager)"
    else
        print_error "Failed to submit test job"
        return 1
    fi
    
    # Step 3: Show Solution A - CclWasm Jobs
    print_step "3. Solution A: Auto-Executing CclWasm Jobs"
    
    print_info "CclWasm jobs auto-execute when the job manager processes them"
    print_info "Let's submit a CclWasm job..."
    
    # Submit CclWasm job
    local wasm_response=$(curl -s -X POST "$node_url/mesh/submit" \
        -H 'Content-Type: application/json' \
        -H "x-api-key: $api_key" \
        -d "{
            \"manifest_cid\": \"$WORKING_CID\",
            \"spec_json\": {
                \"kind\": \"CclWasm\",
                \"inputs\": [],
                \"outputs\": [],
                \"required_resources\": {\"cpu_cores\": 1, \"memory_mb\": 128}
            },
            \"cost_mana\": 20
        }" 2>/dev/null)
    
    local wasm_job_id=$(echo "$wasm_response" | jq -r '.job_id // empty')
    
    if [ -n "$wasm_job_id" ]; then
        print_success "CclWasm job submitted: $wasm_job_id"
        print_info "Note: This job will auto-execute when job manager starts"
    else
        print_info "CclWasm job submission failed (this is expected - needs job manager)"
    fi
    
    # Step 4: Show how to enable the job manager
    print_step "4. Enabling Job Execution"
    
    print_highlight "To enable job execution, the ICN node needs to:"
    echo "  1. Start the Job Manager background task"
    echo "  2. Process pending jobs from the queue"
    echo "  3. Auto-execute CclWasm jobs immediately"
    echo "  4. Handle mesh lifecycle for regular jobs"
    
    print_info "The job manager is started by calling:"
    echo "     RuntimeContext::spawn_mesh_job_manager()"
    
    print_info "In production nodes, this should be called during startup"
    
    # Step 5: API calls to start job processing
    print_step "5. Starting Job Processing (Simulated)"
    
    print_info "In a real implementation, you would:"
    echo "  • Call spawn_mesh_job_manager() on node startup"
    echo "  • Configure nodes as executors to bid on jobs"
    echo "  • Enable P2P mesh networking for job announcements"
    
    print_highlight "For testing with current devnet:"
    echo "  • Use CclWasm jobs for auto-execution"
    echo "  • Submit jobs with proper manifest CIDs"
    echo "  • Enable job manager in node configuration"
    
    # Step 6: Summary and next steps
    print_step "6. Summary & Next Steps"
    
    echo -e "${GREEN}"
    echo "╔══════════════════════════════════════════════════════════════════════════╗"
    echo "║                        EXECUTION SUMMARY                                ║"
    echo "╠══════════════════════════════════════════════════════════════════════════╣"
    echo "║                                                                          ║"
    echo "║  Current Status: Jobs submit successfully but stay PENDING              ║"
    echo "║                                                                          ║"
    echo "║  Missing Components:                                                     ║"
    echo "║  • Job Manager background task                                           ║"
    echo "║  • Executor nodes (bidding)                                             ║"
    echo "║  • Full mesh lifecycle                                                   ║"
    echo "║                                                                          ║"
    echo "║  Quick Fixes:                                                            ║"
    echo "║  • Submit CclWasm jobs for auto-execution                               ║"
    echo "║  • Start job manager on node startup                                    ║"
    echo "║  • Configure nodes as executors                                         ║"
    echo "║                                                                          ║"
    echo "╚══════════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    print_highlight "Ready to implement full job execution! 🚀"
}

# Command line handling
case "${1:-demo}" in
    "demo")
        main
        ;;
    "test-job")
        node_url="${2:-http://localhost:5001}"
        api_key="${3:-devnet-a-key}"
        job_id=$(test_job_manager "$node_url" "$api_key")
        if [ -n "$job_id" ]; then
            echo "Job ID: $job_id"
            monitor_job_status "$node_url" "$api_key" "$job_id"
        fi
        ;;
    "monitor")
        if [ -z "$2" ]; then
            echo "Usage: $0 monitor <job_id> [node_url] [api_key]"
            exit 1
        fi
        job_id="$2"
        node_url="${3:-http://localhost:5001}"
        api_key="${4:-devnet-a-key}"
        monitor_job_status "$node_url" "$api_key" "$job_id"
        ;;
    *)
        echo "Usage: $0 [demo|test-job|monitor]"
        echo ""
        echo "Commands:"
        echo "  demo     - Full explanation and demonstration"
        echo "  test-job - Submit and monitor a test job"
        echo "  monitor  - Monitor a specific job ID"
        ;;
esac 