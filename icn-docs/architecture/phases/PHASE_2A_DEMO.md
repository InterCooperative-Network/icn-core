# Phase 2A Demo: Multi-Node ICN Network

## 🎯 Objective
Demonstrate that two ICN nodes can discover each other via bootstrap peers and establish a P2P connection.

## 🚀 Demo Steps

### Step 1: Start Bootstrap Node
```bash
# Terminal 1: Start the bootstrap node
cargo run -p icn-node --features with-libp2p -- \
  --enable-p2p \
  --node-name "Bootstrap Node" \
  --http-listen-addr "127.0.0.1:7845" \
  --p2p-listen-addr "/ip4/127.0.0.1/tcp/12345"
```

**Expected Output:**
- Node starts with libp2p networking enabled
- Displays local Peer ID (e.g., `12D3KooWABC123...`)
- HTTP server listening on `127.0.0.1:7845`
- P2P networking listening on `127.0.0.1:12345`

### Step 2: Get Bootstrap Node Peer ID
```bash
# Terminal 2: Check bootstrap node info
curl -s http://127.0.0.1:7845/info | jq .

# Look in Terminal 1 logs for the Peer ID line:
# "📟 Local Peer ID: 12D3KooWABC123..."
```

### Step 3: Start Second Node with Bootstrap Connection
```bash
# Terminal 2: Start second node connecting to bootstrap
cargo run -p icn-node --features with-libp2p -- \
  --enable-p2p \
  --node-name "Worker Node" \
  --http-listen-addr "127.0.0.1:7846" \
  --p2p-listen-addr "/ip4/127.0.0.1/tcp/12346" \
  --bootstrap-peers "/ip4/127.0.0.1/tcp/12345/p2p/12D3KooWABC123..."
```

**Replace `12D3KooWABC123...` with the actual Peer ID from Step 2**

### Step 4: Verify Connection
```bash
# Terminal 3: Check both nodes are operational
curl -s http://127.0.0.1:7845/info | jq .  # Bootstrap node
curl -s http://127.0.0.1:7846/info | jq .  # Worker node

# Check status endpoints
curl -s http://127.0.0.1:7845/status | jq .
curl -s http://127.0.0.1:7846/status | jq .
```

**Expected Results:**
- Both nodes respond to HTTP requests
- Logs show successful P2P connection establishment
- Nodes discover each other via Kademlia DHT

## 🎉 Success Criteria

✅ **Bootstrap Node Starts**: First node starts with libp2p networking  
✅ **Worker Node Connects**: Second node successfully connects to bootstrap peer  
✅ **HTTP APIs Work**: Both nodes respond to HTTP requests  
✅ **P2P Discovery**: Nodes discover each other via libp2p networking  

## 🔧 Next Steps (Phase 2B)

Once this demo works, we'll proceed to:
1. **Cross-Node Job Submission**: Submit jobs on one node, execute on another
2. **Network Message Routing**: Test mesh job announcements across nodes
3. **Multi-Node Integration Tests**: Automated tests for cross-node functionality

## 📝 Notes

- Each node generates a fresh DID identity on startup
- P2P networking uses libp2p with Kademlia DHT for peer discovery
- Bootstrap peers enable initial network entry point
- HTTP APIs provide external interface for job submission and status 