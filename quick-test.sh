#!/bin/bash

# Quick ICN Mesh Job Test
# Submit a simple echo job and check its status

set -e

NODE_URL="http://localhost:5001"
API_KEY="devnet-a-key"

echo "=== Quick ICN Mesh Job Test ==="

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
MANA_BALANCE=$(echo "$MANA_RESPONSE" | jq -r '.balance // 0')
echo "✅ Mana balance: $MANA_BALANCE"

# Submit job
echo "Submitting job..."
JOB_RESPONSE=$(curl -s -X POST "$NODE_URL/mesh/submit" \
    -H 'Content-Type: application/json' \
    -H "x-api-key: $API_KEY" \
    -d '{
        "manifest_cid": "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e",
        "spec_json": {
            "kind": { "Echo": { "payload": "Quick test!" } },
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
echo "✅ Job submitted with ID: $JOB_ID"

# Check job status
echo "Checking job status..."
STATUS_RESPONSE=$(curl -s -H "x-api-key: $API_KEY" "$NODE_URL/mesh/jobs/$JOB_ID")
STATUS=$(echo "$STATUS_RESPONSE" | jq -r '.status')
echo "✅ Job status: $STATUS"

echo "=== Quick test complete ==="
echo "Job ID: $JOB_ID"
echo "Status: $STATUS" 