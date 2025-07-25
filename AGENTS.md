# AGENTS.md

## ICN Core – AI Agent & Contributor Guide

**Welcome, AI agent or human contributor! This file is your comprehensive guide to working on ICN Core, an ambitious cooperative infrastructure project under active development.**

---

## 🚧 **Current Reality: Development Status**

The InterCooperative Network (ICN) Core is **experimental software under active development** that provides substantial cooperative infrastructure functionality. While **NOT production-ready**, it has **significantly more working implementation** than typical early-stage projects.

### **Current Status: Advanced Development (⚠️ NOT Production Ready)**
**Important**: Per the official [PROJECT_STATUS_AND_ROADMAP.md](PROJECT_STATUS_AND_ROADMAP.md), this project is:
- ⚠️ **Advanced Development** with substantial working implementations alongside development features
- ⚠️ **NOT Production Ready** - requires security review and operational hardening
- ⚠️ **Feature-Complete in Core Areas** - most functionality works but needs production polish
- ⚠️ **Security Review Needed** - cryptographic implementations need production hardening
- ⚠️ **Operational Excellence Required** - monitoring, recovery, and scale testing needed

### **Actual Implementation Progress** (Based on Codebase Review)
| Domain | Real Implementation | Development Features | Stub/TODO | Overall Status |
|--------|---------------------|---------------------|-----------|----------------|
| **Foundation** | Strong foundation, working utilities | Config management improvements needed | Few edge cases | **~75%** |
| **Mesh Computing** | CCL WASM execution, job pipelines working | Optimization, scale testing needed | Advanced algorithms | **~70%** |
| **Governance** | Ranked choice voting, budget proposals, federation governance | Policy execution optimization | Advanced workflows | **~65%** |
| **Economics** | Multi-backend ledgers, mana, resource tokens | Transaction optimization | Advanced policies | **~60%** |
| **Security** | Ed25519 signing, DID verification, receipts | Production hardening needed | HSM integration | **~55%** |
| **Networking** | libp2p integration, P2P messaging working | Scale optimization needed | Advanced routing | **~60%** |
| **Storage** | Multiple DAG backends, persistence working | Performance optimization | Advanced sync | **~65%** |

### **What Actually Works Today** (Beyond Experimental)
1. **CCL WASM Compilation & Execution** - Real contracts compile to WASM and execute
2. **Multi-backend Persistence** - PostgreSQL, RocksDB, SQLite, Sled all working
3. **Governance Workflows** - Proposals, voting, budget allocation actually function
4. **P2P Networking** - libp2p with gossipsub and Kademlia DHT operational
5. **Economic Transactions** - Mana accounting, resource tokens, mutual credit systems
6. **Job Execution Pipeline** - End-to-end mesh job submission and execution
7. **Identity Management** - DID creation, credential verification, signing working
8. **Frontend Applications** - UI components connecting to real backend APIs

### **Development vs Production Reality**
- **Core Functionality**: Most features have working implementations
- **Production Readiness**: Security review, monitoring, and operational procedures needed
- **Scale Testing**: Works in development, needs validation at production scale
- **Documentation**: Implementation is ahead of documentation in many areas

---

## 🤖 **AI Agent Behavioral Expectations**

### **Core Agent Responsibilities**

**YOU ARE EXPECTED TO:**
- **Be truthful** about project status and limitations
- **Make frequent, atomic commits** with clear, descriptive messages
- **Update documentation continuously** as you make changes
- **Test thoroughly** before committing any changes
- **Roll back immediately** if you introduce bugs or regressions
- **Follow established patterns** and architectural principles
- **Focus on replacing stub implementations** with real functionality
- **Maintain project quality** and help move toward production readiness

### **Development Status Awareness**

#### **Critical Understanding**
- This is **development software** - emphasize experimental nature
- Many features are **stubbed** - focus on implementing real functionality
- **Security is not production-ready** - flag security implications
- **APIs may change** - document breaking changes clearly
- **Data persistence may be unreliable** - warn about data loss risk

#### **Appropriate Language for Commits**
```bash
# Good - acknowledges development status
git commit -m "[icn-api] Add experimental federation health endpoint

DEVELOPMENT STATUS: This is a stub implementation that returns mock data.
TODO: Implement real health checking logic.

- Added GET /api/v1/federation/health endpoint structure
- Returns mock health data for development testing
- Added TypeScript SDK method (also returns mock data)
- Updated API documentation with experimental status
- Added TODO comments for production implementation"

# Good - focuses on moving from stub to implementation
git commit -m "[icn-mesh] Replace stub job bidding with initial implementation

PROGRESS: Moving from StubMeshNetworkService to partial implementation.
LIMITATION: Bidding algorithm is basic and needs optimization.

- Implemented basic bid collection mechanism
- Added timeout handling for bid responses
- Created bid evaluation criteria (needs refinement)
- Updated tests to cover basic bidding flow
- Marked advanced bidding features as TODO items"
```

