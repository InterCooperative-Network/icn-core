#!/bin/bash

# Test ICN Mesh Job Lifecycle via Stub API
# This script tests the full mesh job workflow using the running devnet

set -e  # Exit on any error

# Configuration
NODE_URL="http://localhost:5001"
API_KEY="devnet-a-key"
TIMEOUT=30

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')] $1${NC}"
}

success() {
    echo -e "${GREEN}[$(date '+%H:%M:%S')] ✓ $1${NC}"
}

error() {
    echo -e "${RED}[$(date '+%H:%M:%S')] ✗ $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date '+%H:%M:%S')] ⚠ $1${NC}"
}

# Check if devnet is running
check_devnet() {
    log "Checking if devnet is accessible..."
    if ! curl -s -f -H "x-api-key: $API_KEY" "$NODE_URL/status" >/dev/null; then
        error "Devnet not accessible at $NODE_URL"
        error "Please ensure devnet is running with: just devnet-up"
        exit 1
    fi
    success "Devnet is accessible"
}

# Check initial mana balance
check_mana() {
    log "Checking initial mana balance..."
    local mana_response=$(curl -s -H "x-api-key: $API_KEY" "$NODE_URL/mana")
    local mana_balance=$(echo "$mana_response" | jq -r '.balance // 0')
    
    if [ "$mana_balance" -lt 20 ]; then
        error "Insufficient mana balance: $mana_balance (need at least 20)"
        exit 1
    fi
    success "Mana balance: $mana_balance"
}

# Submit a test job
submit_job() {
    log "Submitting mesh job..."
    
    local job_payload='{
        "manifest_cid": "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e",
        "spec_json": {
            "kind": { "Echo": { "payload": "Hello ICN Mesh!" } },
            "inputs": [],
            "outputs": ["result"],
            "required_resources": { "cpu_cores": 1, "memory_mb": 128 }
        },
        "cost_mana": 10
    }'
    
    local response=$(curl -s -X POST "$NODE_URL/mesh/submit" \
        -H 'Content-Type: application/json' \
        -H "x-api-key: $API_KEY" \
        -d "$job_payload")
    
    if echo "$response" | jq -e '.error' >/dev/null; then
        error "Job submission failed: $(echo "$response" | jq -r '.error')"
        exit 1
    fi
    
    JOB_ID=$(echo "$response" | jq -r '.job_id')
    if [ "$JOB_ID" = "null" ] || [ -z "$JOB_ID" ]; then
        error "No job ID returned from submission"
        error "Response: $response"
        exit 1
    fi
    
    success "Job submitted with ID: $JOB_ID"
}

# Check job status
check_job_status() {
    local expected_status="$1"
    local description="$2"
    
    log "Checking job status (expecting: $expected_status)..."
    
    local response=$(curl -s -H "x-api-key: $API_KEY" "$NODE_URL/mesh/jobs/$JOB_ID")
    
    if echo "$response" | jq -e '.error' >/dev/null; then
        error "Failed to get job status: $(echo "$response" | jq -r '.error')"
        return 1
    fi
    
    local status=$(echo "$response" | jq -r '.status')
    log "Current job status: $status"
    
    if [ "$status" = "$expected_status" ]; then
        success "$description"
        return 0
    else
        warn "Job status is '$status', expected '$expected_status'"
        return 1
    fi
}

# Wait for job status with timeout
wait_for_status() {
    local expected_status="$1"
    local description="$2"
    local timeout_seconds="${3:-$TIMEOUT}"
    
    log "Waiting for job status '$expected_status' (timeout: ${timeout_seconds}s)..."
    
    local start_time=$(date +%s)
    while true; do
        if check_job_status "$expected_status" "$description"; then
            return 0
        fi
        
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [ $elapsed -ge $timeout_seconds ]; then
            error "Timeout waiting for status '$expected_status' after ${timeout_seconds}s"
            return 1
        fi
        
        sleep 2
    done
}

