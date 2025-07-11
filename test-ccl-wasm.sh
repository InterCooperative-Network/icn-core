#!/bin/bash

# Test ICN CCL WASM Job Auto-execution
# CCL WASM jobs get automatically executed without needing bidding

set -e

NODE_URL="http://localhost:5001"
API_KEY="devnet-a-key"

echo "=== ICN CCL WASM Job Test ==="

# Check if devnet is running
echo "Checking devnet..."
if ! curl -s -f -H "x-api-key: $API_KEY" "$NODE_URL/status" >/dev/null; then
    echo "❌ Devnet not accessible - is it running?"
    exit 1
fi
echo "✅ Devnet is running"

# Check mana balance
echo "Checking mana balance..."
MANA_RESPONSE=$(curl -s -H "x-api-key: $API_KEY" "$NODE_URL/mana")
echo "✅ Mana response: $MANA_RESPONSE"

# Submit CCL WASM job
echo "Submitting CCL WASM job..."
JOB_RESPONSE=$(curl -s -X POST "$NODE_URL/mesh/submit" \
    -H 'Content-Type: application/json' \
    -H "x-api-key: $API_KEY" \
    -d '{
        "manifest_cid": "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e",
        "spec_json": {
            "kind": "CclWasm",
            "inputs": [],
            "outputs": ["result"],
            "required_resources": { "cpu_cores": 1, "memory_mb": 128 }
        },
        "cost_mana": 5
    }')

if echo "$JOB_RESPONSE" | jq -e '.error' >/dev/null; then
    echo "❌ Job submission failed: $(echo "$JOB_RESPONSE" | jq -r '.error')"
    exit 1
fi

JOB_ID=$(echo "$JOB_RESPONSE" | jq -r '.job_id')
echo "✅ CCL WASM job submitted with ID: $JOB_ID"

# Monitor job status
echo "Monitoring job status..."
for i in {1..30}; do
    STATUS_RESPONSE=$(curl -s -H "x-api-key: $API_KEY" "$NODE_URL/mesh/jobs/$JOB_ID")
    STATUS=$(echo "$STATUS_RESPONSE" | jq -r '.status')
    
    echo "[$i] Job status: $STATUS"
    
    if [ "$STATUS" = "completed" ]; then
        echo "✅ Job completed successfully!"
        echo "Final job details:"
        echo "$STATUS_RESPONSE" | jq '.'
        break
    elif [ "$STATUS" = "failed" ]; then
        echo "❌ Job failed!"
        echo "Job details:"
        echo "$STATUS_RESPONSE" | jq '.'
        exit 1
    fi
    
    sleep 2
done

echo "=== CCL WASM test complete ===" 