### **Error Handling & Recovery Protocols**

#### **When You Make a Mistake**
```bash
# Immediate rollback if you break existing functionality
git log --oneline -5  # See recent commits
git revert HEAD       # Revert the last commit
git commit -m "[fix] Revert changes that broke X - investigating issue

The previous change introduced regression in Y functionality.
Rolling back to investigate and implement proper solution."

# Or reset if you haven't pushed (use carefully)
git reset --hard HEAD~1  # ONLY if not pushed to remote
```

#### **Development Debugging Protocol**
1. **Understand the stub/real service boundary** in the code you're changing
2. **Reproduce issues** with the actual development setup
3. **Document problems clearly** with development context
4. **Test fixes** in development environment before committing
5. **Verify related functionality** wasn't broken
6. **Add tests** to prevent regression as features become real

### **Code Quality Standards**

#### **Rust Development Standards**
```rust
// ALWAYS follow these patterns for development code:

// 1. Be explicit about development status
/// Submits a mesh job for distributed execution across the network.
/// 
/// **DEVELOPMENT STATUS**: This implementation is experimental and uses
/// stub services for some functionality. Not suitable for production use.
/// 
/// **CURRENT LIMITATIONS**:
/// - Bidding algorithm is simplified
/// - Error handling is incomplete
/// - Security validation is basic
/// 
/// # Arguments
/// * `job` - The job specification including compute requirements
/// * `submitter` - DID of the entity submitting the job
/// 
/// # Returns
/// * `Ok(JobId)` - Unique identifier for the submitted job
/// * `Err(RuntimeError)` - If validation fails or system error occurs
/// 
/// # Development Notes
/// This function needs improvement in the following areas:
/// - [ ] Implement real executor selection algorithm
/// - [ ] Add comprehensive input validation
/// - [ ] Implement proper error recovery
/// 
/// # Examples
/// ```rust
/// // Basic development usage (not production-ready)
/// let job = MeshJob::new("echo hello", ResourceRequirements::default());
/// let submitter = Did::from_str("did:icn:alice")?;
/// let job_id = submit_job(job, submitter).await?;
/// ```
pub async fn submit_job(job: MeshJob, submitter: Did) -> Result<JobId, RuntimeError> {
    // Development-level validation (needs enhancement)
    validate_job_spec_basic(&job)?;
    validate_submitter_credentials_basic(&submitter).await?;
    
    // TODO: Implement proper mana checking
    // Currently using stub implementation
    let cost = estimate_job_cost_stub(&job)?;
    if !check_sufficient_mana_stub(&submitter, cost).await? {
        return Err(RuntimeError::InsufficientMana {
            required: cost,
            available: 0, // TODO: Get real balance
        });
    }
    
    // Submit job with current implementation
    let job_id = runtime_context
        .submit_mesh_job(job, submitter)
        .await
        .with_context(|| "Failed to submit job - this is experimental functionality")?;
    
    // Basic metrics (TODO: implement comprehensive monitoring)
    debug!("Job submitted in development mode: {}", job_id);
    
    Ok(job_id)
}

// 2. Clear service implementation status
pub struct RuntimeContext {
    /// Current mesh service - may be stub or partial implementation
    mesh_service: Arc<dyn MeshNetworkService>,
    /// DAG store - multiple backends available but may have limitations
    dag_store: Arc<dyn DagStore>,
    /// Economic service - basic implementation, needs enhancement
    economics: Arc<dyn EconomicsService>,
}

impl RuntimeContext {
    /// Creates a new runtime context for development use.
    /// 
    /// **WARNING**: This uses development-level services that are not
    /// suitable for production environments.
    pub fn new() -> Result<Self, RuntimeError> {
        let mesh_service = Self::create_mesh_service()?;
        let dag_store = Self::create_dag_store()?;
        let economics = Self::create_economics_service()?;
        
        Ok(Self {
            mesh_service,
            dag_store,
            economics,
        })
    }
    