# Inject a manual bid
inject_bid() {
    log "Injecting manual bid..."
    
    local bid_payload="{
        \"job_id\": \"$JOB_ID\",
        \"executor_id\": \"node-a\",
        \"estimated_cost\": 8
    }"
    
    local response=$(curl -s -X POST "$NODE_URL/mesh/stub/bid" \
        -H 'Content-Type: application/json' \
        -H "x-api-key: $API_KEY" \
        -d "$bid_payload")
    
    if echo "$response" | jq -e '.error' >/dev/null; then
        error "Bid injection failed: $(echo "$response" | jq -r '.error')"
        exit 1
    fi
    
    success "Bid injected for job $JOB_ID"
}

# Inject execution receipt
inject_receipt() {
    log "Injecting execution receipt..."
    
    local receipt_payload="{
        \"job_id\": \"$JOB_ID\",
        \"executor_id\": \"node-a\",
        \"result\": {
            \"status\": \"Success\",
            \"outputs\": {
                \"result\": \"Echo complete: Hello ICN Mesh!\"
            }
        }
    }"
    
    local response=$(curl -s -X POST "$NODE_URL/mesh/stub/receipt" \
        -H 'Content-Type: application/json' \
        -H "x-api-key: $API_KEY" \
        -d "$receipt_payload")
    
    if echo "$response" | jq -e '.error' >/dev/null; then
        error "Receipt injection failed: $(echo "$response" | jq -r '.error')"
        exit 1
    fi
    
    success "Receipt injected for job $JOB_ID"
}

# Show final job details
show_job_details() {
    log "Fetching final job details..."
    
    local response=$(curl -s -H "x-api-key: $API_KEY" "$NODE_URL/mesh/jobs/$JOB_ID")
    
    if echo "$response" | jq -e '.error' >/dev/null; then
        error "Failed to get job details: $(echo "$response" | jq -r '.error')"
        return 1
    fi
    
    success "Final job details:"
    echo "$response" | jq '.'
}

# Check final mana balance
check_final_mana() {
    log "Checking final mana balance..."
    local mana_response=$(curl -s -H "x-api-key: $API_KEY" "$NODE_URL/mana")
    local final_balance=$(echo "$mana_response" | jq -r '.balance // 0')
    
    success "Final mana balance: $final_balance"
}

# Main test flow
main() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  ICN Mesh Job Lifecycle Test${NC}"
    echo -e "${BLUE}========================================${NC}"
    
    # Prerequisites
    check_devnet
    check_mana
    
    # Step 1: Submit job
    echo -e "\n${YELLOW}Step 1: Job Submission${NC}"
    submit_job
    wait_for_status "pending" "Job is pending"
    
    # Step 2: Inject bid
    echo -e "\n${YELLOW}Step 2: Bid Injection${NC}"
    inject_bid
    sleep 2  # Give time for bid processing
    
    # Step 3: Check if job was assigned
    echo -e "\n${YELLOW}Step 3: Job Assignment${NC}"
    if ! wait_for_status "assigned" "Job was assigned to executor" 10; then
        warn "Job not assigned automatically, checking current status..."
        check_job_status "" "Current status check"
    fi
    
    # Step 4: Inject receipt
    echo -e "\n${YELLOW}Step 4: Receipt Injection${NC}"
    inject_receipt
    
    # Step 5: Wait for completion
    echo -e "\n${YELLOW}Step 5: Job Completion${NC}"
    wait_for_status "completed" "Job completed successfully"
    
    # Step 6: Show final results
    echo -e "\n${YELLOW}Step 6: Final Results${NC}"
    show_job_details
    check_final_mana
    
    echo -e "\n${GREEN}========================================${NC}"
    echo -e "${GREEN}  ✓ Mesh Lifecycle Test Completed!${NC}"
    echo -e "${GREEN}========================================${NC}"
}

# Run the test
main "$@" 