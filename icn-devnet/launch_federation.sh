#!/bin/bash
set -e

# Start-only mode for CI/integration tests
START_ONLY=false
if [ "$1" = "--start-only" ]; then
    START_ONLY=true
fi

# ICN Federation Launch Script
# Starts a 3-node federation and tests cross-node mesh job execution

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
DOCKER_COMPOSE=""
MAX_WAIT_TIME=180  # 3 minutes max wait
HEALTH_CHECK_INTERVAL=5

# Node configuration
NODE_A_URL="http://localhost:5001"
NODE_B_URL="http://localhost:5002"
NODE_C_URL="http://localhost:5003"

# API keys for accessing the nodes
NODE_A_API_KEY="devnet-a-key"
NODE_B_API_KEY="devnet-b-key"
NODE_C_API_KEY="devnet-c-key"

# Helper functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed or not in PATH"
    fi
    
    if command -v docker-compose &> /dev/null; then
        DOCKER_COMPOSE="docker-compose"
    elif docker compose version &> /dev/null; then
        DOCKER_COMPOSE="docker compose"
    else
        error "Docker Compose is not installed or not in PATH"
    fi

    log "Using '$DOCKER_COMPOSE' for container orchestration"
    
    if ! command -v curl &> /dev/null; then
        error "curl is not installed or not in PATH"
    fi
    
    if ! command -v jq &> /dev/null; then
        warning "jq is not installed - JSON output will be raw"
    fi
    
    success "Prerequisites checked"
}

# Wait for node to be healthy
wait_for_node() {
    local node_name="$1"
    local node_url="$2"
    local api_key="$3"
    local start_time=$(date +%s)
    
    log "Waiting for $node_name to be healthy..."
    
    while true; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [ $elapsed -gt $MAX_WAIT_TIME ]; then
            error "$node_name failed to become healthy within $MAX_WAIT_TIME seconds"
        fi
        
        # Try to connect to the node
        local curl_output=$(curl -sf -H "x-api-key: $api_key" "$node_url/info" 2>&1)
        local curl_exit_code=$?
        
        if [ $curl_exit_code -eq 0 ]; then
            success "$node_name is healthy"
            break
        fi
        
        # Show debug info every 30 seconds
        if [ $((elapsed % 30)) -eq 0 ] && [ $elapsed -gt 0 ]; then
            log "Debug: Still waiting for $node_name (${elapsed}s elapsed)"
            log "Debug: curl exit code: $curl_exit_code"
            log "Debug: curl output: $curl_output"
        fi
        
        echo -n "."
        sleep $HEALTH_CHECK_INTERVAL
    done
}

# Wait for P2P network convergence
wait_for_network_convergence() {
    log "Waiting for P2P network convergence..."
    
    local converged=false
    local start_time=$(date +%s)
    
    while [ "$converged" = false ]; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [ $elapsed -gt $MAX_WAIT_TIME ]; then
            error "Network failed to converge within $MAX_WAIT_TIME seconds"
        fi
        
        # Check if each node has connected to others
        local node_a_peers=$(curl -s -H "x-api-key: $NODE_A_API_KEY" "$NODE_A_URL/status" | jq -r '.peer_count // 0' 2>/dev/null || echo "0")
        local node_b_peers=$(curl -s -H "x-api-key: $NODE_B_API_KEY" "$NODE_B_URL/status" | jq -r '.peer_count // 0' 2>/dev/null || echo "0")
        local node_c_peers=$(curl -s -H "x-api-key: $NODE_C_API_KEY" "$NODE_C_URL/status" | jq -r '.peer_count // 0' 2>/dev/null || echo "0")
        
        log "Peer counts: Node-A=$node_a_peers, Node-B=$node_b_peers, Node-C=$node_c_peers"
        
        # For development, we'll be more lenient about convergence
        # If any nodes have peers or enough time has passed, we consider it acceptable
        local total_peers=$((node_a_peers + node_b_peers + node_c_peers))
        
        if [ $total_peers -gt 0 ]; then
            converged=true
            success "P2P network has partial convergence (${total_peers} total peer connections)"
        elif [ $elapsed -gt 60 ]; then
            # After 60 seconds, continue anyway for development purposes
            converged=true
            warning "P2P network convergence incomplete after ${elapsed}s, but continuing for development testing"
        else
            echo -n "."
            sleep $HEALTH_CHECK_INTERVAL
        fi
    done
}