    fn create_mesh_service() -> Result<Arc<dyn MeshNetworkService>, RuntimeError> {
        // TODO: Make this configurable to choose between stub and real implementation
        #[cfg(feature = "enable-libp2p")]
        {
            // Use real libp2p service when feature is enabled
            Ok(Arc::new(DefaultMeshNetworkService::new()?))
        }
        #[cfg(not(feature = "enable-libp2p"))]
        {
            // Fall back to stub for development
            warn!("Using stub mesh network service - not suitable for real networking");
            Ok(Arc::new(StubMeshNetworkService::new()))
        }
    }
}
```

#### **Frontend Development Standards**
```typescript
// TypeScript/React standards for development phase

// 1. Clear development status in interfaces
interface FederationHealthResponse {
  federation_id: string;
  peer_count: number;
  job_queue_length: number;
  mana_distribution: ManaDistribution;
  last_updated: string;
  status: 'healthy' | 'degraded' | 'critical';
  
  // Development status fields
  _dev_status: 'mock_data' | 'partial_implementation' | 'real_data';
  _dev_warnings?: string[];
}

// 2. Development-aware error handling
const FederationHealthComponent: React.FC = () => {
  const [health, setHealth] = useState<FederationHealthResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isDevMode, setIsDevMode] = useState(true); // Default to dev mode
  
  useEffect(() => {
    const fetchHealth = async () => {
      try {
        const response = await icnSdk.federation.getHealth();
        setHealth(response);
        setError(null);
        
        // Warn user about development data
        if (response._dev_status === 'mock_data') {
          console.warn('DEVELOPMENT: Displaying mock health data');
        }
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : 'Unknown error';
        setError(`Development API Error: ${errorMsg}`);
        console.error('Failed to fetch federation health (development API):', err);
      }
    };
    
    fetchHealth();
    const interval = setInterval(fetchHealth, 30000);
    return () => clearInterval(interval);
  }, []);
  
  if (error) {
    return (
      <ErrorDisplay 
        message={error} 
        onRetry={() => window.location.reload()}
        isDevelopment={isDevMode}
      />
    );
  }
  
  if (!health) return <LoadingSpinner />;
  
  return (
    <div>
      {health._dev_status === 'mock_data' && (
        <DevelopmentWarning message="This interface displays mock data for development purposes" />
      )}
      <FederationHealthDisplay health={health} />
    </div>
  );
};

// 3. Update TypeScript SDK with development awareness
// packages/ts-sdk/src/federation.ts
export class FederationClient {
  private isDevelopment: boolean = true; // Default to development mode
  
