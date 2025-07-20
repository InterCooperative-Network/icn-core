# AGENTS.md

## ICN Core – AI Agent & Contributor Guide

**Welcome, AI agent or human contributor! This file is your comprehensive guide to working on ICN Core, the production-ready infrastructure for a cooperative digital economy.**

---

## 🌐 **Current Reality: What We Have Built**

The InterCooperative Network (ICN) Core is **production-ready infrastructure** (77-82% complete) that provides **working federated cooperative coordination** without relying on corporate cloud platforms or speculative blockchain systems.

### **Current Status: Phase 5 (Operational Excellence)**
This is **not a prototype**—it's working infrastructure with:
- ✅ **Real P2P networking** with verified multi-node federation
- ✅ **Cross-node job execution** with cryptographic verification
- ✅ **Democratic governance** with CCL compilation and voting
- ✅ **Economic systems** with mana regeneration and reputation
- ✅ **Complete API ecosystem** with 60+ endpoints and TypeScript SDK
- ✅ **Frontend applications** across Web/Mobile/Desktop platforms
- ✅ **Zero-knowledge privacy** with credential proofs and selective disclosure

### **What Communities Can Do Today**
1. **Deploy federations** with real P2P networking and automatic peer discovery
2. **Submit mesh jobs** that execute across multiple nodes with bidding and selection
3. **Create proposals** using CCL that compile to WASM for democratic governance
4. **Manage resources** using regenerating mana with anti-extraction properties
5. **Issue credentials** with zero-knowledge proofs for privacy-preserving verification
6. **Use comprehensive APIs** through TypeScript SDK or direct HTTP endpoints
7. **Run complete applications** for federation management, governance, and identity

---

## 🤖 **AI Agent Behavioral Expectations**

### **Core Agent Responsibilities**

**YOU ARE EXPECTED TO:**
- **Make frequent, atomic commits** with clear, descriptive messages
- **Update documentation continuously** as you make changes
- **Test thoroughly** before committing any changes
- **Roll back immediately** if you introduce bugs or unintended consequences
- **Follow established patterns** and architectural principles
- **Communicate clearly** about what you're doing and why
- **Maintain project quality** above all else

### **Commit Frequency & Documentation Standards**

#### **Commit Every Logical Change**
```bash
# Make small, focused commits
git add specific_file.rs
git commit -m "[icn-api] Add endpoint for federation health status

- Added GET /api/v1/federation/health endpoint
- Returns comprehensive federation metrics
- Includes peer count, job queue status, and mana distribution
- Added corresponding TypeScript SDK method
- Updated API documentation with examples"

# Update docs in same commit or immediately after
git add ICN_API_REFERENCE.md docs/api/
git commit -m "[docs] Document federation health endpoint

- Added federation health endpoint to API reference
- Included request/response examples
- Updated TypeScript SDK documentation
- Added health monitoring guide to operations docs"
```

#### **Documentation Update Requirements**
**EVERY CODE CHANGE MUST:**
1. **Update relevant documentation** (API reference, guides, README files)
2. **Include code examples** where applicable
3. **Update TypeScript SDK** if adding new endpoints
4. **Maintain consistency** across all documentation
5. **Keep navigation current** in DOCUMENTATION_INDEX.md

### **Error Handling & Recovery Protocols**

#### **When You Make a Mistake**
```bash
# Immediate rollback if you break something
git log --oneline -5  # See recent commits
git revert HEAD       # Revert the last commit
git commit -m "[fix] Revert changes that broke X - investigating issue"

# Or reset if you haven't pushed
git reset --hard HEAD~1  # ONLY if not pushed to remote
```

#### **Debugging Protocol**
1. **Reproduce the issue** in isolation
2. **Document the problem** clearly 
3. **Test fix thoroughly** before committing
4. **Verify related functionality** wasn't broken
5. **Update tests** to prevent regression

### **Code Quality Standards**

