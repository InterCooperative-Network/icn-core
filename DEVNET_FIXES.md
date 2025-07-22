# ICN Devnet Issue Fixes

## 🔍 **Issues Resolved**

### **1. Rust Compiler Crash (SIGSEGV)**
- **Problem**: `cargo run -p icn-cli` triggered SIGSEGV during compilation
- **Root Cause**: LLVM debug info generation overwhelming with complex codebase
- **Symptoms**: `error: rustc interrupted by SIGSEGV, printing backtrace`

### **2. P2P Network Convergence Failure**
- **Problem**: All nodes showed 0 peer connections
- **Root Cause**: mDNS discovery not working in Docker bridge network
- **Symptoms**: `Peer counts: Node-A=0, Node-B=0, Node-C=0`

## ✅ **Fixes Applied**

### **Fix 1: Enhanced Build Process**

#### **Updated `scripts/build-devnet.sh`**
- Added CLI binary compilation alongside node binary
- Uses safe environment variables for compilation
- Copies release binaries to debug location if needed

#### **Updated `icn-devnet/launch_federation.sh`**
- Modified `setup_federation_cli()` to use pre-built CLI binaries
- Falls back to protected compilation if binaries not found
- Added RUST_MIN_STACK and RUSTFLAGS for safe compilation

#### **Updated `icn-devnet/Dockerfile`**
- Builds both `icn-node` and `icn-cli` in Docker image
- Copies both binaries to runtime stage
- Uses consistent environment variables with build script

### **Fix 2: P2P Network Configuration**

#### **Updated `icn-devnet/docker-compose.yml`**
- Added `ICN_BOOTSTRAP_PEERS=/ip4/icn-node-a/tcp/4001` to all worker nodes (B-J)
- Enables explicit peer discovery instead of relying only on mDNS
- Node A acts as bootstrap node for the federation

## 🧪 **Testing the Fixes**

### **Step 1: Clean Environment**
```bash
# Stop any existing containers
cd icn-devnet
docker-compose down --volumes --remove-orphans

# Clean Rust build cache
cd ..
cargo clean
```

### **Step 2: Build with Fixed Scripts**
```bash
# Use the enhanced build script
./scripts/build-devnet.sh

# Verify both binaries were built
ls -la target/release/icn-*
```

### **Step 3: Launch Devnet**
```bash
# Launch the federation
just devnet

# Or manually:
cd icn-devnet
./launch_federation.sh
```

### **Step 4: Verify P2P Connectivity**
```bash
# Check node status (should show peers > 0)
curl -s -H "x-api-key: devnet-a-key" "http://localhost:5001/status" | jq '.peer_count'
curl -s -H "x-api-key: devnet-b-key" "http://localhost:5002/status" | jq '.peer_count'
curl -s -H "x-api-key: devnet-c-key" "http://localhost:5003/status" | jq '.peer_count'
```

### **Step 5: Test Job Submission**
```bash
# Submit a test job
curl -X POST http://localhost:5001/mesh/submit \
  -H "Content-Type: application/json" \
  -H "x-api-key: devnet-a-key" \
  -d '{
    "manifest_cid": "test",
    "spec_json": {
      "kind": {"Echo": {"payload": "Hello ICN!"}},
      "inputs": [],
      "outputs": [],
      "required_resources": {"cpu_cores": 0, "memory_mb": 0}
    },
    "cost_mana": 50
  }'
```

## 🔧 **Environment Variables Set**

The following environment variables are used to work around compilation issues:

```bash
RUST_MIN_STACK=16777216          # 16MB stack (as suggested by error)
RUSTFLAGS="-C debuginfo=1 -C opt-level=1 -C strip=symbols -C codegen-units=4"
CARGO_PROFILE_RELEASE_DEBUG=1    # Limited debug info
```

## 📁 **Files Modified**

1. `scripts/build-devnet.sh` - Enhanced to build CLI and handle compilation safely
2. `icn-devnet/launch_federation.sh` - Updated federation setup to use pre-built CLI
3. `icn-devnet/docker-compose.yml` - Added bootstrap peers for P2P discovery
4. `icn-devnet/Dockerfile` - Enhanced to build and copy both binaries

## ⚠️ **Important Notes**

- The fixes maintain backward compatibility
- If pre-built binaries aren't found, the system falls back to protected compilation
- Bootstrap peer configuration is only for Docker environment
- All environment variables are scoped to avoid affecting other builds

## 🚀 **Expected Results**

After applying these fixes:
- ✅ Compilation should complete without SIGSEGV crashes
- ✅ P2P network should show peer connections > 0
- ✅ Federation setup should complete successfully
- ✅ Basic job submission and tracking should work
- ✅ All 3+ nodes should be healthy and responding

If issues persist, check the Docker logs:
```bash
docker-compose logs icn-node-a
docker-compose logs icn-node-b
docker-compose logs icn-node-c
``` 