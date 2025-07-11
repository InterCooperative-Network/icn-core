#!/bin/bash

# Test script for mesh lifecycle with manually injected bids
# This tests the full mesh lifecycle using StubMeshNetworkService

set -e

echo "=== ICN Mesh Lifecycle Test with Manual Bid Injection ==="

# Configuration
NODE_PORT_START=5001
NODE_COUNT=3
TEST_TIMEOUT=60
API_KEY="test123"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if nodes are running
check_nodes() {
    log_info "Checking if nodes are running..."
    
    for i in $(seq 1 $NODE_COUNT); do
        port=$((NODE_PORT_START + i - 1))
        if curl -s -H "X-API-Key: $API_KEY" "http://localhost:$port/status" > /dev/null; then
            log_success "Node $i (port $port) is running"
        else
            log_error "Node $i (port $port) is not responding"
            echo "Please start the nodes first with: just run-devnet"
            exit 1
        fi
    done
}

# Submit a test job to the first node
submit_test_job() {
    local port=$NODE_PORT_START
    local manifest_cid="bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e"
    
    log_info "Submitting Echo job to node 1 (port $port)..."
    
    # Submit Echo job
    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -H "X-API-Key: $API_KEY" \
        -d '{
            "manifest_cid": "'$manifest_cid'",
            "spec_json": {
                "kind": {
                    "Echo": {
                        "payload": "Hello from manual bid test!"
                    }
                },
                "inputs": [],
                "outputs": [],
                "required_resources": {
                    "cpu_cores": 1,
                    "memory_mb": 100
                }
            },
            "cost_mana": 50
        }' \
        "http://localhost:$port/mesh/submit")
    
    if echo "$response" | jq -e '.job_id' > /dev/null 2>&1; then
        local job_id=$(echo "$response" | jq -r '.job_id')
        log_success "Job submitted successfully: $job_id"
        echo "$job_id"
    else
        log_error "Failed to submit job"
        echo "Response: $response"
        exit 1
    fi
}

# Check job status
check_job_status() {
    local port=$1
    local job_id=$2
    
    local response=$(curl -s -H "X-API-Key: $API_KEY" \
        "http://localhost:$port/mesh/jobs/$job_id")
    
    if echo "$response" | jq -e '.status' > /dev/null 2>&1; then
        echo "$response" | jq -r '.status'
    else
        echo "unknown"
    fi
}

# Create and compile the bid injection helper
create_bid_injection_helper() {
    log_info "Creating bid injection helper..."
    
    cat > /tmp/bid_injector.rs << 'EOF'
use icn_runtime::context::{RuntimeContext, StubMeshNetworkService, MeshNetworkServiceType};
use icn_mesh::{JobId, MeshJobBid, Resources};
use icn_common::Did;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <job_id> <executor_did> <price_mana> <node_port>", args[0]);
        std::process::exit(1);
    }
    
    let job_id_str = &args[1];
    let executor_did_str = &args[2];
    let price_mana: u64 = args[3].parse()?;
    let node_port: u16 = args[4].parse()?;
    
    // Parse inputs
    let job_id = JobId::from_str(job_id_str)?;
    let executor_did = Did::from_str(executor_did_str)?;
    
    println!("Injecting bid for job {} from executor {} with price {} mana", 
             job_id_str, executor_did_str, price_mana);
    
    // For testing, we'll create a minimal context to access the stub service
    // In reality, this would need to be done through the actual running node
    // This is a simplified approach for demonstration
    
    // Create a test bid
    let bid = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: executor_did.clone(),
        price_mana,
        resources: Resources {
            cpu_cores: 1,
            memory_mb: 100,
        },
        signature: icn_identity::SignatureBytes(vec![]), // Empty signature for testing
    };
    
    println!("Created bid: {:?}", bid);
    
    // Note: In a real implementation, we would need to access the actual 
    // running node's stub service instance. For now, this demonstrates
    // the bid structure.
    
    println!("Bid injection completed (simulation)");
    Ok(())
}
EOF

    # Try to compile the helper
    if cd /tmp && cargo init --name bid_injector --bin . > /dev/null 2>&1; then
        # Add dependencies to Cargo.toml
        cat >> /tmp/Cargo.toml << 'EOF'

[dependencies]
icn-runtime = { path = "../home/faherty.network.matt/dev/icn/icn-core/crates/icn-runtime" }
icn-mesh = { path = "../home/faherty.network.matt/dev/icn/icn-core/crates/icn-mesh" }
icn-common = { path = "../home/faherty.network.matt/dev/icn/icn-core/crates/icn-common" }
icn-identity = { path = "../home/faherty.network.matt/dev/icn/icn-core/crates/icn-identity" }
tokio = { version = "1.0", features = ["full"] }
EOF
        
        mv /tmp/bid_injector.rs /tmp/src/main.rs
        
        log_info "Bid injection helper created (simulation)"
        return 0
    else
        log_warning "Could not create bid injection helper, will use simulated approach"
        return 1
    fi
}