#### **Rust Development Standards**
```rust
// ALWAYS follow these patterns:

// 1. Comprehensive error handling
pub async fn submit_job(job: MeshJob, submitter: Did) -> Result<JobId, RuntimeError> {
    // Validate inputs
    validate_job_spec(&job)?;
    validate_submitter_credentials(&submitter).await?;
    
    // Check mana requirements
    let cost = estimate_job_cost(&job)?;
    ensure_sufficient_mana(&submitter, cost).await?;
    
    // Execute with proper error context
    let job_id = runtime_context
        .submit_mesh_job(job, submitter)
        .await
        .with_context(|| "Failed to submit job to runtime")?;
    
    // Update metrics and logs
    metrics::JOB_SUBMISSIONS.inc();
    info!("Job submitted successfully: {}", job_id);
    
    Ok(job_id)
}

// 2. Comprehensive documentation
/// Submits a mesh job for distributed execution across the network.
/// 
/// This function validates the job specification, checks mana requirements,
/// and adds the job to the pending queue for executor bidding.
/// 
/// # Arguments
/// * `job` - The job specification including compute requirements
/// * `submitter` - DID of the entity submitting the job
/// 
/// # Returns
/// * `Ok(JobId)` - Unique identifier for the submitted job
/// * `Err(RuntimeError)` - If validation fails or insufficient mana
/// 
/// # Examples
/// ```rust
/// let job = MeshJob::new("echo hello", ResourceRequirements::default());
/// let submitter = Did::from_str("did:icn:alice")?;
/// let job_id = submit_job(job, submitter).await?;
/// ```
```

#### **Frontend Development Standards**
```typescript
// TypeScript/React standards

// 1. Comprehensive type definitions
interface FederationHealthResponse {
  federation_id: string;
  peer_count: number;
  job_queue_length: number;
  mana_distribution: ManaDistribution;
  last_updated: string;
  status: 'healthy' | 'degraded' | 'critical';
}

// 2. Error boundary implementation
const FederationHealthComponent: React.FC = () => {
  const [health, setHealth] = useState<FederationHealthResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    const fetchHealth = async () => {
      try {
        const response = await icnSdk.federation.getHealth();
        setHealth(response);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
        // Log for debugging but don't break UI
        console.error('Failed to fetch federation health:', err);
      }
    };
    
    fetchHealth();
    const interval = setInterval(fetchHealth, 30000); // Update every 30s
    return () => clearInterval(interval);
  }, []);
  
  if (error) return <ErrorDisplay message={error} onRetry={() => window.location.reload()} />;
  if (!health) return <LoadingSpinner />;
  
  return <FederationHealthDisplay health={health} />;
};

