# ICN Core Troubleshooting Guide

This document provides solutions to common issues encountered when developing, testing, and deploying ICN nodes.

## 🚨 Critical Issues (Recently Fixed)

### Mana Initialization Failure - "Account not found" (FIXED ✅)

**Symptoms:**
- Job submission fails with error: `"Mesh job submission failed: Internal runtime error: Error spending mana: Common(DatabaseError(\"Account not found\"))"`
- Nodes start successfully but can't process jobs
- No clear error messages during startup

**Root Cause:**
Silent panic in `crates/icn-node/src/node.rs` when mana initialization failed due to improper error handling with `.expect()` call.

**Solution (Implemented):**
Replaced `.expect()` with proper error handling that provides detailed error information:

```rust
// BEFORE (bad - silent panic):
rt_ctx.credit_mana(&node_did, 1000).await.expect("Failed to initialize node with mana");

// AFTER (good - proper error handling):
match rt_ctx.credit_mana(&node_did, 1000).await {
    Ok(()) => {
        info!("✅ Node initialized with 1000 mana");
    }
    Err(e) => {
        error!("❌ Failed to initialize node with mana: {:?}", e);
        error!("Node DID: {:?}", node_did);
        error!("Mana ledger type: {:?}", rt_ctx.mana_ledger);
        return Err(Box::new(e));
    }
}
```

**Verification:**
Look for this line in node startup logs:
```
✅ Node initialized with 1000 mana
```

**Prevention:**
- Never use `.expect()` or `.unwrap()` in startup code
- Always handle initialization errors gracefully with detailed logging
- Test initialization failure scenarios

## 🔧 Development Issues

### Wrong API Authentication Header

**Symptoms:**
- API calls return: `{"error":"missing or invalid api key"}`
- Using `Authorization: Bearer` header

**Solution:**
Use `X-API-Key` header instead:

```bash
# WRONG:
curl -H "Authorization: Bearer devnet-a-key" http://localhost:5001/info

# CORRECT:
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/info
```

### Invalid Job Submission JSON Structure

**Symptoms:**
- Job submission fails with: `Failed to deserialize the JSON body into the target type: missing field 'manifest_cid'`

**Solution:**
Use the correct JSON structure with all required fields:

```json
{
  "manifest_cid": "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354",
  "spec_json": {
    "kind": {
      "Echo": {
        "payload": "your message"
      }
    }
  },
  "cost_mana": 50
}
```

**Common Mistakes:**
- Missing `kind` wrapper in `spec_json`
- Using `command` instead of structured `spec_json`
- Missing `manifest_cid` field

### Build Memory Issues

**Symptoms:**
- Rust compiler crashes with `SIGKILL` during build
- Docker build fails with memory-related errors
- `cargo build` hangs or terminates unexpectedly

**Solutions:**

1. **Increase Rust stack size:**
```bash
export RUST_MIN_STACK=67108864  # 64 MiB
cargo build --release
```

2. **Reduce build parallelism:**
```bash
cargo build --release -j 1  # Single-threaded build
```

3. **Use more memory for Docker:**
```bash
# In Docker settings, increase memory limit to 8GB+ if possible
```

4. **Clear build cache:**
```bash
cargo clean
docker system prune -a
```

## 🌐 Network and Connectivity Issues

### Peers Not Connecting

**Symptoms:**
- Nodes start but don't discover each other
- Empty peer lists: `curl http://localhost:5001/network/peers`
- libp2p logs show `No known peers`

**Solutions:**

1. **Check mDNS is enabled:**
```bash
# In docker-compose.yml or node startup:
ICN_ENABLE_MDNS=true
```

2. **Verify network configuration:**
```bash
# Check if nodes are on the same Docker network
docker network ls
docker network inspect icn-devnet_icn-federation
```

3. **Check firewall/ports:**
```bash
# Ensure P2P ports are accessible
netstat -an | grep 4001
netstat -an | grep 4002
netstat -an | grep 4003
```

4. **Manual peer connection:**
```bash
# Get peer ID from logs and connect manually
curl -X POST http://localhost:5001/network/connect \
  -H "Content-Type: application/json" \
  -H "X-API-Key: devnet-a-key" \
  -d '{"peer": "/ip4/172.20.0.3/tcp/4001/p2p/PEER_ID"}'
```

### Job Stuck in "Pending" or "Failed - No Bids"

**Symptoms:**
- Jobs submit successfully but never execute
- Job status shows "pending" indefinitely or "failed - no bids"

**Explanation:**
This is **expected behavior** in single-node testing. The node can't bid on its own jobs.

**Solutions:**

1. **For testing purposes:** This is normal - mana is automatically refunded
2. **For real execution:** Start multiple nodes that can act as executors
3. **Check node capabilities:** Ensure nodes have appropriate executor capabilities

### Port Conflicts

**Symptoms:**
- Docker containers fail to start
- Error: "Port already in use"

**Solutions:**

1. **Check for existing processes:**
```bash
lsof -i :5001
lsof -i :5002
lsof -i :5003
```

2. **Stop conflicting services:**
```bash
docker-compose down
docker stop $(docker ps -q)  # Stop all containers
```

