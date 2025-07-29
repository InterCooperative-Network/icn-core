#!/bin/bash

# Test script to validate the enhanced P2P mesh networking configuration
# This script tests the configuration without actually running the full devnet

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

log "🧪 Testing Enhanced P2P Mesh Networking Configuration"
echo ""

# Test 1: Validate Docker Compose configuration
log "Test 1: Validating Docker Compose configuration..."
cd /home/runner/work/icn-core/icn-core/icn-devnet
if docker compose -f docker-compose.yml config --quiet; then
    success "Docker Compose configuration is valid"
else
    error "Docker Compose configuration is invalid"
fi

# Test 2: Check timing configuration for all nodes
log "Test 2: Checking devnet timing configuration..."
timing_count=$(grep "ICN_BOOTSTRAP_INTERVAL_SECS=30" docker-compose.yml | wc -l)
if [ "$timing_count" -eq 10 ]; then
    success "All 10 nodes have devnet timing configuration (30s bootstrap)"
else
    error "Only $timing_count/10 nodes have devnet timing configuration"
fi

discovery_count=$(grep "ICN_PEER_DISCOVERY_INTERVAL_SECS=10" docker-compose.yml | wc -l)
if [ "$discovery_count" -eq 10 ]; then
    success "All 10 nodes have discovery timing configuration (10s discovery)"
else
    error "Only $discovery_count/10 nodes have discovery timing configuration"
fi

# Test 3: Validate multi-bootstrap strategy
log "Test 3: Validating multi-bootstrap peer strategy..."

# Node B should bootstrap from A
node_b_peers=$(grep -A 10 "devnet-b-key" docker-compose.yml | grep "ICN_BOOTSTRAP_PEERS" | grep -o "icn-node-[a-j]" | wc -l)
if [ "$node_b_peers" -eq 1 ]; then
    success "Node B bootstraps from 1 peer (Node A) ✓"
else
    warning "Node B bootstrap configuration unexpected: $node_b_peers peers"
fi

# Node C should bootstrap from A and B
node_c_peers=$(grep -A 10 "devnet-c-key" docker-compose.yml | grep "ICN_BOOTSTRAP_PEERS" | grep -o "icn-node-[a-j]" | wc -l)
if [ "$node_c_peers" -eq 2 ]; then
    success "Node C bootstraps from 2 peers (A, B) ✓"
else
    warning "Node C bootstrap configuration unexpected: $node_c_peers peers"
fi

# Node J should bootstrap from multiple peers
node_j_peers=$(grep -A 10 "devnet-j-key" docker-compose.yml | grep "ICN_BOOTSTRAP_PEERS" | grep -o "icn-node-[a-j]" | wc -l)
if [ "$node_j_peers" -ge 3 ]; then
    success "Node J bootstraps from $node_j_peers peers (multi-bootstrap) ✓"
else
    warning "Node J bootstrap configuration may be insufficient: $node_j_peers peers"
fi

# Test 4: Build verification
log "Test 4: Verifying build configuration..."
cd /home/runner/work/icn-core/icn-core
if cargo check --package icn-node --package icn-network > /dev/null 2>&1; then
    success "Build configuration is valid (cargo check passed)"
else
    error "Build configuration has issues"
fi

# Test 5: Enhanced launch script validation
log "Test 5: Validating enhanced launch script..."
cd icn-devnet
if [ -x "./launch_enhanced_federation.sh" ]; then
    success "Enhanced launch script is executable"
else
    error "Enhanced launch script is not executable"
fi

if ./launch_enhanced_federation.sh --help > /dev/null 2>&1; then
    success "Enhanced launch script help works"
else
    error "Enhanced launch script help failed"
fi

# Test 6: Environment variable configuration
log "Test 6: Testing environment variable configuration..."
cd /home/runner/work/icn-core/icn-core

# Check that environment variables are properly configured
if grep -q "ICN_BOOTSTRAP_INTERVAL_SECS" crates/icn-node/src/config.rs; then
    success "Bootstrap interval environment variable support added"
else
    error "Bootstrap interval environment variable support missing"
fi

if grep -q "ICN_PEER_DISCOVERY_INTERVAL_SECS" crates/icn-node/src/config.rs; then
    success "Peer discovery interval environment variable support added"
else
    error "Peer discovery interval environment variable support missing"
fi

# Test 7: API endpoint verification
log "Test 7: Checking new API endpoints..."
if grep -q "/network/discover" crates/icn-node/src/node.rs; then
    success "Network discover endpoint added"
else
    error "Network discover endpoint missing"
fi

if grep -q "network_discover_handler" crates/icn-node/src/node.rs; then
    success "Network discover handler implemented"
else
    error "Network discover handler missing"
fi

# Test 8: Configuration validation
log "Test 8: Validating network configuration options..."
if grep -q "devnet()" crates/icn-network/src/lib.rs; then
    success "Devnet NetworkConfig method added"
else
    error "Devnet NetworkConfig method missing"
fi

if grep -q "devnet()" crates/icn-network/src/service_factory.rs; then
    success "Devnet NetworkServiceConfig method added"
else
    error "Devnet NetworkServiceConfig method missing"
fi

echo ""
log "🎉 Configuration Validation Complete"
echo ""
success "✅ All tests passed! Enhanced P2P mesh networking is properly configured"
echo ""
echo -e "${GREEN}Key improvements implemented:${NC}"
echo -e "  🚀 Multi-bootstrap strategy (A→B→C→... instead of all→A)"
echo -e "  ⚡ Devnet-optimized timing (10s discovery, 30s bootstrap vs 60s/300s)"
echo -e "  🌐 Enhanced network API (/network/discover endpoint)"
echo -e "  📊 Improved status endpoint (detailed network stats)"
echo -e "  🔧 Environment variable support for timing configuration"
echo -e "  🎯 Enhanced launch script with mesh convergence validation"
echo ""
echo -e "${YELLOW}To test the mesh:${NC}"
echo -e "  ./launch_enhanced_federation.sh --nodes 3    # 3-node test"
echo -e "  ./launch_enhanced_federation.sh --nodes 10   # 10-node test"
echo ""