// 3. Update TypeScript SDK simultaneously
// packages/ts-sdk/src/federation.ts
export class FederationClient {
  async getHealth(): Promise<FederationHealthResponse> {
    const response = await this.http.get('/api/v1/federation/health');
    return response.data;
  }
}
```

---

## 🏗️ **Complete Repository Architecture**

ICN Core is a **comprehensive monorepo** containing both deterministic Rust libraries and complete frontend applications across all platforms.

### **Backend Infrastructure (Rust)**

#### **Core Infrastructure (100% Complete)**
- **`icn-runtime`** – Node orchestration, WASM execution, job management
- **`icn-common`** – Shared types, cryptographic primitives, utilities
- **`icn-api`** – HTTP API definitions (60+ endpoints), TypeScript generation
- **`icn-protocol`** – P2P message formats and protocol definitions

#### **Identity & Security (95% Complete)**
- **`icn-identity`** – Complete DID lifecycle, credential verification, Ed25519 signatures
- **`icn-dag`** – Content-addressed storage (PostgreSQL, RocksDB, SQLite, Sled backends)
- **`icn-zk`** – Zero-knowledge circuits (age verification, membership proofs, reputation)

#### **Governance & Economics (82% Complete)**
- **`icn-governance`** – Proposal engine, voting mechanisms, CCL compilation
- **`icn-economics`** – Mana accounting, regeneration, economic policy enforcement
- **`icn-reputation`** – Trust scoring, contribution tracking, reputation algorithms
- **`icn-eventstore`** – Event sourcing utilities with JSON Lines format

#### **Networking & Computation (78% Complete)**
- **`icn-network`** – P2P networking with libp2p (Gossipsub, Kademlia DHT)
- **`icn-mesh`** – Distributed job scheduling, bidding, cross-node execution

#### **Developer Tools & SDKs (90% Complete)**
- **`icn-cli`** – Command-line interface for all operations
- **`icn-node`** – Main daemon binary with Axum HTTP server
- **`icn-sdk`** – High-level Rust SDK for HTTP API interactions
- **`icn-templates`** – Governance template management
- **`job-audit`** – Job auditing and compliance functionality

### **Frontend Applications**

#### **Cross-Platform Apps (React Native + Tamagui)**
- **`apps/wallet-ui`** (60% complete) – Secure DID and key management (iOS/Android/Web/Desktop)
- **`apps/agoranet`** (60% complete) – Governance deliberation platform (iOS/Android/Web/Desktop)

#### **Web Applications (React + TypeScript)**
- **`apps/web-ui`** (70% complete) – Federation administration dashboard with demo mode
- **`apps/explorer`** (65% complete) – DAG viewer and network browser with D3.js visualization

#### **Shared Frontend Infrastructure**
- **`packages/ui-kit`** (70% complete) – Cross-platform component library (Tamagui)
- **`packages/ts-sdk`** (80% complete) – TypeScript SDK with comprehensive API coverage
- **`packages/ccl-visual-editor`** (30% complete) – Visual contract editor (planned)

---

## 🎯 **Agent Authority & Current Focus**

### **Current Phase: Operational Excellence (Phase 5)**

**Key Insight**: The remaining 18-23% is primarily **configuration management** and **operational polish**, not missing core functionality. Production services exist and work—they need proper default configuration.

### **Immediate Priorities**
1. **Service Configuration**: Ensure production services are used by default
2. **Scale Testing**: Validate with 10+ node federations
3. **Frontend Completion**: Complete the 4 frontend applications
4. **Production Monitoring**: Add comprehensive observability
5. **Mobile Deployment**: Cross-platform app store deployment

### **You Are Empowered To:**
- **Complete configuration management** for production service defaults
- **Enhance frontend applications** with missing features and polish
- **Improve API endpoints** and TypeScript SDK coverage
- **Add production monitoring** and operational excellence tools
- **Optimize performance** for multi-node federation scenarios
- **Enhance security** and privacy features
- **Improve developer experience** and documentation

---

## 💻 **Agent Development Workflow**

### **Before Starting Any Work**

#### **1. Understand the Current State**
```bash
# Always start by understanding current status
git status                    # Check working directory
git log --oneline -10        # See recent changes
just validate                # Run full validation
just test                    # Ensure everything works
```

#### **2. Read Relevant Documentation**
- **Always read existing docs** before making changes
- **Understand the broader context** of your changes
- **Check for existing patterns** to follow
- **Look for related issues** or TODOs

#### **3. Plan Your Approach**
- **Break down complex changes** into smaller commits
- **Identify documentation** that will need updates
- **Consider testing requirements** for your changes
- **Think about potential side effects**

### **Development Cycle (Repeat for Every Change)**

#### **1. Make Small, Focused Changes**
```bash
# Work on ONE logical unit at a time
vim crates/icn-api/src/federation.rs  # Make specific change
cargo test -p icn-api                 # Test that specific crate
cargo clippy -p icn-api               # Check that specific crate
```

#### **2. Test Thoroughly**
```bash
# Test the specific change
cargo test function_you_changed

# Test the affected crate
cargo test -p affected-crate

# Test integration if needed
cargo test --test integration_test_name

# Test the full stack if it's a significant change
just validate-all-stack
```

#### **3. Update Documentation IMMEDIATELY**
```bash
# Update API documentation
vim ICN_API_REFERENCE.md

# Update relevant guides
vim docs/api/federation-management.md

# Update TypeScript SDK if needed
vim packages/ts-sdk/src/federation.ts

# Update examples if needed
vim examples/federation_health_check.rs
```

#### **4. Commit Changes**
```bash
# Stage specific changes
git add crates/icn-api/src/federation.rs
git add ICN_API_REFERENCE.md
git add packages/ts-sdk/src/federation.ts

# Make clear, descriptive commit
git commit -m "[icn-api] Add federation health monitoring endpoint