3. **Change ports in docker-compose.yml if needed**

## 🛠️ Docker and Devnet Issues

### Docker Build Failures

**Symptoms:**
- `docker-compose build` fails
- Missing dependencies or compilation errors

**Solutions:**

1. **Clean Docker state:**
```bash
docker-compose down --volumes --remove-orphans
docker system prune -a -f
```

2. **Rebuild without cache:**
```bash
docker-compose build --no-cache
```

3. **Check Dockerfile dependencies:**
- Ensure all system packages are installed
- Verify Rust toolchain is properly configured

### Container Startup Issues

**Symptoms:**
- Containers start but immediately exit
- No logs or minimal logging

**Debugging Steps:**

1. **Check container logs:**
```bash
docker-compose logs icn-node-a
docker-compose logs icn-node-b
docker-compose logs icn-node-c
```

2. **Run container interactively:**
```bash
docker run -it icn-devnet-icn-node-a /bin/bash
```

3. **Check entrypoint script:**
```bash
# Verify entrypoint.sh is executable and correct
cat icn-devnet/entrypoint.sh
```

### Database/Storage Issues

**Symptoms:**
- Nodes fail to start with database errors
- Data corruption or inconsistent state

**Solutions:**

1. **Reset storage:**
```bash
docker-compose down --volumes
# This removes all persistent data
```

2. **Check storage permissions:**
```bash
# Ensure data directories are writable
ls -la icn-devnet/data/
```

3. **Use memory backend for testing:**
```bash
# In docker-compose.yml:
ICN_STORAGE_BACKEND=memory
```

## 🔍 Debugging Commands

### Essential Debugging Commands

```bash
# Check overall devnet status
just devnet
just status

# View logs with filtering
docker-compose logs -f | grep -E "(ERROR|WARN|mana|job|initialized)"

# Check specific node logs
docker-compose logs icn-node-a | tail -50

# Test API endpoints
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/info
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/status
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/network/peers

# Check job states
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/mesh/jobs

# Monitor in real-time
watch 'curl -s -H "X-API-Key: devnet-a-key" http://localhost:5001/mesh/jobs | jq'
```

### Log Analysis Patterns

```bash
# Check for mana initialization
docker-compose logs icn-node-a | grep -i "mana.*initialized"

# Look for error patterns
docker-compose logs | grep -E "(ERROR|FATAL|panic|failed)"

# Network connectivity checks
docker-compose logs | grep -E "(peer|connect|discover)"

# Job processing flow
docker-compose logs | grep -E "(job|submit|bid|execute)"
```

## 📊 Performance Issues

### High Memory Usage

**Symptoms:**
- Nodes consume excessive memory
- System becomes unresponsive

**Solutions:**

1. **Monitor memory usage:**
```bash
docker stats
```

2. **Adjust container limits:**
```yaml
# In docker-compose.yml:
services:
  icn-node-a:
    mem_limit: 2g
    memswap_limit: 2g
```

3. **Use file-based storage:**
```bash
ICN_STORAGE_BACKEND=file
```

### Slow Performance

**Symptoms:**
- API responses are slow
- Job processing takes too long

**Debugging:**

1. **Check metrics:**
```bash
curl http://localhost:5001/metrics
```

2. **Profile with appropriate tools:**
```bash
# Enable detailed logging
RUST_LOG=debug cargo run
```

3. **Check for resource bottlenecks:**
```bash
htop
iotop
```

## 🚨 Emergency Recovery

### Complete System Reset

If everything is broken and you need to start fresh:

```bash
# 1. Stop everything
docker-compose down --volumes --remove-orphans

# 2. Clean Docker system
docker system prune -a -f
docker volume prune -f

# 3. Clean Rust build artifacts
cargo clean

# 4. Rebuild everything
docker-compose build --no-cache

# 5. Start fresh
docker-compose up -d

# 6. Verify operation
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/info
```

### Backup and Restore Data

```bash
# Backup node data
docker run --rm -v icn-devnet_node-a-data:/data -v $(pwd):/backup alpine tar czf /backup/node-a-backup.tar.gz /data

# Restore node data
docker run --rm -v icn-devnet_node-a-data:/data -v $(pwd):/backup alpine tar xzf /backup/node-a-backup.tar.gz -C /
```

## 🔍 Getting Help

If you encounter issues not covered here:

1. **Check existing issues:** [GitHub Issues](https://github.com/InterCooperative/icn-core/issues)
2. **Search discussions:** [GitHub Discussions](https://github.com/InterCooperative/icn-core/discussions)
3. **Create detailed bug reports** with:
   - Complete error messages
   - Steps to reproduce
   - System information (OS, Docker version, etc.)
   - Relevant log excerpts

4. **Include debugging information:**
```bash
# System info
uname -a
docker --version
docker-compose --version
cargo --version

# ICN version
git rev-parse HEAD
git describe --tags

# Runtime environment
env | grep ICN
env | grep RUST
```

---

**Remember:** When reporting issues, provide as much context as possible. The more detailed your bug report, the faster we can help resolve the issue. 