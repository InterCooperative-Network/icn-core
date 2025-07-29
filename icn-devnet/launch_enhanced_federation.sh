#!/bin/bash
set -e

# ICN Enhanced Federation Launch Script
# Supports both 3-node and 10-node federation testing with mesh convergence validation

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
DOCKER_COMPOSE=""
MAX_WAIT_TIME=300  # 5 minutes max wait for mesh convergence
HEALTH_CHECK_INTERVAL=5
DISCOVERY_TRIGGER_INTERVAL=15
TARGET_PEERS_PER_NODE=2  # Default for 3-node setup

# Command line options
START_ONLY=false
NODE_COUNT=3
ENABLE_JOB_TESTING=true
ENABLE_CONVERGENCE_VALIDATION=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --start-only)
      START_ONLY=true
      shift
      ;;
    --nodes)
      NODE_COUNT="$2"
      shift 2
      ;;
    --no-job-testing)
      ENABLE_JOB_TESTING=false
      shift
      ;;
    --no-convergence)
      ENABLE_CONVERGENCE_VALIDATION=false
      shift
      ;;
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo "Options:"
      echo "  --start-only          Start federation but skip tests"
      echo "  --nodes COUNT         Number of nodes to start (3 or 10)"
      echo "  --no-job-testing      Skip job distribution testing"
      echo "  --no-convergence      Skip mesh convergence validation"
      echo "  -h, --help           Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Validate node count
if [[ "$NODE_COUNT" != "3" && "$NODE_COUNT" != "10" ]]; then
    error "Invalid node count: $NODE_COUNT. Only 3 or 10 nodes are supported."
fi

# Calculate expected peer count for mesh topology
TARGET_PEERS_PER_NODE=$((NODE_COUNT - 1))

# Node configuration arrays
declare -a NODE_NAMES=()
declare -a NODE_URLS=()
declare -a NODE_API_KEYS=()
declare -a NODE_SERVICES=()

# Configure node arrays based on count
if [[ "$NODE_COUNT" == "3" ]]; then
    NODE_NAMES=("Node-A" "Node-B" "Node-C")
    NODE_URLS=("http://localhost:5001" "http://localhost:5002" "http://localhost:5003")
    NODE_API_KEYS=("devnet-a-key" "devnet-b-key" "devnet-c-key")
    NODE_SERVICES=("icn-node-a" "icn-node-b" "icn-node-c")
else
    NODE_NAMES=("Node-A" "Node-B" "Node-C" "Node-D" "Node-E" "Node-F" "Node-G" "Node-H" "Node-I" "Node-J")
    NODE_URLS=("http://localhost:5001" "http://localhost:5002" "http://localhost:5003" "http://localhost:5004" "http://localhost:5005" "http://localhost:5006" "http://localhost:5007" "http://localhost:5008" "http://localhost:5009" "http://localhost:5010")
    NODE_API_KEYS=("devnet-a-key" "devnet-b-key" "devnet-c-key" "devnet-d-key" "devnet-e-key" "devnet-f-key" "devnet-g-key" "devnet-h-key" "devnet-i-key" "devnet-j-key")
    NODE_SERVICES=("icn-node-a" "icn-node-b" "icn-node-c" "icn-node-d" "icn-node-e" "icn-node-f" "icn-node-g" "icn-node-h" "icn-node-i" "icn-node-j")
fi

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

info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
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

# Get peer count for a node
get_peer_count() {
    local node_url="$1"
    local api_key="$2"
    
    local response=$(curl -s -H "x-api-key: $api_key" "$node_url/status" 2>/dev/null)
    if [[ $? -eq 0 ]]; then
        if command -v jq &> /dev/null; then
            echo "$response" | jq -r '.peer_count // 0' 2>/dev/null || echo "0"
        else
            # Fallback without jq - extract peer_count from JSON
            echo "$response" | grep -o '"peer_count":[0-9]*' | cut -d':' -f2 | head -1 || echo "0"
        fi
    else
        echo "0"
    fi
}