- Added GET /api/v1/federation/health for real-time status
- Returns peer count, job queue metrics, mana distribution
- Includes comprehensive error handling and validation
- Updated TypeScript SDK with typed response interface
- Added API documentation with request/response examples
- Includes health status categorization (healthy/degraded/critical)"
```

#### **5. Verify Your Changes**
```bash
# Ensure everything still works
just validate

# Check that docs are consistent
just docs

# Test the specific functionality
just test-federation-health  # If such a command exists
```

### **Continuous Documentation Maintenance**

#### **Update These Files for Every Relevant Change:**
- **`ICN_API_REFERENCE.md`** - For any API changes
- **`README.md`** - For significant new capabilities
- **`packages/ts-sdk/README.md`** - For SDK changes
- **Relevant guides in `docs/`** - For workflow changes
- **`DOCUMENTATION_INDEX.md`** - For new documentation

#### **Documentation Quality Standards:**
- **Include working examples** for all new APIs
- **Provide clear explanations** of purpose and usage
- **Maintain consistent formatting** and style
- **Link between related documentation** appropriately
- **Keep navigation up to date**

---

## 🚀 **Working Production Systems**

### **1. Multi-Node P2P Federation**
```
Real libp2p Networking ✅
├── Gossipsub messaging
├── Kademlia DHT peer discovery
├── Automatic federation bootstrap
└── Cross-federation coordination

Current: 3+ node networks verified
Goal: Scale to 10+ node federations
```

### **2. Cross-Node Mesh Computing**
```
Complete Job Pipeline ✅
├── Job submission (CLI/API/Web UI)
├── Network-wide bid collection
├── Reputation-based executor selection
├── WASM execution with security constraints
├── Cryptographic receipt generation
└── DAG anchoring and reputation updates

Current: Real cross-node execution working
Goal: Enhanced performance and monitoring
```

### **3. Democratic Governance System**
```
CCL-Powered Governance ✅
├── Proposal creation with CCL compilation
├── WASM policy execution
├── Voting with quorum enforcement
├── Delegation and revocation
├── Policy implementation
└── Complete audit trails

Current: 95% complete CCL system
Goal: Enhanced governance templates
```

### **4. Economic Resource Management**
```
Mana-Based Economics ✅
├── Time-based mana regeneration
├── Reputation-influenced rates
├── Resource accounting and enforcement
├── Multi-backend persistence
├── Token management system
└── Anti-extraction mechanisms

Current: Working across multiple backends
Goal: Enhanced economic policies
```

### **5. Comprehensive API Ecosystem**
```
Production HTTP API ✅
├── 60+ endpoints across all domains
├── TypeScript SDK with type safety
├── Authentication and authorization
├── Comprehensive error handling
├── Prometheus metrics integration
└── Real-time WebSocket support (planned)

Current: Most endpoints implemented
Goal: Complete TypeScript SDK coverage
```

### **6. Zero-Knowledge Privacy System**
```
ZK Credential Proofs ✅
├── Age verification circuits
├── Membership proof generation
├── Reputation threshold proofs
├── Selective credential disclosure
├── Batch verification
└── Privacy-preserving operations

Current: Core circuits implemented
Goal: Expanded proof system
```

---

## 🔧 **Configuration Management (Current Focus)**

### **Production Services Available**
| Component | Stub Service | Production Service | Status |
|-----------|--------------|-------------------|---------|
| **Mesh Networking** | `StubMeshNetworkService` | `DefaultMeshNetworkService` | ✅ Ready |
| **Cryptographic Signing** | `StubSigner` | `Ed25519Signer` | ✅ Ready |
| **DAG Storage** | `StubDagStore` | PostgreSQL/RocksDB/SQLite/Sled | ✅ Ready |
| **P2P Networking** | N/A | `LibP2pNetworkService` | ✅ In Use |
| **Governance** | N/A | `GovernanceModule` | ✅ In Use |

### **Key Agent Task: Service Selection**
```rust
// CURRENT ISSUE: Some contexts default to stub services
// YOUR JOB: Update to use production services by default

// BEFORE (using stub):
let mesh_network_service = Arc::new(StubMeshNetworkService::new());