  async getHealth(): Promise<FederationHealthResponse> {
    const response = await this.http.get('/api/v1/federation/health');
    
    if (this.isDevelopment) {
      console.log('ICN Development SDK: API call to federation health endpoint');
      if (response.data._dev_status) {
        console.warn(`Development data status: ${response.data._dev_status}`);
      }
    }
    
    return response.data;
  }
}
```

---

## 🏗️ **Complete Repository Architecture**

ICN Core is a **comprehensive monorepo** containing experimental Rust libraries and frontend applications for cooperative infrastructure development.

### **Backend Infrastructure (Rust)**

#### **Core Infrastructure**
- **`icn-runtime`** – Node orchestration, WASM execution framework, job management (partial implementation)
- **`icn-common`** – Shared types, cryptographic primitives, utilities (foundational, mostly complete)
- **`icn-api`** – HTTP API definitions with 60+ endpoint structures (many return mock data)
- **`icn-protocol`** – P2P message formats and protocol definitions (basic implementation)

#### **Identity & Security (Development Phase)**
- **`icn-identity`** – DID lifecycle management, credential verification (needs security review)
- **`icn-dag`** – Content-addressed storage with multiple backends (PostgreSQL, RocksDB, SQLite, Sled - data models incomplete)
- **`icn-zk`** – Zero-knowledge proof frameworks (circuits designed, implementation varies)

#### **Governance & Economics (Early Development)**
- **`icn-governance`** – Proposal engine, voting mechanisms, CCL foundations (core structures defined, execution incomplete)
- **`icn-economics`** – Mana accounting concepts, economic policy framework (transaction logic needs implementation)
- **`icn-reputation`** – Trust scoring framework, contribution tracking (algorithms need implementation)
- **`icn-eventstore`** – Event sourcing utilities with JSON Lines format (basic implementation)

#### **Networking & Computation (Mixed Status)**
- **`icn-network`** – P2P networking with libp2p integration (basic messaging works, advanced features stubbed)
- **`icn-mesh`** – Distributed job scheduling concepts, bidding framework (scheduling algorithms stubbed)

#### **Developer Tools & SDKs**
- **`icn-cli`** – Command-line interface for development operations (basic commands work, some stubbed)
- **`icn-node`** – Main daemon binary with Axum HTTP server (development-level functionality)
- **`icn-sdk`** – High-level Rust SDK for HTTP API interactions (connects to development APIs)
- **`icn-templates`** – Governance template management (framework exists)
- **`job-audit`** – Job auditing and compliance functionality (early development)

### **Frontend Applications (Development Phase)**

#### **Cross-Platform Apps (React Native + Tamagui)**
- **`apps/wallet-ui`** – Secure DID and key management interface (development UI, backend integration limited)
- **`apps/agoranet`** – Governance deliberation platform (UI framework, backend integration partial)

#### **Web Applications (React + TypeScript)**
- **`apps/web-ui`** – Federation administration dashboard (development interface with demo mode)
- **`apps/explorer`** – DAG viewer and network browser (visualization framework, data integration limited)

#### **Shared Frontend Infrastructure**
- **`packages/ui-kit`** – Cross-platform component library with Tamagui (development components)
- **`packages/ts-sdk`** – TypeScript SDK with development API coverage (connects to experimental backends)
- **`packages/ccl-visual-editor`** – Visual contract editor (planned/early development)

---

## 🎯 **Agent Authority & Current Focus**

### **Current Phase: Active Development**

**Key Insight**: The project needs substantial work to move from experimental software to production-ready infrastructure. Focus on implementing real functionality behind the well-designed API surface.

### **Immediate Development Priorities**
1. **Replace Stub Implementations**: Convert stub services to working implementations
2. **Complete Core Algorithms**: Implement job scheduling, bidding, reputation calculation
3. **Security Hardening**: Conduct security review of cryptographic implementations
4. **Data Model Completion**: Finish persistence layer implementations
5. **API Integration**: Connect frontend applications to real backend services
6. **Testing Infrastructure**: Comprehensive testing for production readiness

### **You Are Empowered To:**
- **Replace stub implementations** with real functionality
- **Implement missing algorithms** for core features
- **Enhance security** and add proper validation
- **Complete data models** and persistence layers
- **Improve API endpoints** to return real data instead of mocks
- **Connect frontend applications** to working backend services
- **Add comprehensive testing** for reliability
- **Improve error handling** and edge case coverage
- **Optimize performance** for development and eventual production use
- **Enhance developer experience** and documentation accuracy

### **Critical Focus Areas**
1. **Mesh Computing**: Implement real job scheduling and executor selection algorithms
2. **Economics**: Complete mana transaction logic and economic policy enforcement
3. **Governance**: Finish voting mechanisms and proposal execution
4. **Security**: Conduct security review and harden cryptographic implementations
5. **Persistence**: Complete data model implementations and ensure data integrity

---

## 💻 **Agent Development Workflow**

### **Before Starting Any Work**

#### **1. Understand Current Implementation Status**
```bash
# Check project status
git status
git log --oneline -10
just validate                # Run validation suite
just test                   # Ensure tests pass

# Check for stub implementations in the area you're working on
grep -r "TODO\|STUB\|mock\|placeholder" crates/your-target-crate/
```

#### **2. Read Current Documentation** 
- **[README.md](README.md)** – Current development status and warnings
- **[PROJECT_STATUS_AND_ROADMAP.md](PROJECT_STATUS_AND_ROADMAP.md)** – Official project status and roadmap
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** – System design and component overview
- **[ICN_API_REFERENCE.md](ICN_API_REFERENCE.md)** – API endpoint documentation
- **[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)** – Navigation guide

#### **3. Identify Development Opportunities**
- **Look for stub implementations** that can be replaced with real functionality
- **Find TODO comments** that indicate missing features
- **Check for mock data returns** in API endpoints
- **Identify security review items** in cryptographic code
- **Find incomplete data models** in persistence layers

### **Development Cycle (Focus on Moving from Stub to Real)**

#### **1. Make Implementation Progress**
```bash
# Work on replacing stubs with real implementations
vim crates/icn-mesh/src/bidding.rs  # Replace stub bidding algorithm
cargo test -p icn-mesh              # Test the specific implementation
cargo clippy -p icn-mesh            # Check for issues
```

#### **2. Test Development Progress**
```bash
# Test specific functionality you're implementing
cargo test function_you_implemented

# Test the affected crate
cargo test -p affected-crate

# Test integration if moving beyond stubs
cargo test --test integration_test_name

# Run broader validation for significant changes
just validate
```

#### **3. Document Development Progress**
```bash
# Update documentation to reflect implementation progress
vim ICN_API_REFERENCE.md           # Update API docs if endpoint behavior changed
vim docs/implementation-status.md   # Update implementation status
vim packages/ts-sdk/README.md      # Update SDK docs if functionality changed