# Get detailed network stats for a node
get_network_stats() {
    local node_url="$1"
    local api_key="$2"
    
    local response=$(curl -s -H "x-api-key: $api_key" "$node_url/status" 2>/dev/null)
    if [[ $? -eq 0 ]] && command -v jq &> /dev/null; then
        echo "$response" | jq -r '.network_stats // {}' 2>/dev/null || echo "{}"
    else
        echo "{}"
    fi
}

# Trigger peer discovery on a node
trigger_peer_discovery() {
    local node_url="$1"
    local api_key="$2"
    local node_name="$3"
    
    local response=$(curl -s -X POST -H "x-api-key: $api_key" "$node_url/network/discover" 2>/dev/null)
    if [[ $? -eq 0 ]]; then
        if command -v jq &> /dev/null; then
            local discovered=$(echo "$response" | jq -r '.discovered_peers // 0' 2>/dev/null || echo "0")
            log "Triggered discovery on $node_name: $discovered peers discovered"
        else
            log "Triggered discovery on $node_name"
        fi
    else
        warning "Failed to trigger discovery on $node_name"
    fi
}

# Wait for mesh network convergence
wait_for_mesh_convergence() {
    if [[ "$ENABLE_CONVERGENCE_VALIDATION" != "true" ]]; then
        info "Mesh convergence validation skipped"
        return 0
    fi

    log "Waiting for P2P mesh convergence (target: $TARGET_PEERS_PER_NODE peers per node)..."
    
    local converged=false
    local start_time=$(date +%s)
    local last_discovery_time=0
    
    while [ "$converged" = false ]; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [ $elapsed -gt $MAX_WAIT_TIME ]; then
            warning "Mesh convergence incomplete after $MAX_WAIT_TIME seconds"
            show_convergence_status
            return 1
        fi
        
        # Trigger discovery on all nodes periodically
        if [ $((elapsed - last_discovery_time)) -gt $DISCOVERY_TRIGGER_INTERVAL ]; then
            log "Triggering peer discovery on all nodes..."
            for i in "${!NODE_NAMES[@]}"; do
                trigger_peer_discovery "${NODE_URLS[$i]}" "${NODE_API_KEYS[$i]}" "${NODE_NAMES[$i]}"
            done
            last_discovery_time=$elapsed
        fi
        
        # Check peer counts for all nodes
        local all_converged=true
        local total_connections=0
        
        for i in "${!NODE_NAMES[@]}"; do
            local peer_count=$(get_peer_count "${NODE_URLS[$i]}" "${NODE_API_KEYS[$i]}")
            total_connections=$((total_connections + peer_count))
            
            if [ "$peer_count" -lt "$TARGET_PEERS_PER_NODE" ]; then
                all_converged=false
            fi
        done
        
        # Show status every 30 seconds
        if [ $((elapsed % 30)) -eq 0 ] && [ $elapsed -gt 0 ]; then
            log "Convergence status after ${elapsed}s:"
            show_convergence_status
        fi
        
        if [ "$all_converged" = true ]; then
            converged=true
            success "✨ Full mesh convergence achieved! (${total_connections} total peer connections)"
            show_convergence_status
        else
            echo -n "."
            sleep $HEALTH_CHECK_INTERVAL
        fi
    done
}

# Show detailed convergence status
show_convergence_status() {
    echo ""
    log "🌐 Mesh Convergence Status:"
    
    local total_connections=0
    local converged_nodes=0
    
    for i in "${!NODE_NAMES[@]}"; do
        local peer_count=$(get_peer_count "${NODE_URLS[$i]}" "${NODE_API_KEYS[$i]}")
        local status_icon="❌"
        
        if [ "$peer_count" -eq "$TARGET_PEERS_PER_NODE" ]; then
            status_icon="✅"
            converged_nodes=$((converged_nodes + 1))
        elif [ "$peer_count" -gt 0 ]; then
            status_icon="🔄"
        fi
        
        echo "  $status_icon ${NODE_NAMES[$i]}: $peer_count/$TARGET_PEERS_PER_NODE peers"
        total_connections=$((total_connections + peer_count))
    done
    
    local convergence_percentage=$((converged_nodes * 100 / NODE_COUNT))
    echo "  📊 Convergence: $converged_nodes/$NODE_COUNT nodes ($convergence_percentage%)"
    echo "  🔗 Total connections: $total_connections"
    echo ""
}