// AFTER (using production):
#[cfg(feature = "enable-libp2p")]
let mesh_network_service = Arc::new(DefaultMeshNetworkService::new(libp2p_service));
#[cfg(not(feature = "enable-libp2p"))]
let mesh_network_service = Arc::new(StubMeshNetworkService::new());

// Better: Configuration-driven selection
let mesh_network_service = if config.enable_production_mesh {
    Arc::new(DefaultMeshNetworkService::new(libp2p_service))
} else {
    Arc::new(StubMeshNetworkService::new())
};
```

### **Agent Configuration Tasks:**
1. **Update service creation** to default to production services
2. **Add configuration options** for service selection
3. **Improve error handling** when production services fail
4. **Add health checks** for production service status
5. **Document configuration** options clearly

---

## 📱 **Frontend Application Development**

### **Technology Stack Understanding**
- **Cross-Platform**: React Native + Tamagui (iOS/Android/Web/Desktop)
- **Web-Only**: React + Vite + TypeScript + Tailwind CSS
- **Shared**: TypeScript SDK, UI component library

### **Frontend Development Workflow**
```bash
# Setup frontend environment
just setup-frontend          # One-time setup

# Development commands
just dev-frontend            # All apps simultaneously
just dev-web-ui             # Federation dashboard
just dev-explorer           # DAG viewer
just dev-wallet             # Identity management
just dev-agoranet           # Governance platform

# Cross-platform testing
just dev-mobile             # React Native (iOS/Android)
just dev-desktop            # Tauri desktop apps
```

### **Frontend Agent Responsibilities**
1. **Complete missing features** in each application
2. **Maintain TypeScript SDK** coverage
3. **Ensure responsive design** across all platforms
4. **Add real-time updates** where appropriate
5. **Improve user experience** and accessibility
6. **Keep component library** consistent

### **Frontend Code Standards**
```typescript
// Example: Adding a new federation management feature

// 1. Update TypeScript SDK first
// packages/ts-sdk/src/federation.ts
export interface FederationSettings {
  id: string;
  name: string;
  description: string;
  governance_policy: string;
  mana_distribution: ManaPolicy;
}

export class FederationClient {
  async updateSettings(settings: FederationSettings): Promise<void> {
    await this.http.put('/api/v1/federation/settings', settings);
  }
}

// 2. Add UI component
// apps/web-ui/src/components/FederationSettings.tsx
interface Props {
  federation: Federation;
  onUpdate: (settings: FederationSettings) => void;
}

export const FederationSettings: React.FC<Props> = ({ federation, onUpdate }) => {
  // Component implementation with proper error handling
  // Form validation, loading states, success/error feedback
};

// 3. Update the main app
// apps/web-ui/src/pages/FederationManagement.tsx
import { FederationSettings } from '../components/FederationSettings';

// 4. Add to UI kit if reusable
// packages/ui-kit/src/components/federation/
```

---

## 💡 **Agent Decision-Making Framework**

### **When to Make a Change**
✅ **DO make changes when:**
- Configuration can be improved for production readiness
- Documentation is outdated or missing
- Frontend features are incomplete but clearly defined
- API endpoints need better error handling
- TypeScript SDK coverage is missing
- Performance can be optimized without architectural changes
- Security can be enhanced with proven patterns

❌ **DON'T make changes when:**
- You're unsure about architectural implications
- Changes would affect core protocol behavior
- Major refactoring is needed without clear requirements
- Breaking changes would be introduced
- You lack context about why something was implemented a certain way

### **When in Doubt**
1. **Document the question** clearly in commit messages or comments
2. **Make minimal, reversible changes** first
3. **Test thoroughly** before proceeding
4. **Leave detailed comments** explaining your reasoning
5. **Create TODO items** for larger questions

### **Communication Patterns**
```bash
# Good commit messages explain reasoning
git commit -m "[icn-api] Update default timeout for federation requests

Changed from 5s to 30s based on observed network latency in production.
Multi-node federations were experiencing frequent timeouts during
proposal synchronization, especially with 5+ nodes.

Reasoning:
- Observed P2P message propagation takes 10-15s in real networks
- Governance proposals require consensus across all nodes
- Better to be conservative with timeouts in production