# Update code documentation
vim crates/your-crate/src/lib.rs   # Update module documentation
```

#### **4. Commit Development Progress**
```bash
# Stage implementation changes
git add crates/icn-mesh/src/bidding.rs
git add tests/integration/mesh_bidding_test.rs
git add docs/implementation-status.md

# Make clear commit about development progress
git commit -m "[icn-mesh] Implement basic job bidding algorithm

DEVELOPMENT PROGRESS: Replaced StubBiddingAlgorithm with working implementation.
STATUS: Basic functionality working, advanced features still needed.

Implementation changes:
- Replaced stub bidding logic with real bid collection
- Added timeout handling for bid responses  
- Implemented basic bid evaluation criteria
- Added integration test for end-to-end bidding

Limitations of current implementation:
- Bidding criteria is simplified (needs sophistication)
- Error handling is basic (needs comprehensive coverage)
- Performance not optimized (needs benchmarking)

Next steps:
- [ ] Implement advanced bid evaluation metrics
- [ ] Add comprehensive error recovery
- [ ] Optimize for larger node networks
- [ ] Security review of bid validation logic"
```

#### **5. Verify Development Changes**
```bash
# Ensure development environment still works
just validate
just test

# Check that documentation is consistent
just docs

# Test in development devnet if applicable
just devnet  # If this command exists for multi-node testing
```

### **Documentation Maintenance for Development Phase**

#### **Update These Files for Development Progress:**
- **`PROJECT_STATUS_AND_ROADMAP.md`** - For implementation status changes
- **`ICN_API_REFERENCE.md`** - When API endpoints move from mock to real data
- **`README.md`** - For significant functionality milestones
- **`packages/ts-sdk/README.md`** - When SDK gains real functionality
- **`docs/ARCHITECTURE.md`** - For architectural decisions during implementation
- **Implementation status files in `docs/status/`** - For progress tracking

#### **Development Documentation Standards:**
- **Be honest about implementation status** - distinguish between working, partial, and stubbed functionality
- **Include development warnings** where appropriate
- **Provide clear examples** that work with current implementation level
- **Document known limitations** and next steps for improvement
- **Link to related GitHub issues** for tracking progress

---

## 🚀 **Current Development State**

### **1. Basic Development Environment**
```
Development Setup ✅ (Working)
├── Multi-crate Rust workspace
├── Frontend applications buildable
├── CLI interface functional for basic operations
├── Devnet scripts available
└── Documentation structure established

Current: Development environment complete
Goal: Production-ready deployment configurations
```

### **2. Experimental P2P Networking**
```
Basic P2P Framework ⚠️ (Development)
├── libp2p integration established
├── Basic message passing works
├── Peer discovery functional in development
├── Gossipsub messaging basic implementation
└── Advanced features need implementation

Current: Basic networking for development testing
Goal: Production-ready networking with comprehensive features
```

### **3. Job Framework (Early Development)**
```
Job Pipeline Foundations ⚠️ (Partial)
├── Job submission API structure exists
├── Basic job data models defined
├── Stub implementations for bidding
├── Framework for executor selection
└── Receipt generation concepts designed

Current: Framework exists, algorithms need implementation
Goal: Full mesh computing pipeline with real scheduling
```

### **4. Governance Framework (Conceptual)**
```
Governance Foundations ⚠️ (Partial)
├── Proposal data structures defined
├── Voting framework designed
├── CCL language specification exists
├── Basic voting UI created
└── Policy execution needs implementation

Current: Structure and UI exist, core logic needs work
Goal: Full democratic governance with policy execution
```

### **5. Economic Concepts (Early Development)**
```
Economic Framework ⚠️ (Conceptual)
├── Mana account structures defined
├── Basic regeneration concepts
├── Policy framework designed
├── Transaction models outlined
└── Implementation logic needed

Current: Data models and concepts defined
Goal: Working economic system with real transactions
```

### **6. Development API Ecosystem**
```
API Development Framework ⚠️ (Partial)
├── HTTP endpoint structures defined
├── Request/response models exist
├── TypeScript SDK generated
├── Some endpoints return real data
└── Many endpoints return mock data