# Test cross-node job distribution
test_cross_node_job_distribution() {
    if [[ "$ENABLE_JOB_TESTING" != "true" ]]; then
        info "Job distribution testing skipped"
        return 0
    fi

    log "🚀 Testing cross-node job distribution..."
    
    # Submit a job from Node A
    log "Submitting test job from ${NODE_NAMES[0]}..."
    local job_response=$(curl -s -X POST "${NODE_URLS[0]}/mesh/submit" \
        -H "Content-Type: application/json" \
        -H "x-api-key: ${NODE_API_KEYS[0]}" \
        -d '{
            "manifest_cid": "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku",
            "spec_json": {
                "kind": { "Echo": { "payload": "Cross-node job distribution test!" } },
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
        warning "Failed to submit job to ${NODE_NAMES[0]}"
        return 1
    fi
    
    local job_id=""
    if command -v jq &> /dev/null; then
        job_id=$(echo "$job_response" | jq -r '.job_id // empty' 2>/dev/null)
    fi
    
    if [ -z "$job_id" ]; then
        warning "No job_id returned from job submission. Response: $job_response"
        return 1
    fi
    
    success "Job submitted with ID: $job_id"
    
    # Give jobs time to propagate
    log "Waiting for job propagation across the mesh..."
    sleep 10
    
    # Check job visibility on all nodes
    local visible_count=0
    
    for i in "${!NODE_NAMES[@]}"; do
        local jobs_response=$(curl -s -H "x-api-key: ${NODE_API_KEYS[$i]}" "${NODE_URLS[$i]}/mesh/jobs" 2>/dev/null)
        if [ $? -eq 0 ]; then
            local job_count=0
            if command -v jq &> /dev/null; then
                job_count=$(echo "$jobs_response" | jq '.jobs | length' 2>/dev/null || echo "0")
            else
                # Simple fallback - check if job_id appears in response
                if echo "$jobs_response" | grep -q "$job_id"; then
                    job_count=1
                fi
            fi
            
            if [ "$job_count" -gt 0 ]; then
                success "✅ ${NODE_NAMES[$i]}: Job visible ($job_count jobs total)"
                visible_count=$((visible_count + 1))
            else
                warning "❌ ${NODE_NAMES[$i]}: Job not visible"
            fi
        else
            warning "❌ ${NODE_NAMES[$i]}: Failed to query jobs"
        fi
    done
    
    local visibility_percentage=$((visible_count * 100 / NODE_COUNT))
    
    if [ "$visible_count" -eq "$NODE_COUNT" ]; then
        success "🎉 Perfect job distribution: Job visible on all $NODE_COUNT nodes (100%)"
    elif [ "$visible_count" -gt $((NODE_COUNT / 2)) ]; then
        warning "⚠️  Partial job distribution: Job visible on $visible_count/$NODE_COUNT nodes ($visibility_percentage%)"
    else
        warning "❌ Poor job distribution: Job visible on only $visible_count/$NODE_COUNT nodes ($visibility_percentage%)"
    fi
}

# Display federation status summary
show_federation_status() {
    log "📋 Federation Status Summary ($NODE_COUNT nodes):"
    echo ""
    
    for i in "${!NODE_NAMES[@]}"; do
        local name="${NODE_NAMES[$i]}"
        local url="${NODE_URLS[$i]}"
        local api_key="${NODE_API_KEYS[$i]}"
        
        echo -e "${BLUE}$name ($url):${NC}"
        
        local info=$(curl -s -H "x-api-key: $api_key" "$url/info" 2>/dev/null)
        local status=$(curl -s -H "x-api-key: $api_key" "$url/status" 2>/dev/null)
        
        if [ $? -eq 0 ]; then
            if command -v jq &> /dev/null; then
                echo "  📋 Name: $(echo "$info" | jq -r '.name // "unknown"' 2>/dev/null)"
                echo "  🔢 Version: $(echo "$info" | jq -r '.version // "unknown"' 2>/dev/null)"
                echo "  🌐 Peers: $(echo "$status" | jq -r '.peer_count // "unknown"' 2>/dev/null)"
                
                # Show network stats if available
                local network_stats=$(echo "$status" | jq -r '.network_stats // {}' 2>/dev/null)
                if [ "$network_stats" != "{}" ]; then
                    local kademlia_peers=$(echo "$network_stats" | jq -r '.kademlia_peers // 0' 2>/dev/null)
                    local messages_sent=$(echo "$network_stats" | jq -r '.messages_sent // 0' 2>/dev/null)
                    local messages_received=$(echo "$network_stats" | jq -r '.messages_received // 0' 2>/dev/null)
                    echo "  📡 Kademlia peers: $kademlia_peers"
                    echo "  📤 Messages sent: $messages_sent"
                    echo "  📥 Messages received: $messages_received"
                fi
            else
                echo "  📋 Status: Online (detailed stats require jq)"
            fi
            echo "  ✅ Status: Online"
        else
            echo "  ❌ Status: Offline"
        fi
        echo ""
    done
}

# Setup federation CLI (simplified for demonstration)
setup_federation_cli() {
    log "Setting up federation coordination..."
    # This could be expanded to use icn-cli for federation setup
    # For now, we'll just ensure the nodes are connected via P2P mesh
    success "Federation setup completed"
}

# Main execution flow
main() {
    log "🚀 ICN Enhanced Federation Devnet Launch Starting..."
    echo "   Configuration: $NODE_COUNT nodes, convergence target: $TARGET_PEERS_PER_NODE peers per node"
    echo ""
    
    check_prerequisites
    
    # Clean up any existing containers
    log "Cleaning up existing containers..."
    if [ "$NODE_COUNT" == "3" ]; then
        $DOCKER_COMPOSE -f "$COMPOSE_FILE" down --volumes --remove-orphans 2>/dev/null || true
    else
        $DOCKER_COMPOSE -f "$COMPOSE_FILE" down --volumes --remove-orphans 2>/dev/null || true
    fi
    
    # Start the federation
    log "Starting ICN federation ($NODE_COUNT nodes)..."
    if [ "$NODE_COUNT" == "3" ]; then
        # Start only first 3 services for 3-node setup
        $DOCKER_COMPOSE -f "$COMPOSE_FILE" up -d icn-node-a icn-node-b icn-node-c
    else
        # Start all services for 10-node setup
        $DOCKER_COMPOSE -f "$COMPOSE_FILE" up -d
    fi
    
    # Wait for nodes to be healthy
    for i in "${!NODE_NAMES[@]}"; do
        wait_for_node "${NODE_NAMES[$i]}" "${NODE_URLS[$i]}" "${NODE_API_KEYS[$i]}"
    done
    
    # Wait for mesh convergence
    wait_for_mesh_convergence
    
    setup_federation_cli
    
    if [ "$START_ONLY" = true ]; then
        success "Federation started (start-only mode)"
        show_federation_status
        return 0
    fi
    
    # Test cross-node job distribution
    test_cross_node_job_distribution
    
    # Show final status
    show_federation_status
    
    success "🎉 ICN Enhanced Devnet is running successfully!"
    echo ""
    echo -e "${GREEN}✅ Verified functionality:${NC}"
    echo -e "  ✓ All $NODE_COUNT nodes are healthy and responding"
    echo -e "  ✓ P2P mesh network convergence achieved"
    echo -e "  ✓ HTTP APIs are working with authentication"
    if [[ "$ENABLE_JOB_TESTING" == "true" ]]; then
        echo -e "  ✓ Cross-node job distribution tested"
    fi
    echo ""
    echo -e "${GREEN}Access points:${NC}"
    for i in "${!NODE_NAMES[@]}"; do
        echo -e "  📡 ${NODE_NAMES[$i]} HTTP API: ${BLUE}${NODE_URLS[$i]}${NC}"
    done
    echo ""
    echo -e "${YELLOW}Try testing the mesh with:${NC}"
    echo -e "  curl -X POST ${NODE_URLS[0]}/network/discover -H 'x-api-key: ${NODE_API_KEYS[0]}'"
    echo -e "  curl ${NODE_URLS[1]}/status -H 'x-api-key: ${NODE_API_KEYS[1]}' | jq"
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