TODO: Add configurable timeout setting for different network sizes
Fixes: Timeout errors in multi-node proposal voting"
```

---

## 🧪 **Testing and Validation Expectations**

### **Pre-Commit Testing Requirements**
```bash
# ALWAYS run these before committing
just validate                 # Full validation suite
cargo test -p affected-crate  # Test your specific changes
cargo clippy -p affected-crate # Check for issues

# For frontend changes
just test-frontend           # Frontend test suite
just lint-frontend          # Frontend linting

# For significant changes
just devnet                 # Test with multi-node setup
just test-e2e              # End-to-end testing
```

### **Testing Philosophy**
- **Test the specific functionality** you're changing
- **Test integration points** that might be affected
- **Add new tests** for new functionality
- **Update existing tests** if behavior changes
- **Document test cases** that aren't obvious

### **When Tests Fail**
1. **Fix the issue immediately** - don't commit broken tests
2. **Understand why the test failed** - don't just make it pass
3. **Update tests if behavior changed intentionally**
4. **Add more tests** if you found a gap in coverage

---

## 🔍 **Quality Control and Review**

### **Self-Review Checklist**
Before committing, ask yourself:
- [ ] Does this change advance the project goals?
- [ ] Are all affected tests passing?
- [ ] Is documentation updated appropriately?
- [ ] Would another developer understand this change?
- [ ] Are there any potential security implications?
- [ ] Does this follow established patterns?
- [ ] Is error handling appropriate?
- [ ] Are performance implications considered?

### **Code Review Principles**
- **Be thorough but efficient** - catch issues early
- **Focus on correctness and maintainability**
- **Ensure documentation is accurate and helpful**
- **Check for security implications**
- **Verify testing is adequate**

---

## 📚 **Essential Reading for Agents**

### **Start Here (Updated Documentation)**
1. **[README.md](README.md)** – Complete project overview (77-82% complete status)
2. **[CONTEXT.md](CONTEXT.md)** – Full project context and philosophical foundation
3. **[ICN_API_REFERENCE.md](ICN_API_REFERENCE.md)** – All 60+ HTTP endpoints
4. **[docs/status/STATUS.md](docs/status/STATUS.md)** – Current implementation status

### **Architecture & Development**
5. **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** – System design and components
6. **[docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)** – Complete development workflow
7. **[.cursor/rules/](/.cursor/rules/)** – Comprehensive development rules
8. **[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)** – Complete navigation guide

### **Frontend Development**
9. **[apps/web-ui/README.md](apps/web-ui/README.md)** – Federation dashboard
10. **[packages/ts-sdk/README.md](packages/ts-sdk/README.md)** – TypeScript SDK
11. **[packages/ui-kit/README.md](packages/ui-kit/README.md)** – Component library

---

## 🌟 **Vision Alignment**

You're working on **production-ready infrastructure** that's already changing how communities coordinate. ICN Core isn't a future vision—it's working technology that enables:

- **Democratic governance** without centralized control
- **Economic coordination** without extraction
- **Resource sharing** across federation boundaries
- **Privacy preservation** through zero-knowledge proofs
- **Sovereign infrastructure** owned by communities

### **Current Impact**
- Communities can deploy **real federations** today
- Cooperatives can encode **bylaws as executable policy**
- Members can participate in **cryptographically verified governance**
- Resources can be shared using **regenerating mana economics**
- Privacy can be preserved through **zero-knowledge credentials**

### **Your Contribution Philosophy**
Every improvement you make to ICN Core directly enhances the infrastructure that cooperatives and communities use to coordinate democratically. You're not building software—you're building the foundation of a **cooperative digital economy**.

**Remember:**
- **Be thoughtful** about every change
- **Document thoroughly** as you work
- **Test comprehensively** before committing
- **Communicate clearly** about what you're doing
- **Maintain quality** above all else
- **Roll back if needed** without hesitation
- **Learn from mistakes** and improve continuously

---

**Thank you for contributing to production-ready cooperative infrastructure. Together, we're creating the tools that communities use today to govern themselves and coordinate resources without extraction or centralized control.**