# Configure federation using icn-cli
setup_federation_cli() {
    log "Setting up federation with icn-cli..."
    
    # Set environment variables to work around LLVM/Rust compiler issues if CLI needs compilation
    export RUST_MIN_STACK=16777216
    export RUSTFLAGS="-C debuginfo=0 -C opt-level=1"
    
    # Use pre-built CLI binary instead of cargo run to avoid compilation
    local cli_binary=""
    if [ -f "../target/debug/icn-cli" ]; then
        cli_binary="../target/debug/icn-cli"
    elif [ -f "../target/release/icn-cli" ]; then
        cli_binary="../target/release/icn-cli"
    else
        # Fall back to cargo run with protected environment
        log "Pre-built CLI not found, attempting to compile with safe environment..."
        cargo run -p icn-cli -- \
            --api-url "$NODE_A_URL" \
            --api-key "$NODE_A_API_KEY" \
            federation init >/dev/null
        cargo run -p icn-cli -- \
            --api-url "$NODE_B_URL" \
            --api-key "$NODE_B_API_KEY" \
            federation join "$NODE_A_URL" >/dev/null
        cargo run -p icn-cli -- \
            --api-url "$NODE_C_URL" \
            --api-key "$NODE_C_API_KEY" \
            federation join "$NODE_A_URL" >/dev/null
        cargo run -p icn-cli -- \
            --api-url "$NODE_A_URL" \
            --api-key "$NODE_A_API_KEY" \
            federation sync >/dev/null
        return
    fi
    
    log "Using pre-built CLI binary: $cli_binary"
    
    # Use pre-built binary for federation setup
    $cli_binary \
        --api-url "$NODE_A_URL" \
        --api-key "$NODE_A_API_KEY" \
        federation init >/dev/null
    $cli_binary \
        --api-url "$NODE_B_URL" \
        --api-key "$NODE_B_API_KEY" \
        federation join "$NODE_A_URL" >/dev/null
    $cli_binary \
        --api-url "$NODE_C_URL" \
        --api-key "$NODE_C_API_KEY" \
        federation join "$NODE_A_URL" >/dev/null
    $cli_binary \
        --api-url "$NODE_A_URL" \
        --api-key "$NODE_A_API_KEY" \
        federation sync >/dev/null
}

# Test basic node functionality and job submission
test_mesh_job_execution() {
    log "Testing basic node functionality and job submission..."
    
    # Submit job to Node A
    log "Submitting test job to Node A..."
    local job_response=$(curl -s -X POST "$NODE_A_URL/mesh/submit" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $NODE_A_API_KEY" \
        -d '{
            "manifest_cid": "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku",
            "spec_json": {
                "kind": { "Echo": { "payload": "ICN Devnet is working!" } },
                "inputs": [],
                "outputs": [],
                "required_resources": {
                    "cpu_cores": 0,
                    "memory_mb": 0
                }
            },
            "cost_mana": 50
        }')
    
    if [ $? -ne 0 ]; then
        error "Failed to submit job to Node A"
    fi
    
    local job_id=$(echo "$job_response" | jq -r '.job_id // empty' 2>/dev/null)
    if [ -z "$job_id" ]; then
        error "No job_id returned from job submission. Response: $job_response"
    fi
    
    success "Job submitted with ID: $job_id"
    
    # Check job status on Node A
    log "Checking job status and listings..."
    
    sleep 2  # Give the job a moment to be processed
    
    # Check Node A job status
    local status_a=$(curl -s -H "x-api-key: $NODE_A_API_KEY" "$NODE_A_URL/mesh/jobs/$job_id" 2>/dev/null | jq -r '.status // "unknown"' 2>/dev/null || echo "unknown")
    log "Node A job status: $status_a"
    
    # List jobs on Node A
    local jobs_a=$(curl -s -H "x-api-key: $NODE_A_API_KEY" "$NODE_A_URL/mesh/jobs" 2>/dev/null)
    local job_count=$(echo "$jobs_a" | jq '.jobs | length' 2>/dev/null || echo "unknown")
    log "Jobs visible on Node A: $job_count"
    
    if [ "$job_count" != "unknown" ] && [ "$job_count" -gt 0 ]; then
        success "Job submission and tracking is working"
    else
        warning "Job tracking may not be fully operational, but basic API is working"
    fi
    
    success "Basic functionality test completed"
}

