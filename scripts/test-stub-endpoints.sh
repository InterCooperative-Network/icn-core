#!/bin/bash

# Test ICN Mesh Job Lifecycle via Direct API (no auth)
# This script tests the stub endpoints with a locally created server

set -e

NODE_URL="http://localhost:5004"

echo "=== Testing Stub API Endpoints ==="

# Start with a simple job submission test
echo "1. Testing job submission..."
JOB_RESPONSE=$(curl -s -X POST "$NODE_URL/mesh/submit" \
    -H 'Content-Type: application/json' \
    -d '{
        "manifest_cid": "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e",
        "spec_json": {
            "kind": { "Echo": { "payload": "testing" } },
            "inputs": [],
            "outputs": ["result"],
            "required_resources": { "cpu_cores": 1, "memory_mb": 128 }
        },
        "cost_mana": 10
    }')

echo "Job submission response: $JOB_RESPONSE"

# Check if we got a job ID
if echo "$JOB_RESPONSE" | jq -e '.job_id' >/dev/null; then
    JOB_ID=$(echo "$JOB_RESPONSE" | jq -r '.job_id')
    echo "✅ Job submitted successfully: $JOB_ID"
    
    # Test stub bid injection
    echo "2. Testing stub bid injection..."
    BID_RESPONSE=$(curl -s -X POST "$NODE_URL/mesh/stub/bid" \
        -H 'Content-Type: application/json' \
        -d "{
            \"job_id\": \"$JOB_ID\",
            \"executor_id\": \"test-executor\",
            \"estimated_cost\": 5
        }")
    
    echo "Bid injection response: $BID_RESPONSE"
    
    # Test stub receipt injection
    echo "3. Testing stub receipt injection..."
    RECEIPT_RESPONSE=$(curl -s -X POST "$NODE_URL/mesh/stub/receipt" \
        -H 'Content-Type: application/json' \
        -d "{
            \"job_id\": \"$JOB_ID\",
            \"executor_id\": \"test-executor\",
            \"result\": {
                \"status\": \"Success\",
                \"outputs\": {
                    \"result\": \"Echo complete: testing\"
                }
            }
        }")
    
    echo "Receipt injection response: $RECEIPT_RESPONSE"
    
    # Check final job status
    echo "4. Checking final job status..."
    sleep 2
    FINAL_STATUS=$(curl -s "$NODE_URL/mesh/jobs/$JOB_ID")
    echo "Final job status: $FINAL_STATUS"
    
else
    echo "❌ Job submission failed: $JOB_RESPONSE"
    exit 1
fi

echo "=== Test Complete ===" 