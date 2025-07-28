#!/bin/bash
set -e

# ICN Devnet Connectivity Test Script
# Tests P2P networking between nodes in the devnet

echo "🧪 ICN Devnet Connectivity Test"
echo "================================"

# Configuration
NODE_A_URL="http://localhost:5001"
NODE_B_URL="http://localhost:5002"
NODE_C_URL="http://localhost:5003"

NODE_A_API_KEY="devnet-a-key"
NODE_B_API_KEY="devnet-b-key"
NODE_C_API_KEY="devnet-c-key"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Test function to check if a node is healthy
test_node_health() {
    local node_name="$1"
    local node_url="$2"
    local api_key="$3"
    
    info "Testing $node_name health..."
    
    local response=$(curl -s -H "x-api-key: $api_key" "$node_url/info" 2>/dev/null || echo "")
    if [ -n "$response" ]; then
        local name=$(echo "$response" | jq -r '.name // "unknown"' 2>/dev/null || echo "unknown")
        local version=$(echo "$response" | jq -r '.version // "unknown"' 2>/dev/null || echo "unknown")
        success "$node_name is healthy - Name: $name, Version: $version"
        return 0
    else
        error "$node_name is not responding"
        return 1
    fi
}

# Test function to check P2P peer count
test_peer_connectivity() {
    local node_name="$1"
    local node_url="$2"
    local api_key="$3"
    
    info "Testing $node_name P2P connectivity..."
    
    local response=$(curl -s -H "x-api-key: $api_key" "$node_url/status" 2>/dev/null || echo "")
    if [ -n "$response" ]; then
        local peer_count=$(echo "$response" | jq -r '.peer_count // 0' 2>/dev/null || echo "0")
        local kademlia_peers=$(echo "$response" | jq -r '.kademlia_peers // 0' 2>/dev/null || echo "0")
        if [ "$peer_count" -gt 0 ]; then
            success "$node_name has $peer_count peer connection(s), $kademlia_peers Kademlia peers"
            return 0
        else
            warning "$node_name has 0 peer connections, $kademlia_peers Kademlia peers"
            # Also check network stats for more details
            local stats_response=$(curl -s -H "x-api-key: $api_key" "$node_url/network/stats" 2>/dev/null || echo "")
            if [ -n "$stats_response" ]; then
                info "Network stats for $node_name: $stats_response"
            fi
            return 1
        fi
    else
        error "$node_name status endpoint not responding"
        return 1
    fi
}

# Test function to submit a job and verify it works
test_job_submission() {
    local node_name="$1"
    local node_url="$2"
    local api_key="$3"
    
    info "Testing job submission on $node_name..."
    
    local job_response=$(curl -s -X POST "$node_url/mesh/submit" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $api_key" \
        -d '{
            "manifest_cid": "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku",
            "spec_json": {
                "kind": { "Echo": { "payload": "Test job from '"$node_name"'" } },
                "inputs": [],
                "outputs": [],
                "required_resources": {
                    "cpu_cores": 0,
                    "memory_mb": 0
                }
            },
            "cost_mana": 50
        }' 2>/dev/null || echo "")
    
    if [ -n "$job_response" ]; then
        local job_id=$(echo "$job_response" | jq -r '.job_id // empty' 2>/dev/null || echo "")
        if [ -n "$job_id" ]; then
            success "$node_name job submitted successfully - Job ID: $job_id"
            return 0
        else
            warning "$node_name job submission returned unexpected response: $job_response"
            return 1
        fi
    else
        error "$node_name job submission failed"
        return 1
    fi
}

# Main test execution
main() {
    echo ""
    info "Starting devnet connectivity tests..."
    echo ""
    
    # Test basic health
    echo "🏥 Testing Node Health"
    echo "----------------------"
    health_a=0
    health_b=0
    health_c=0
    
    test_node_health "Node A" "$NODE_A_URL" "$NODE_A_API_KEY" && health_a=1
    test_node_health "Node B" "$NODE_B_URL" "$NODE_B_API_KEY" && health_b=1
    test_node_health "Node C" "$NODE_C_URL" "$NODE_C_API_KEY" && health_c=1
    
    echo ""
    
    # Test P2P connectivity
    echo "🌐 Testing P2P Connectivity"
    echo "----------------------------"
    p2p_a=0
    p2p_b=0
    p2p_c=0
    
    test_peer_connectivity "Node A" "$NODE_A_URL" "$NODE_A_API_KEY" && p2p_a=1
    test_peer_connectivity "Node B" "$NODE_B_URL" "$NODE_B_API_KEY" && p2p_b=1
    test_peer_connectivity "Node C" "$NODE_C_URL" "$NODE_C_API_KEY" && p2p_c=1
    
    echo ""
    
    # Test job submission (only if nodes are healthy)
    echo "🚀 Testing Job Submission"
    echo "--------------------------"
    jobs_a=0
    jobs_b=0
    jobs_c=0
    
    if [ $health_a -eq 1 ]; then
        test_job_submission "Node A" "$NODE_A_URL" "$NODE_A_API_KEY" && jobs_a=1
    fi
    if [ $health_b -eq 1 ]; then
        test_job_submission "Node B" "$NODE_B_URL" "$NODE_B_API_KEY" && jobs_b=1
    fi
    if [ $health_c -eq 1 ]; then
        test_job_submission "Node C" "$NODE_C_URL" "$NODE_C_API_KEY" && jobs_c=1
    fi
    
    echo ""
    
    # Summary
    echo "📊 Test Summary"
    echo "==============="
    
    local healthy_nodes=$((health_a + health_b + health_c))
    local connected_nodes=$((p2p_a + p2p_b + p2p_c))
    local working_job_nodes=$((jobs_a + jobs_b + jobs_c))
    
    echo "Healthy nodes: $healthy_nodes/3"
    echo "Nodes with P2P connections: $connected_nodes/3"
    echo "Nodes accepting jobs: $working_job_nodes/3"
    echo ""
    
    if [ $healthy_nodes -eq 3 ] && [ $connected_nodes -ge 2 ]; then
        success "✨ Devnet is functioning well!"
        
        if [ $connected_nodes -eq 3 ]; then
            success "🌟 Perfect P2P connectivity - all nodes connected!"
        else
            warning "P2P could be improved - not all nodes have connections"
        fi
        
        return 0
    elif [ $healthy_nodes -ge 2 ]; then
        warning "Devnet is partially functional but has issues"
        return 1
    else
        error "Devnet has significant problems"
        return 2
    fi
}

# Run tests
main "$@"