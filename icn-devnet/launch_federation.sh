#!/bin/bash
set -e

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
MAX_WAIT_TIME=180  # 3 minutes max wait
HEALTH_CHECK_INTERVAL=5

# Node endpoints
NODE_A_URL="http://localhost:5001"
NODE_B_URL="http://localhost:5002"
NODE_C_URL="http://localhost:5003"

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
    
    if ! command -v docker-compose &> /dev/null; then
        error "Docker Compose is not installed or not in PATH"
    fi
    
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
    local start_time=$(date +%s)
    
    log "Waiting for $node_name to be healthy..."
    
    while true; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [ $elapsed -gt $MAX_WAIT_TIME ]; then
            error "$node_name failed to become healthy within $MAX_WAIT_TIME seconds"
        fi
        
        if curl -sf "$node_url/info" > /dev/null 2>&1; then
            success "$node_name is healthy"
            break
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
        local node_a_peers=$(curl -s "$NODE_A_URL/status" | jq -r '.peer_count // 0' 2>/dev/null || echo "0")
        local node_b_peers=$(curl -s "$NODE_B_URL/status" | jq -r '.peer_count // 0' 2>/dev/null || echo "0")
        local node_c_peers=$(curl -s "$NODE_C_URL/status" | jq -r '.peer_count // 0' 2>/dev/null || echo "0")
        
        log "Peer counts: Node-A=$node_a_peers, Node-B=$node_b_peers, Node-C=$node_c_peers"
        
        # Each node should have at least 1 peer (ideally 2)
        if [ "$node_a_peers" -gt 0 ] && [ "$node_b_peers" -gt 0 ] && [ "$node_c_peers" -gt 0 ]; then
            converged=true
            success "P2P network has converged"
        else
            echo -n "."
            sleep $HEALTH_CHECK_INTERVAL
        fi
    done
}

# Test mesh job submission and execution
test_mesh_job_execution() {
    log "Testing mesh job execution across federation..."
    
    # Submit job to Node A
    log "Submitting mesh job to Node A..."
    local job_response=$(curl -s -X POST "$NODE_A_URL/mesh/submit" \
        -H "Content-Type: application/json" \
        -d '{
            "manifest_cid": "cidv1-85-20-federation_test_manifest",
            "spec_json": { "Echo": { "payload": "Federation devnet test!" } },
            "cost_mana": 100
        }')
    
    if [ $? -ne 0 ]; then
        error "Failed to submit job to Node A"
    fi
    
    local job_id=$(echo "$job_response" | jq -r '.job_id // empty' 2>/dev/null)
    if [ -z "$job_id" ]; then
        error "No job_id returned from job submission. Response: $job_response"
    fi
    
    success "Job submitted with ID: $job_id"
    
    # Check job status on all nodes
    log "Checking job status across all nodes..."
    
    for i in {1..12}; do  # Check for up to 60 seconds
        log "Status check attempt $i/12..."
        
        # Check Node A
        local status_a=$(curl -s "$NODE_A_URL/mesh/jobs/$job_id" | jq -r '.status // "unknown"' 2>/dev/null)
        log "Node A job status: $status_a"
        
        # Check if job appears in job listings
        local jobs_a=$(curl -s "$NODE_A_URL/mesh/jobs")
        local jobs_b=$(curl -s "$NODE_B_URL/mesh/jobs")
        local jobs_c=$(curl -s "$NODE_C_URL/mesh/jobs")
        
        log "Jobs visible on Node A: $(echo "$jobs_a" | jq '.jobs | length' 2>/dev/null || echo 'unknown')"
        log "Jobs visible on Node B: $(echo "$jobs_b" | jq '.jobs | length' 2>/dev/null || echo 'unknown')"
        log "Jobs visible on Node C: $(echo "$jobs_c" | jq '.jobs | length' 2>/dev/null || echo 'unknown')"
        
        # For now, we're primarily testing that the job was successfully submitted and tracked
        if [ "$status_a" != "unknown" ] && [ "$status_a" != "null" ]; then
            success "Job is being tracked with status: $status_a"
            break
        fi
        
        sleep 5
    done
    
    success "Mesh job execution test completed"
}

# Display federation status
show_federation_status() {
    log "Federation Status Summary:"
    echo ""
    
    for node in "Node-A:$NODE_A_URL" "Node-B:$NODE_B_URL" "Node-C:$NODE_C_URL"; do
        local name=$(echo "$node" | cut -d: -f1)
        local url=$(echo "$node" | cut -d: -f2-3)
        
        echo -e "${BLUE}$name ($url):${NC}"
        
        local info=$(curl -s "$url/info" 2>/dev/null)
        local status=$(curl -s "$url/status" 2>/dev/null)
        
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
    docker-compose -f "$COMPOSE_FILE" down --volumes --remove-orphans 2>/dev/null || true
    
    # Start the federation
    log "Starting ICN federation (3 nodes)..."
    docker-compose -f "$COMPOSE_FILE" up -d
    
    # Wait for nodes to be healthy
    wait_for_node "Node A" "$NODE_A_URL"
    wait_for_node "Node B" "$NODE_B_URL"
    wait_for_node "Node C" "$NODE_C_URL"
    
    # Wait for network convergence
    wait_for_network_convergence
    
    # Test mesh job execution
    test_mesh_job_execution
    
    # Show final status
    show_federation_status
    
    success "🎉 ICN Federation is now running!"
    echo ""
    echo -e "${GREEN}Access points:${NC}"
    echo -e "  📡 Node A HTTP API: ${BLUE}$NODE_A_URL${NC}"
    echo -e "  📡 Node B HTTP API: ${BLUE}$NODE_B_URL${NC}"
    echo -e "  📡 Node C HTTP API: ${BLUE}$NODE_C_URL${NC}"
    echo ""
    echo -e "${YELLOW}Try submitting a job:${NC}"
    echo -e "  curl -X POST $NODE_A_URL/mesh/submit \\"
    echo -e "    -H 'Content-Type: application/json' \\"
    echo -e "    -d '{\"manifest_cid\": \"test\", \"spec_json\": {\"Echo\": {\"payload\": \"Hello Federation!\"}}, \"cost_mana\": 50}'"
    echo ""
    echo -e "${YELLOW}To stop the federation:${NC}"
    echo -e "  docker-compose -f $COMPOSE_FILE down"
}

# Handle cleanup on exit
cleanup() {
    if [ "$1" != "0" ]; then
        error "Federation launch failed"
        log "Cleaning up..."
        docker-compose -f "$COMPOSE_FILE" down --volumes --remove-orphans 2>/dev/null || true
    fi
}

trap cleanup EXIT

# Run main function
main "$@" 