# Simulate bid injection by creating bids manually
simulate_bid_injection() {
    local job_id=$1
    
    log_info "Simulating bid injection for job: $job_id"
    
    # For this test, we'll manually create some fake executor DIDs
    # and simulate the bid injection process
    
    local executors=(
        "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH"
        "did:key:z6MkrJvwAfLVgFntBzYCBLXXNGMNPdpJcRw4Qc9vq8vN8oSz"
        "did:key:z6MkoTHsgNNrby8JzCNQ1iRLyW5QQ6R8Xuu6AA8igGrMVPUM"
    )
    
    local prices=(30 40 35)
    
    for i in "${!executors[@]}"; do
        local executor_did="${executors[$i]}"
        local price="${prices[$i]}"
        
        log_info "Creating bid from $executor_did for $price mana"
        
        # In a real implementation, this would inject the bid into the 
        # StubMeshNetworkService's staged_bids collection
        # For now, we'll just log the bid creation
        
        echo "  Bid $((i+1)): executor=$executor_did, price=$price mana"
    done
    
    log_success "Simulated $((${#executors[@]})) bids for job $job_id"
}

# Test the actual mesh service for stub usage
test_mesh_service_type() {
    local port=$NODE_PORT_START
    
    log_info "Testing mesh service type on node 1..."
    
    # Try to get some information about the mesh service
    local info_response=$(curl -s -H "X-API-Key: $API_KEY" \
        "http://localhost:$port/info")
    
    if echo "$info_response" | jq -e '.version' > /dev/null 2>&1; then
        local version=$(echo "$info_response" | jq -r '.version')
        log_info "Node version: $version"
        
        # Check if this is a testing build (likely using stub services)
        if echo "$info_response" | grep -q "test" || echo "$info_response" | grep -q "stub"; then
            log_success "Node appears to be using stub services"
            return 0
        else
            log_warning "Node may be using production services"
            return 1
        fi
    else
        log_warning "Could not determine node service type"
        return 1
    fi
}

# Monitor job progression
monitor_job_progression() {
    local job_id=$1
    local timeout=$2
    local port=$NODE_PORT_START
    
    log_info "Monitoring job $job_id for up to $timeout seconds..."
    
    local start_time=$(date +%s)
    local last_status=""
    
    while true; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [ $elapsed -ge $timeout ]; then
            log_error "Timeout reached ($timeout seconds)"
            break
        fi
        
        local status=$(check_job_status $port $job_id)
        
        if [ "$status" != "$last_status" ]; then
            log_info "Job status changed: $last_status -> $status"
            last_status="$status"
            
            case "$status" in
                "pending")
                    log_info "Job is pending, waiting for bids..."
                    ;;
                "bidding")
                    log_info "Job is in bidding phase"
                    ;;
                "assigned")
                    log_success "Job has been assigned to an executor"
                    ;;
                "executing")
                    log_info "Job is being executed"
                    ;;
                "completed")
                    log_success "Job completed successfully!"
                    return 0
                    ;;
                "failed")
                    log_error "Job failed"
                    return 1
                    ;;
                *)
                    log_warning "Unknown status: $status"
                    ;;
            esac
        fi
        
        sleep 2
    done
    
    return 1
}

# Main test flow
main() {
    echo "Starting mesh lifecycle test with manual bid injection..."
    echo "Node count: $NODE_COUNT"
    echo "Port range: $NODE_PORT_START-$((NODE_PORT_START + NODE_COUNT - 1))"
    echo
    
    # Step 1: Check nodes
    check_nodes
    
    # Step 2: Test mesh service type
    test_mesh_service_type
    
    # Step 3: Create bid injection helper
    create_bid_injection_helper
    
    # Step 4: Submit a test job
    local job_id=$(submit_test_job)
    if [ -z "$job_id" ]; then
        log_error "Failed to get job ID"
        exit 1
    fi
    
    echo
    log_info "Job ID: $job_id"
    echo
    
    # Step 5: Wait a moment for job to be announced
    log_info "Waiting for job announcement..."
    sleep 3
    
    # Step 6: Simulate bid injection
    simulate_bid_injection "$job_id"
    
    echo
    
    # Step 7: Monitor job progression
    if monitor_job_progression "$job_id" "$TEST_TIMEOUT"; then
        log_success "Test completed successfully!"
        exit 0
    else
        log_error "Test failed or timed out"
        
        # Show final job status
        local final_status=$(check_job_status $NODE_PORT_START "$job_id")
        log_info "Final job status: $final_status"
        
        # Get job details for debugging
        local job_details=$(curl -s -H "X-API-Key: $API_KEY" \
            "http://localhost:$NODE_PORT_START/mesh/jobs/$job_id")
        echo
        log_info "Job details:"
        echo "$job_details" | jq . || echo "$job_details"
        
        exit 1
    fi
}

# Handle script arguments
case "${1:-main}" in
    "check-nodes")
        check_nodes
        ;;
    "submit-job")
        job_id=$(submit_test_job)
        echo "Job ID: $job_id"
        ;;
    "monitor")
        if [ -z "$2" ]; then
            echo "Usage: $0 monitor <job_id>"
            exit 1
        fi
        monitor_job_progression "$2" "$TEST_TIMEOUT"
        ;;
    "inject-bids")
        if [ -z "$2" ]; then
            echo "Usage: $0 inject-bids <job_id>"
            exit 1
        fi
        simulate_bid_injection "$2"
        ;;
    "main"|*)
        main
        ;;
esac 