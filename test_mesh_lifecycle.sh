#!/bin/bash

# Test script to demonstrate complete mesh job lifecycle with production services

echo "🚀 Testing Complete Mesh Job Lifecycle with Production Services"
echo "============================================================="

# Build with production features
echo "🔧 Building with production features..."
cargo build --release --bin icn-node --features "with-libp2p"

# Create node configurations
echo "📝 Creating node configurations..."
mkdir -p ./icn_data/{node_a,node_b}
echo '{"balances": {}}' > ./icn_data/node_a/mana_ledger.json
echo '{"balances": {}}' > ./icn_data/node_b/mana_ledger.json

# Start Node A (submitter) in background
echo "🚀 Starting Node A (submitter)..."
./target/release/icn-node \
    --mana-ledger-path ./icn_data/node_a/mana_ledger.json \
    --http-listen-addr 0.0.0.0:5001 \
    --listen-address "/ip4/0.0.0.0/tcp/4001" \
    --enable-p2p \
    --node-name "ICN Node A (Submitter)" \
    --storage-backend memory \
    --api-key "node-a-key" &
NODE_A_PID=$!

# Start Node B (executor) in background
echo "🚀 Starting Node B (executor)..."
./target/release/icn-node \
    --mana-ledger-path ./icn_data/node_b/mana_ledger.json \
    --http-listen-addr 0.0.0.0:5002 \
    --listen-address "/ip4/0.0.0.0/tcp/4002" \
    --enable-p2p \
    --node-name "ICN Node B (Executor)" \
    --storage-backend memory \
    --api-key "node-b-key" &
NODE_B_PID=$!

# Wait for nodes to start
echo "⏳ Waiting for nodes to start up..."
sleep 10

# Test function to wait for node readiness
wait_for_node() {
    local port=$1
    local key=$2
    local name=$3
    
    echo "🔍 Checking $name..."
    for i in {1..30}; do
        if curl -s -H "x-api-key: $key" "http://localhost:$port/status" > /dev/null 2>&1; then
            echo "✅ $name is ready"
            return 0
        fi
        sleep 1
    done
    echo "❌ $name failed to start"
    return 1
}

# Check if both nodes are ready
wait_for_node 5001 "node-a-key" "Node A"
wait_for_node 5002 "node-b-key" "Node B"

echo ""
echo "🧪 Testing Mesh Job Lifecycle..."
echo "================================"

# Submit a job to Node A
echo "📤 Submitting job to Node A..."
JOB_RESPONSE=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -H "x-api-key: node-a-key" \
    -d '{
        "manifest_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        "spec_json": {
            "kind": "Echo",
            "inputs": ["hello"],
            "outputs": ["result"],
            "required_resources": {
                "cpu_cores": 1,
                "memory_mb": 256
            }
        },
        "cost_mana": 50
    }' \
    "http://localhost:5001/mesh/submit")

echo "📋 Job submission response: $JOB_RESPONSE"

# Extract job ID
JOB_ID=$(echo "$JOB_RESPONSE" | jq -r '.job_id // empty')

if [ -z "$JOB_ID" ]; then
    echo "❌ Failed to submit job"
    kill $NODE_A_PID $NODE_B_PID 2>/dev/null
    exit 1
fi

echo "✅ Job submitted with ID: $JOB_ID"

# Monitor job status
echo "🔍 Monitoring job status..."
for i in {1..60}; do
    JOB_STATUS=$(curl -s -H "x-api-key: node-a-key" "http://localhost:5001/mesh/jobs/$JOB_ID")
    STATUS=$(echo "$JOB_STATUS" | jq -r '.status // "unknown"')
    
    echo "📊 Job status: $STATUS"
    
    if [ "$STATUS" = "completed" ]; then
        echo "🎉 Job completed successfully!"
        echo "📋 Final job details:"
        echo "$JOB_STATUS" | jq '.'
        break
    elif [ "$STATUS" = "failed" ]; then
        echo "❌ Job failed"
        echo "📋 Job details:"
        echo "$JOB_STATUS" | jq '.'
        break
    fi
    
    sleep 2
done

echo ""
echo "🏁 Test completed!"
echo "=================="

# Clean up
echo "🧹 Cleaning up..."
kill $NODE_A_PID $NODE_B_PID 2>/dev/null
wait

echo "✅ Mesh job lifecycle test completed" 