# GitHub Copilot Instructions for ICN Core

## Project Status & Context

ICN Core is an **advanced development cooperative infrastructure project** with substantial working implementations. While **NOT production-ready**, it has significantly more functional code than typical early-stage projects.

### Current Implementation Status
- **~65-75% implemented** across core domains
- **Real working features** including CCL WASM execution, multi-backend persistence, governance workflows
- **Advanced development phase** requiring security review and production hardening
- **Substantial codebase** with comprehensive functionality

## Architecture & Codebase Reality

### Working Implementations ✅
- **CCL Compiler**: Full WASM compilation pipeline working
- **Multi-Backend Storage**: PostgreSQL, RocksDB, SQLite, Sled all functional
- **P2P Networking**: libp2p with gossipsub and Kademlia DHT operational
- **Governance**: Ranked choice voting, proposals, budget allocation functional
- **Economics**: Mana ledgers, resource tokens, mutual credit systems working
- **Mesh Computing**: End-to-end job submission and execution pipeline
- **Identity**: DID creation, credential verification, signing working
- **Frontend Apps**: UI components connecting to real backend APIs

### Development Areas ⚠️
- **Security Review**: Cryptographic implementations need hardening
- **Scale Testing**: Works in development, needs production-scale validation
- **Operational Excellence**: Monitoring, recovery procedures needed
- **Documentation**: Implementation ahead of documentation

### Technology Stack
```
Backend: Rust with comprehensive trait-based architecture
Frontend: React/React Native + TypeScript + Tamagui
Storage: PostgreSQL/RocksDB/SQLite/Sled backends
Networking: libp2p (gossipsub, Kademlia DHT)
Contracts: CCL → WASM compilation and execution
```

## Contribution Guidelines

### 🎯 Focus Areas
1. **Security Hardening**: Review cryptographic implementations
2. **Production Readiness**: Add monitoring, error recovery, scale testing
3. **Feature Completion**: Finish partial implementations
4. **Documentation**: Update docs to match implementation reality
5. **Frontend Integration**: Connect UIs to working backend APIs

### ✅ Good Contributions
- Security improvements and reviews
- Performance optimizations
- Production monitoring and alerting
- Test coverage improvements
- Documentation updates
- Frontend polish and integration
- Scale testing and optimization

### ❌ Avoid
- Breaking existing working functionality
- Major architectural changes without discussion
- Security changes without expert review
- Reducing current implementation level

## Code Quality Standards

### Rust Backend
```rust
// Example: Real implementation pattern in the codebase
impl ManaLedger for SqliteManaLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        // Real implementation with error handling
        self.execute_query("SELECT balance FROM accounts WHERE did = ?", did)
            .unwrap_or(0)
    }
    
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        // Real persistence with proper error handling
        self.execute_update(
            "INSERT OR REPLACE INTO accounts (did, balance) VALUES (?, ?)",
            (did, amount)
        )
    }
}
```

### Frontend Integration
```typescript
// Example: Real API integration pattern
export class ICNClient {
    async submitJob(job: JobSpec): Promise<JobId> {
        // Real API call to working backend
        const response = await this.http.post('/api/v1/jobs', job);
        return response.data.job_id;
    }
    
    async getJobStatus(jobId: JobId): Promise<JobStatus> {
        // Real status monitoring
        const response = await this.http.get(`/api/v1/jobs/${jobId}`);
        return response.data;
    }
}
```

## Implementation Patterns to Follow

### Service Architecture
- Use trait-based interfaces from `icn-api`
- Support multiple backend implementations
- Comprehensive error handling with specific error types
- Async/await for all I/O operations
- Prometheus metrics for monitoring

### Testing Approach
- Unit tests for core logic
- Integration tests for cross-component functionality
- Feature flags to control stub vs real implementations
- Development configuration separate from production

### Documentation Standards
- Comprehensive rustdoc for public APIs
- Working examples in documentation
- Security considerations noted
- Performance characteristics documented

## Current Development Priorities

1. **Security Review & Hardening**
   - Review cryptographic implementations
   - Add security test cases
   - Implement production security measures

2. **Production Readiness**
   - Add comprehensive monitoring
   - Implement error recovery
   - Scale testing and optimization

3. **Feature Completion**
   - Finish partial implementations
   - Add missing edge case handling
   - Complete API coverage

4. **Documentation Updates**
   - Update docs to match implementation
   - Add deployment guides
   - Document security considerations

## Commands & Development

```bash
# Development setup
just setup && just build
just test                    # Run test suite
just validate               # Full validation

# Multi-node testing
just devnet                 # 3-node federation

# Frontend development
just dev-frontend           # All frontend apps
just dev-web-ui            # Federation dashboard
```

## Remember

- **This is advanced development software** with substantial working features
- **Security review required** before production use
- **Focus on production readiness** over new features
- **Maintain high code quality** standards
- **Document security implications** of changes
- **Test thoroughly** - working code should stay working