Current: API framework with mixed implementation levels
Goal: Comprehensive API with real backend integration
```

---

## 🔧 **Service Implementation Status (Focus Area)**

### **Stub Services Needing Implementation**
| Component | Current State | Implementation Priority | Complexity |
|-----------|---------------|------------------------|------------|
| **Job Bidding Algorithm** | `StubBiddingService` | 🔥 High | Medium |
| **Executor Selection** | Basic logic | 🔥 High | Medium |
| **Mana Transaction Logic** | Concept only | 🔥 High | High |
| **Reputation Calculation** | Framework only | 🔥 High | High |
| **Policy Execution** | Stub implementation | 🔥 High | High |
| **Advanced P2P Features** | Basic messaging only | 🔶 Medium | High |

### **Implementation Strategy**
```rust
// FOCUS: Replace stub implementations with working code

// BEFORE (current state):
pub struct StubBiddingService;
impl BiddingService for StubBiddingService {
    async fn collect_bids(&self, job: &MeshJob) -> Result<Vec<Bid>, BiddingError> {
        // TODO: Implement real bidding
        Ok(vec![]) // Returns empty - not functional
    }
}

// AFTER (development target):
pub struct RealtimeBiddingService {
    network: Arc<dyn NetworkService>,
    timeout: Duration,
    evaluation_criteria: BidEvaluationCriteria,
}

impl BiddingService for RealtimeBiddingService {
    async fn collect_bids(&self, job: &MeshJob) -> Result<Vec<Bid>, BiddingError> {
        // Real implementation that:
        // 1. Broadcasts job to network
        // 2. Collects responses within timeout
        // 3. Validates bid authenticity
        // 4. Returns ranked bids
        
        let bid_request = BidRequest::from_job(job);
        self.network.broadcast_job_announcement(bid_request).await?;
        
        let bids = self.collect_responses_with_timeout().await?;
        let validated_bids = self.validate_bids(bids).await?;
        
        Ok(validated_bids)
    }
}

// Configuration to choose between stub and real implementation
pub fn create_bidding_service(config: &RuntimeConfig) -> Result<Arc<dyn BiddingService>, RuntimeError> {
    match config.implementation_level {
        ImplementationLevel::Stub => {
            warn!("Using stub bidding service - not functional for real use");
            Ok(Arc::new(StubBiddingService))
        }
        ImplementationLevel::Development => {
            info!("Using development bidding service - basic functionality");
            Ok(Arc::new(RealtimeBiddingService::new_development()?))
        }
        ImplementationLevel::Production => {
            // Not available yet
            Err(RuntimeError::ProductionNotReady("Bidding service not production-ready"))
        }
    }
}
```

---

## 📱 **Frontend Development (Integration Focus)**

### **Current Frontend Status**
- **UI Components**: Development-level components exist and are functional
- **Backend Integration**: Limited - many connections to stub services
- **Data Flow**: Partial - some real data, some mock data
- **User Experience**: Development-quality, needs polish for production

### **Frontend Development Priorities**
```bash
# Setup development environment
just setup-frontend          # One-time setup for development

# Development workflow
just dev-frontend            # All apps in development mode
just dev-web-ui             # Federation dashboard (partial backend integration)
just dev-explorer           # DAG viewer (needs real DAG data)
just dev-wallet             # Identity management (needs security review)
just dev-agoranet           # Governance interface (needs governance backend)
```

### **Frontend Integration Tasks**
1. **Connect to Real APIs**: Replace mock data with actual backend calls
2. **Handle Development Limitations**: Graceful handling of partial implementations
3. **Add Development Indicators**: Show users when functionality is limited
4. **Improve Error Handling**: Better handling of development-phase errors
5. **Real-time Updates**: Implement when backend services support it

### **Frontend Development Standards**
```typescript
// Development phase frontend patterns

// 1. Handle mixed implementation levels gracefully
interface ApiResponse<T> {
  data: T;
  implementation_status: 'stub' | 'partial' | 'complete';
  limitations?: string[];
}

// 2. Provide development feedback to users
const FederationDashboard: React.FC = () => {
  const { data: federation, status, error } = useFederationData();
  
  return (
    <div>
      {status === 'stub' && (
        <DevelopmentNotice>
          This dashboard shows limited data from development APIs.
          Full functionality will be available in future releases.
        </DevelopmentNotice>
      )}
      
      {federation && <FederationDisplay data={federation} />}
      
      {error && (
        <ErrorDisplay 
          error={error} 
          context="development" 
          suggestions={[
            "Check that the development server is running",
            "Verify API endpoints are accessible",
            "See development documentation for troubleshooting"
          ]}
        />
      )}
    </div>
  );
};

// 3. TypeScript SDK with development awareness
export class ICNDevSDK {
  private readonly baseURL: string;
  private readonly developmentMode: boolean = true;
  
  constructor(baseURL: string) {
    this.baseURL = baseURL;
    
    if (this.developmentMode) {
      console.log('ICN SDK: Running in development mode');
      console.log('Some features may return mock data or have limited functionality');
    }
  }
  