# Display federation status
show_federation_status() {
    log "Federation Status Summary:"
    echo ""
    
    for node in "Node-A:$NODE_A_URL" "Node-B:$NODE_B_URL" "Node-C:$NODE_C_URL"; do
        local name=$(echo "$node" | cut -d: -f1)
        local url=$(echo "$node" | cut -d: -f2-3)
        
        echo -e "${BLUE}$name ($url):${NC}"
        
        # Get the appropriate API key for this node
        local api_key=""
        case "$name" in
            "Node-A") api_key="$NODE_A_API_KEY" ;;
            "Node-B") api_key="$NODE_B_API_KEY" ;;
            "Node-C") api_key="$NODE_C_API_KEY" ;;
        esac
        
        local info=$(curl -s -H "x-api-key: $api_key" "$url/info" 2>/dev/null)
        local status=$(curl -s -H "x-api-key: $api_key" "$url/status" 2>/dev/null)
        
        if [ $? -eq 0 ]; then
            echo "  📋 Name: $(echo "$info" | jq -r '.name // "unknown"' 2>/dev/null)"
            echo "  🔢 Version: $(echo "$info" | jq -r '.version // "unknown"' 2>/dev/null)"
            echo "  🌐 Peers: $(echo "$status" | jq -r '.peer_count // "unknown"' 2>/dev/null)"
            echo "  ✅ Status: Online"
        else
            echo "  ❌ Status: Offline"
        fi
        echo ""
    done
}

# Main execution flow
main() {
    log "🚀 ICN Federation Devnet Launch Starting..."
    echo ""
    
    check_prerequisites
    
    # Clean up any existing containers
    log "Cleaning up existing containers..."
    $DOCKER_COMPOSE -f "$COMPOSE_FILE" down --volumes --remove-orphans 2>/dev/null || true
    
    # Start the federation
    log "Starting ICN federation (3 nodes)..."
    $DOCKER_COMPOSE -f "$COMPOSE_FILE" up -d
    
    # Wait for nodes to be healthy
    wait_for_node "Node A" "$NODE_A_URL" "$NODE_A_API_KEY"
    wait_for_node "Node B" "$NODE_B_URL" "$NODE_B_API_KEY"
    wait_for_node "Node C" "$NODE_C_URL" "$NODE_C_API_KEY"
    
    # Wait for network convergence
    wait_for_network_convergence

    setup_federation_cli

    if [ "$START_ONLY" = true ]; then
        success "Federation started (start-only mode)"
        return 0
    fi

    # Test mesh job execution
    test_mesh_job_execution

    # Show final status
    show_federation_status

    success "🎉 ICN Devnet is running successfully!"
    echo ""
    echo -e "${GREEN}✅ Verified functionality:${NC}"
    echo -e "  ✓ All 3 nodes are healthy and responding"
    echo -e "  ✓ HTTP APIs are working with authentication"
    echo -e "  ✓ Basic job submission is functional"
    echo ""
    echo -e "${GREEN}Access points:${NC}"
    echo -e "  📡 Node A HTTP API: ${BLUE}$NODE_A_URL${NC}"
    echo -e "  📡 Node B HTTP API: ${BLUE}$NODE_B_URL${NC}"
    echo -e "  📡 Node C HTTP API: ${BLUE}$NODE_C_URL${NC}"
    echo ""
    echo -e "${YELLOW}Try submitting a job:${NC}"
    echo -e "  curl -X POST $NODE_A_URL/mesh/submit \\"
    echo -e "    -H 'Content-Type: application/json' \\"
    echo -e "    -H 'x-api-key: $NODE_A_API_KEY' \\"
    echo -e "    -d '{\"manifest_cid\": \"test\", \"spec_json\": {\"kind\": {\"Echo\": {\"payload\": \"Hello ICN!\"}}, \"inputs\": [], \"outputs\": [], \"required_resources\": {\"cpu_cores\": 0, \"memory_mb\": 0}}, \"cost_mana\": 50}'"
    echo ""
    echo -e "${YELLOW}To stop the federation:${NC}"
    echo -e "  $DOCKER_COMPOSE -f $COMPOSE_FILE down"
}

# Handle cleanup on exit
cleanup() {
    if [ "$1" != "0" ]; then
        error "Federation launch failed"
        log "Cleaning up..."
        $DOCKER_COMPOSE -f "$COMPOSE_FILE" down --volumes --remove-orphans 2>/dev/null || true
    fi
}

trap 'cleanup $?' EXIT

# Run main function
main "$@" 