  async submitJob(job: JobSpec): Promise<JobSubmissionResult> {
    try {
      const response = await fetch(`${this.baseURL}/api/v1/jobs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(job)
      });
      
      if (!response.ok) {
        throw new APIError(`Job submission failed: ${response.statusText}`, {
          status: response.status,
          context: 'development_api',
          suggestion: 'Check development server logs for details'
        });
      }
      
      const result = await response.json();
      
      if (this.developmentMode && result.implementation_status !== 'complete') {
        console.warn('Development API: Job submission using partial implementation');
        console.warn('Limitations:', result.limitations);
      }
      
      return result;
    } catch (error) {
      if (this.developmentMode) {
        console.error('Development API Error:', error);
        console.log('Common development issues:');
        console.log('- Development server not running');
        console.log('- API endpoint not implemented');
        console.log('- Backend service using stub implementation');
      }
      throw error;
    }
  }
}
```

---

## 💡 **Agent Decision-Making Framework**

### **When to Make Changes (Development Phase)**
✅ **DO make changes when:**
- Replacing stub implementations with working code
- Implementing missing algorithms or logic
- Adding real functionality behind existing API endpoints
- Improving error handling for partial implementations
- Adding comprehensive testing for new functionality
- Updating documentation to reflect implementation progress
- Enhancing security of cryptographic implementations
- Completing data models and persistence layers

❌ **DON'T make changes when:**
- You would introduce breaking changes without careful consideration
- Changes would reduce the current level of functionality
- Major architectural changes are needed without clear requirements
- You lack understanding of the broader system implications
- Security implications are unclear and need expert review

### **Development Phase Priorities**
1. **Replace stubs with basic working implementations**
2. **Complete missing core algorithms**
3. **Add comprehensive error handling**
4. **Implement real data persistence**
5. **Enhance security where possible**
6. **Add testing for reliability**

### **When in Doubt (Development Context)**
1. **Check existing TODO comments** for guidance on intended implementation
2. **Look for related stub services** that might provide clues
3. **Document your uncertainty** clearly in commit messages
4. **Make minimal, reversible changes** first
5. **Add comprehensive logging** for debugging during development
6. **Create GitHub issues** for complex implementation questions

### **Communication Patterns for Development**
```bash
# Good commit messages for development phase
git commit -m "[icn-economics] Implement basic mana transaction logic

DEVELOPMENT MILESTONE: Moving from stub to working mana transactions.
IMPLEMENTATION STATUS: Basic functionality working, advanced features needed.

Changes:
- Replaced StubManaLedger with SqliteManaLedger
- Implemented basic debit/credit operations
- Added transaction validation and balance checking
- Created migration scripts for development databases
- Added integration tests for basic transaction flows

Current limitations:
- No support for complex mana policies yet
- Error handling is basic (needs comprehensive coverage)
- Performance not optimized for production loads
- Security review needed for transaction validation

Next implementation steps:
- [ ] Add support for mana regeneration policies
- [ ] Implement advanced transaction types
- [ ] Add comprehensive audit logging
- [ ] Security review of balance calculation logic
- [ ] Performance testing and optimization

Related issues: #123 (mana policy implementation), #456 (transaction security)"
```

---

## 🧪 **Testing and Validation for Development Phase**

### **Development Testing Requirements**
```bash
# ALWAYS run these before committing
just validate                 # Full validation suite
cargo test -p affected-crate  # Test your specific implementation changes
cargo clippy -p affected-crate # Check for implementation issues

# For significant functionality implementation
cargo test --test integration_tests  # Integration testing
just test-devnet                     # Multi-node testing (if available)

# For frontend changes connecting to new backend functionality
just test-frontend          # Frontend test suite
just lint-frontend         # Frontend linting
```

### **Development Testing Philosophy**
- **Test new implementations thoroughly** - ensure they work as intended
- **Test integration points** between previously stubbed and real services
- **Add tests for error conditions** that weren't relevant for stubs
- **Test with development data** that reflects real usage patterns
- **Document test limitations** when testing against partial implementations

### **When Tests Fail in Development**
1. **Understand if failure is due to partial implementation** vs real bugs
2. **Check if test expectations match current implementation level**
3. **Update tests if moving from stub to real implementation changes behavior**
4. **Add more tests if implementation reveals new edge cases**
5. **Document test limitations** when testing against development APIs

---

## 🔍 **Quality Control for Development Phase**

### **Development Self-Review Checklist**
Before committing, ask yourself:
- [ ] Does this change advance the project toward production readiness?
- [ ] Are implementation level warnings clear for users?
- [ ] Is documentation updated to reflect the current implementation status?
- [ ] Would other developers understand the current limitations?
- [ ] Are security implications flagged appropriately?
- [ ] Does this follow established development patterns?
- [ ] Is error handling appropriate for the development phase?
- [ ] Are performance implications reasonable for development use?
- [ ] Is the change testable in the current development environment?

### **Code Review Principles for Development**
- **Focus on moving toward production readiness**
- **Ensure implementation progress is real, not just interface changes**
- **Check that documentation accurately reflects current status**
- **Verify that security implications are considered**
- **Confirm that testing is appropriate for implementation level**

---

## 📚 **Essential Reading for Development Contributors**

### **Start Here (Current Project Status)**
1. **[README.md](README.md)** – Development status and warnings
2. **[PROJECT_STATUS_AND_ROADMAP.md](PROJECT_STATUS_AND_ROADMAP.md)** – Official development roadmap and status
3. **[CONTEXT.md](CONTEXT.md)** – Project vision and philosophical foundation
4. **[ICN_API_REFERENCE.md](ICN_API_REFERENCE.md)** – API endpoints (note implementation status)

### **Architecture & Development Process**
5. **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** – System design and component relationships
6. **[docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)** – Development workflow and setup
7. **[.cursor/rules/](/.cursor/rules/)** – Comprehensive development rules and standards
8. **[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)** – Navigation guide for all documentation

### **Implementation Status & Roadmaps**
9. **[docs/SYSTEM_COMPLETENESS_ROADMAP.md](docs/SYSTEM_COMPLETENESS_ROADMAP.md)** – Completion roadmap
10. **[docs/status/](docs/status/)** – Implementation status tracking documents

### **Specific Development Areas**
11. **[docs/CCL_SPEC_0_1.md](docs/CCL_SPEC_0_1.md)** – Cooperative Contract Language specification
12. **[docs/governance-framework.md](docs/governance-framework.md)** – Governance system implementation
13. **[docs/MANA_REPUTATION_SYSTEM.md](docs/MANA_REPUTATION_SYSTEM.md)** – Economic system design

---

## 🌟 **Vision Alignment for Development Phase**

You're working on **ambitious experimental infrastructure** that aims to enable democratic coordination for communities and cooperatives. While ICN Core is not yet production-ready, it represents important work toward infrastructure independence.

### **Current Development Impact**
- **Experimental Platform**: Developers can experiment with cooperative infrastructure concepts
- **Learning Laboratory**: Contributors can explore decentralized governance and economics
- **Foundation Building**: Creating the building blocks for eventual production systems
- **Community Testing**: Early adopters can test concepts in development environments

### **Development Contribution Philosophy**
Every improvement you make to ICN Core moves the project closer to production readiness. You're not just writing code—you're building the foundation for communities to eventually coordinate democratically without relying on extractive platforms.

**Development Principles:**
- **Be honest** about current limitations and development status
- **Focus on substance** over cosmetic improvements
- **Build incrementally** toward production readiness
- **Document thoroughly** to help future contributors
- **Test comprehensively** to ensure reliability
- **Consider security implications** even in development
- **Learn from mistakes** and improve the development process

---

**Thank you for contributing to experimental cooperative infrastructure. While ICN Core is not yet production-ready, your work helps build the foundation for communities to eventually govern themselves and coordinate resources democratically.**

---

## 🔗 **Development Resources & Getting Help**

### **Community & Support**
- **GitHub Issues**: [https://github.com/InterCooperative-Network/icn-core/issues](https://github.com/InterCooperative-Network/icn-core/issues)
- **Discussions**: [https://github.com/InterCooperative-Network/icn-core/discussions](https://github.com/InterCooperative-Network/icn-core/discussions)
- **Contributing Guide**: [CONTRIBUTING.md](CONTRIBUTING.md)

### **Development Setup**
- **Quick Start**: Follow the setup instructions in [README.md](README.md)
- **Development Environment**: See [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)
- **Troubleshooting**: [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)

### **Understanding Current Implementation**
- **Search for "TODO"**: `grep -r "TODO" crates/` to find implementation opportunities
- **Search for "STUB"**: `grep -r "STUB\|Stub" crates/` to find stub implementations
- **Check test files**: Look at tests to understand expected behavior vs current implementation
- **Review API endpoints**: Check which ones return real data vs mock data

**Remember**: This is development software. Focus on building solid foundations and moving incrementally toward production readiness while being transparent about current limitations.
