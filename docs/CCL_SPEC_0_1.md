# Cooperative Contract Language (CCL) Specification v0.1

> **Legal Notice:** This specification defines a legally-binding programming language for enforceable contracts, governance rules, and economic agreements. Code written in CCL constitutes executable law.

---

## 1 · Language Overview

### Purpose
The Cooperative Contract Language (CCL) is a deterministic, verifiable programming language designed to encode legal contracts, governance systems, and economic rules. CCL serves as the foundational law engine for the InterCooperative Network (ICN), enabling communities, cooperatives, and federations to define, execute, and evolve their own legal frameworks.

### Design Principles
- **Deterministic Execution**: All CCL code produces identical results given identical inputs
- **Cryptographic Verification**: Every execution produces signed, auditable receipts
- **Scoped Authority**: Contracts operate within defined jurisdictional boundaries
- **Legal Binding**: CCL code constitutes enforceable law within its scope
- **Federation Compatible**: Contracts can join, leave, and interact across federations

### Core Philosophy
CCL replaces traditional legal infrastructure:
- **Contracts replace statutes**: Legal rules are explicit, versioned code
- **Proposals replace legislation**: Changes follow programmable democratic processes  
- **Execution receipts replace court records**: Cryptographic proof of legal actions
- **Federations replace jurisdictions**: Opt-in, programmable governance boundaries

### CCL v0.1 Core Principles

> **Note:** In v0.1, all voting and governance rights are derived strictly from membership, never from token holding or reputation. Mana is computation-only. Delegation is via explicit, non-tradable representation tokens.

| **Principle** | **Source** | **Purpose** |
|---------------|------------|-------------|
| **Voting Rights** | Membership | Only members may vote or propose |
| **Proposals** | Membership | Only members may submit proposals |
| **Mana** | Computation | Rate-limiting and execution costs only |
| **Tokens** | Value/Access/Delegation | Economic value and explicit delegation |
| **Reputation** | Trust/Incentives | Social trust and mana regeneration bonuses |
| **Privacy** | ZKP/Consent | Anonymous participation with verification |
| **Federation** | Chain of Trust (VC) | Verifiable credentials across scopes |

**Critical Distinctions:**
- **Mana** is the exclusive meter for computational work and rate-limiting
- **Tokens** represent economic value, access rights, or explicit delegation (not voting power)
- **Reputation** provides trust scoring and mana regeneration bonuses, but never reduces access below baseline
- **Membership** is the sole source of governance rights, proven by verifiable credentials

---

## 2 · Language Syntax

### 2.1 Lexical Elements

#### Identifiers
```ccl
// Valid identifiers
member_count
HousingCollective
calculate_mana
```

#### Literals
```ccl
// Integer
42
1000000

// Float  
3.14159
0.5

// String
"Housing Collective Brooklyn"
"local:brooklyn:district5"

// Boolean
true
false
```

#### Comments
```ccl
// Single-line comment
/* Multi-line
   comment */
```

### 2.2 Contract Structure

#### Basic Contract
```ccl
contract HousingCollective {
    scope: "local:brooklyn:district5"
    version: "1.0.0"
    
    // Contract body
}
```

#### Contract with Imports
```ccl
import "std::governance";
import "std::economics";
import "local:brooklyn::shared_resources";

contract CooperativeKitchen {
    scope: "local:brooklyn:kitchen"
    extends: SharedResources
    
    // Contract implementation
}
```

### 2.3 Role Definitions

#### Simple Role
```ccl
role Member {
    can: [vote, propose, view_financials]
}
```

#### Role with Conditions
```ccl
role Steward {
    can: [vote, propose, execute, manage_funds]
    requires: [
        member_since < now() - 6.months,
        reputation_score > 75
    ]
}
```

#### Hierarchical Roles
```ccl
role Admin extends Steward {
    can: [amend_contract, manage_roles]
    requires: [
        elected_by: Member,
        term_limit: 2.years
    ]
}
```

### 2.4 State Declarations

#### Basic State
```ccl
state treasury: token<USD>;
state member_count: int;
state active_proposals: [Proposal];
```

#### State with Initial Values
```ccl
state founded_date: timestamp = now();
state governance_threshold: float = 0.67;
state emergency_fund: token<USD> = token<USD>(10000);
```

### 2.5 Proposal Definitions

#### Simple Proposal
```ccl
proposal AllocateFunds {
    description: "Fund community kitchen renovation"
    amount: token<USD>(15000)
    recipient: did:coop:kitchen
    eligible: Member
    quorum: 60%
    threshold: 2/3
    duration: 7.days
    execution: {
        transfer(to: recipient, amount: amount);
        emit FundsAllocated { amount: amount, recipient: recipient };
    }
}
```

#### Complex Proposal with Multiple Stages
```ccl
proposal AmendGovernanceRules {
    description: "Update voting thresholds"
    eligible: Member
    stages: [
        Stage {
            name: "initial_vote"
            quorum: 50%
            threshold: majority
            duration: 5.days
        },
        Stage {
            name: "ratification"  
            quorum: 75%
            threshold: supermajority
            duration: 3.days
        }
    ]
    execution: {
        governance_threshold = new_threshold;
        emit GovernanceAmended { old_threshold: governance_threshold, new_threshold: new_threshold };
    }
}
```

### 2.6 Function Definitions

#### Pure Function
```ccl
fn calculate_mana_regeneration(base_rate: float, reputation: float) -> float {
    base_rate * (1.0 + reputation / 100.0)
}
```

#### State-Modifying Function
```ccl
fn add_member(new_member: did, initial_mana: token<Mana>) {
    require(caller_has_role(Admin));
    require(initial_mana >= token<Mana>(100));
    
    members.insert(new_member);
    mana_balances[new_member] = initial_mana;
    member_count += 1;
    
    emit MemberAdded { member: new_member, initial_mana: initial_mana };
}
```

---

## 3 · Type System

### 3.1 Primitive Types

| Type | Description | Example |
|------|-------------|---------|
| `int` | 64-bit signed integer | `42`, `-100` |
| `float` | 64-bit floating point | `3.14`, `0.5` |
| `bool` | Boolean value | `true`, `false` |
| `string` | UTF-8 string | `"Hello"` |
| `timestamp` | Unix timestamp | `now()`, `2024-01-01T00:00:00Z` |
| `did` | Decentralized Identifier | `did:key:alice` |
| `vc` | Verifiable Credential | Cryptographic credential |

### 3.2 Token Types

#### Basic Token
```ccl
token<USD>      // US Dollar token
token<Mana>     // ICN Mana token  
token<Credit>   // Mutual credit token
```

#### Token with Metadata
```ccl
token<USD> {
    decimals: 2,
    transferable: true,
    mintable: false
}
```

### 3.3 Composite Types

#### Arrays
```ccl
[int]           // Array of integers
[did]           // Array of DIDs
[Proposal]      // Array of proposals
```

#### Structs
```ccl
struct Project {
    title: string,
    budget: token<USD>,
    lead: did,
    deadline: timestamp,
    completed: bool
}
```

#### Enums
```ccl
enum VoteType {
    Majority,
    Supermajority(float),
    Unanimous,
    Quadratic
}
```

### 3.4 Role Types

```ccl
role Member {
    can: [vote, propose]
}

role Steward extends Member {
    can: [execute, manage_funds]
}
```

### 3.5 Scope Types

```ccl
scope Local = "local:brooklyn:district5";
scope Regional = "region:northeast";  
scope Global = "global";
```

---

## 4 · Execution Model

### 4.1 Deterministic Execution

#### Requirements
- No access to system time (use provided timestamps)
- No random number generation (use seeded PRNGs)
- No external I/O (use host-provided data)
- Deterministic floating-point operations

#### Example
```ccl
// FORBIDDEN - non-deterministic
fn bad_example() {
    let current_time = system_time(); // ERROR: non-deterministic
    let random_value = random(); // ERROR: unseeded random
}

// CORRECT - deterministic
fn good_example(provided_time: timestamp, seed: int) -> int {
    let prng = SeededRng::new(seed);
    prng.next_int()
}
```

### 4.2 WASM Compilation

CCL contracts compile to WebAssembly (WASM) for deterministic execution:

```
CCL Source → AST → Semantic Analysis → WASM Module
```

#### Host Interface
WASM modules interact with the ICN runtime through a defined Host ABI:

```rust
// Host functions available to CCL contracts
extern "C" {
    fn host_get_state(key: *const u8, key_len: usize) -> *const u8;
    fn host_set_state(key: *const u8, key_len: usize, value: *const u8, value_len: usize);
    fn host_emit_event(event_type: *const u8, data: *const u8, data_len: usize);
    fn host_transfer_tokens(from: *const u8, to: *const u8, amount: u64) -> bool;
    fn host_verify_signature(message: *const u8, signature: *const u8, public_key: *const u8) -> bool;
}
```

### 4.3 Execution Context

Every CCL execution receives a context containing:

```ccl
struct ExecutionContext {
    caller: did,                    // Who initiated this execution
    timestamp: timestamp,           // Current block time
    block_height: int,             // Current block number
    contract_address: address,      // This contract's address
    available_mana: token<Mana>,   // Caller's available mana
    scope: scope                   // Contract's scope
}
```

---

## 5 · Governance Primitives

### 5.1 Proposal Lifecycle

```ccl
enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Failed,
    Executed,
    Cancelled
}
```

#### Proposal Flow
```
Draft → Active → [Passed/Failed] → [Executed]
   ↓       ↓           ↓
Cancelled ← ←——————————————
```

### 5.2 Voting Mechanisms

#### Simple Majority
```ccl
proposal SimpleDecision {
    eligible: Member
    quorum: 25%
    threshold: majority
    duration: 3.days
}
```

#### Supermajority with High Quorum
```ccl
proposal ConstitutionalAmendment {
    eligible: Member
    quorum: 75%
    threshold: supermajority(2/3)
    duration: 14.days
}
```

#### Quadratic Voting
```ccl
proposal ResourceAllocation {
    eligible: Member
    vote_type: Quadratic
    max_votes_per_member: 100
    duration: 7.days
}
```

### 5.3 Membership and Credentials

Every contract defines membership explicitly. Only members may vote or propose. Membership is proven by verifiable credentials issued by communities, cooperatives, or federations.

#### Membership Credential Structure
```ccl
struct MembershipCredential {
    subject: did,           // The member's DID
    issuer: did,            // The issuing community/cooperative DID
    credential_type: string, // "membership", "role", etc.
    scope: string,          // Contract scope this applies to
    issued_at: timestamp,
    expires_at: Option<timestamp>,
    revoked: bool,
    signature: Signature    // Cryptographic proof from issuer
}
```

#### Membership Verification
```ccl
fn verify_membership(member: did, credential: MembershipCredential) -> bool {
    require(credential.subject == member);
    require(credential.scope == contract_scope());
    require(!credential.revoked);
    require(credential.expires_at.is_none() || credential.expires_at.unwrap() > now());
    
    // Verify cryptographic signature from issuer
    verify_credential_signature(credential)
}

fn caller_has_membership() -> bool {
    let credentials = get_member_credentials(caller());
    credentials.iter().any(|cred| verify_membership(caller(), cred.clone()))
}
```

### 5.4 Delegation via Representation Tokens

Delegation is achieved through explicit, non-tradable, time-limited representation tokens rather than direct vote delegation.

#### Representation Token
```ccl
struct RepresentationToken {
    delegator: did,
    delegate: did,
    scope: string,           // What decisions this covers
    issued_at: timestamp,
    expires_at: timestamp,
    revocable: bool,
    max_decisions: Option<int>, // Limit number of decisions
    used_decisions: int
}
```

#### Issue Representation Token
```ccl
fn issue_representation(
    delegate_to: did, 
    scope: string, 
    duration: Duration,
    max_decisions: Option<int>
) -> RepresentationToken {
    require(caller_has_membership());
    require(verify_membership(delegate_to, get_member_credentials(delegate_to)));
    
    let token = RepresentationToken {
        delegator: caller(),
        delegate: delegate_to,
        scope: scope,
        issued_at: now(),
        expires_at: now() + duration,
        revocable: true,
        max_decisions: max_decisions,
        used_decisions: 0
    };
    
    representation_tokens[caller()].push(token);
    
    emit RepresentationIssued { 
        delegator: caller(), 
        delegate: delegate_to, 
        scope: scope,
        expires_at: token.expires_at
    };
    
    token
}

fn revoke_representation(delegate: did, scope: string) {
    let tokens = &mut representation_tokens[caller()];
    for token in tokens {
        if token.delegate == delegate && token.scope == scope {
            token.revocable = false; // Mark as revoked
            emit RepresentationRevoked {
                delegator: caller(),
                delegate: delegate,
                scope: scope
            };
        }
    }
}
```

#### Liquid Democracy
```ccl
fn delegate_with_chain(delegate_to: did, max_chain_length: int) {
    require(delegation_chain_length(delegate_to) < max_chain_length);
    
    delegations[caller()] = Delegation {
        delegate: delegate_to,
        scope: "all",
        expires: never,
        revocable: true
    };
}
```

---

## 6 · Economic Primitives

### 6.1 Token Operations

#### Transfer
```ccl
fn transfer_tokens(to: did, amount: token<USD>) {
    require(balances[caller()] >= amount);
    
    balances[caller()] -= amount;
    balances[to] += amount;
    
    emit Transfer { from: caller(), to: to, amount: amount };
}
```

#### Minting (Controlled)
```ccl
fn mint_tokens(to: did, amount: token<USD>) {
    require(caller_has_role(Treasurer));
    require(amount <= monthly_mint_limit);
    
    total_supply += amount;
    balances[to] += amount;
    
    emit Mint { to: to, amount: amount };
}
```

#### Burning
```ccl
fn burn_tokens(amount: token<USD>) {
    require(balances[caller()] >= amount);
    
    balances[caller()] -= amount;
    total_supply -= amount;
    
    emit Burn { from: caller(), amount: amount };
}
```

### 6.2 Mana System

Mana is a regenerating compute resource that governs execution throughput and rate-limiting. 

**Key Principles:**
- Every member receives a **base mana regeneration rate** based on their membership status
- Reputation may **increase** mana regeneration rate as an incentive bonus
- Reputation may **never reduce** mana regeneration below the baseline
- No member can be blocked from participation due to low mana or reputation

#### Mana Regeneration
```ccl
fn calculate_mana_regen(account: did) -> token<Mana> {
    let base_rate = get_membership_base_rate(account);  // Based on membership
    let reputation_bonus = get_reputation_bonus(account);  // Always >= 0
    let time_factor = (now() - last_regen[account]) / 1.hour;
    
    (base_rate + reputation_bonus) * time_factor
}

fn get_reputation_bonus(account: did) -> token<Mana> {
    let reputation = reputation_scores[account];
    // Reputation can only add bonus, never subtract from base
    max(0, reputation * mana_config.reputation_bonus_rate)
}
```

#### Mana Spending (Computation Only)
```ccl
fn spend_mana(account: did, amount: token<Mana>, action: string) -> bool {
    require(is_computational_action(action)); // Only for execution costs
    let available = current_mana_balance(account);
    
    if available >= amount {
        mana_balances[account] = available - amount;
        emit ManaSpent { 
            account: account, 
            amount: amount, 
            action: action 
        };
        true
    } else {
        false
    }
}
```

### 6.3 Mutual Credit

```ccl
fn mutual_credit_transfer(from: did, to: did, amount: token<Credit>) {
    let from_balance = credit_balances[from];
    let credit_limit = credit_limits[from];
    
    require(from_balance - amount >= -credit_limit);
    
    credit_balances[from] -= amount;
    credit_balances[to] += amount;
    
    emit CreditTransfer { from: from, to: to, amount: amount };
}
```

---

## 7 · Federation System

### 7.1 Federation Structure

#### Basic Federation
```ccl
contract DefaultFederation {
    scope: "global"
    discoverable: true
    auto_enroll: true
    
    state member_contracts: [address];
    state federation_rules: GovernanceRules;
    
    fn join_federation(contract_addr: address) {
        require(is_valid_contract(contract_addr));
        member_contracts.push(contract_addr);
        emit FederationJoined { contract: contract_addr };
    }
    
    fn leave_federation(contract_addr: address) {
        require(caller() == contract_addr);
        member_contracts.remove(contract_addr);
        emit FederationLeft { contract: contract_addr };
    }
}
```

### 7.2 Cross-Federation Interaction

#### Delegated Governance
```ccl
proposal CrossFederationProposal {
    target_federation: address
    local_threshold: majority
    requires_ratification: true
    
    execution: {
        // First pass locally
        if local_vote_passes() {
            // Then submit to parent federation
            submit_to_federation(target_federation, self);
        }
    }
}
```

#### Discovery Protocol
```ccl
fn discover_federations(scope_filter: string) -> [FederationInfo] {
    let mut results = [];
    
    for federation in known_federations {
        if federation.scope.matches(scope_filter) && federation.discoverable {
            results.push(FederationInfo {
                address: federation.address,
                scope: federation.scope,
                member_count: federation.member_contracts.len(),
                governance_type: federation.governance_type
            });
        }
    }
    
    results
}
```

---

## 8 · Standard Library

### 8.1 std::membership

#### Membership Verification
```ccl
use std::membership::{
    verify_membership,
    issue_membership_credential,
    revoke_membership,
    get_member_credentials,
    MembershipCredential
};

fn check_voting_eligibility(member: did) -> bool {
    let credentials = get_member_credentials(member);
    credentials.iter().any(|cred| 
        verify_membership(member, cred.clone()) && 
        cred.scope == contract_scope()
    )
}
```

### 8.2 std::governance

#### Voting Functions
```ccl
use std::governance::{
    calculate_quorum,
    tally_votes,
    check_threshold,
    VoteType,
    QuorumRequirement
};

fn execute_if_passed(proposal_id: int) {
    let proposal = proposals[proposal_id];
    let votes = vote_tallies[proposal_id];
    
    if calculate_quorum(votes, total_eligible) >= proposal.quorum &&
       check_threshold(votes, proposal.threshold) {
        proposal.execution();
        proposals[proposal_id].status = ProposalStatus::Executed;
    }
}
```

#### Representation Management
```ccl
use std::governance::representation::{
    issue_representation_token,
    revoke_representation,
    get_active_representations,
    RepresentationToken
};
```

### 8.3 std::economics

#### Token Utilities (Value/Access/Delegation Only)
```ccl
use std::economics::{
    transfer_tokens,
    mint_tokens,
    burn_tokens,
    get_balance,
    TokenConfig
};
```

#### Mana Management (Computation Only)
```ccl
use std::economics::mana::{
    charge_mana,
    regenerate_mana,
    get_mana_balance,
    get_membership_base_rate,
    get_reputation_bonus,
    ManaConfig
};
```

### 8.4 std::identity

#### DID Operations
```ccl
use std::identity::{
    verify_did,
    resolve_did_document,
    check_credential,
    DidDocument,
    VerifiableCredential
};

fn verify_member_credential(member: did) -> bool {
    let credential = get_credential(member, "membership");
    check_credential(credential, trusted_issuers)
}
```

### 8.5 std::federation

#### Federation Management
```ccl
use std::federation::{
    join_federation,
    leave_federation,
    discover_federations,
    cross_federation_call
};
```

---

## 9 · Legal Binding Semantics

### 9.1 Cryptographic Evidence

Every CCL execution produces a **legal receipt** containing:

```ccl
struct LegalReceipt {
    contract_address: address,
    function_name: string,
    inputs: [u8],
    outputs: [u8],
    timestamp: timestamp,
    gas_used: int,
    signature: Signature,
    merkle_proof: MerkleProof
}
```

### 9.2 Evidentiary Standards

#### Authenticity
- **Code Integrity**: Contract source code is content-addressed (CID) in the DAG
- **Execution Proof**: Each function call produces a signed receipt
- **State Proof**: All state changes are cryptographically verified

#### Legal Equivalency
```ccl
// This CCL code is legally equivalent to:
// "Upon majority vote of members, the treasurer shall transfer 
//  $5000 to the kitchen collective for equipment purchase"

proposal KitchenEquipment {
    description: "Purchase kitchen equipment"
    amount: token<USD>(5000)
    recipient: did:coop:kitchen
    eligible: Member
    threshold: majority
    execution: {
        transfer(to: recipient, amount: amount);
    }
}
```

### 9.3 Amendment Process

All legal changes must go through the defined amendment process:

```ccl
proposal AmendContract {
    description: "Update voting threshold to 60%"
    target_section: "governance.voting_threshold" 
    new_value: 0.6
    eligible: Member
    threshold: supermajority(2/3)
    quorum: 75%
    
    execution: {
        // Create new contract version
        let new_contract = current_contract.clone();
        new_contract.governance.voting_threshold = new_value;
        
        // Deploy and transition
        deploy_contract(new_contract);
        emit ContractAmended { 
            old_version: current_version,
            new_version: new_contract.version,
            changes: [ChangeRecord { 
                section: target_section, 
                old_value: governance.voting_threshold,
                new_value: new_value 
            }]
        };
    }
}
```

---

## 10 · Compilation and Runtime

### 10.1 Compiler Pipeline

```
CCL Source Code
       ↓
   Lexical Analysis (Pest)
       ↓
   Syntactic Analysis (AST)
       ↓
   Semantic Analysis
       ↓
   Type Checking
       ↓
   WASM Code Generation
       ↓
   Runtime Deployment
```

### 10.2 Runtime Environment

#### Host Functions
The CCL runtime provides these host functions to WASM modules:

```rust
// State management
host_get_state(key: &str) -> Option<Vec<u8>>
host_set_state(key: &str, value: &[u8])

// Token operations  
host_transfer_tokens(from: Did, to: Did, amount: u64) -> Result<(), Error>
host_mint_tokens(to: Did, amount: u64) -> Result<(), Error>
host_burn_tokens(from: Did, amount: u64) -> Result<(), Error>

// Identity verification
host_verify_signature(message: &[u8], signature: &Signature, public_key: &PublicKey) -> bool
host_resolve_did(did: &Did) -> Option<DidDocument>

// Event emission
host_emit_event(event_type: &str, data: &[u8])

// Cross-contract calls
host_call_contract(address: Address, function: &str, args: &[u8]) -> Result<Vec<u8>, Error>
```

### 10.3 Gas Model

CCL uses **mana** instead of traditional gas:

```ccl
// Each operation costs mana
fn expensive_operation() {
    // Loops cost mana per iteration
    for i in 0..1000 {
        expensive_computation(); // 10 mana per call
    }
    
    // Storage operations cost mana
    state_variable = new_value; // 100 mana per state write
    
    // Token transfers cost mana
    transfer(to: recipient, amount: 1000); // 50 mana per transfer
}
```

---

## 11 · Error Handling and Recovery

### 11.1 Error Categories

CCL distinguishes between three types of errors:

#### Fatal Errors (Contract Termination)
```ccl
// These errors terminate execution immediately
require(caller_has_role(Admin));     // Access control violation
require(amount > 0);                 // Logic constraint violation
require(balance >= amount);          // Resource constraint violation
```

#### Recoverable Errors (Handled in Code)
```ccl
// These errors can be caught and handled
fn safe_transfer(to: did, amount: token<USD>) -> Result<(), TransferError> {
    match get_balance(caller()) {
        Ok(balance) if balance >= amount => {
            transfer(to: to, amount: amount);
            Ok(())
        },
        Ok(_) => Err(TransferError::InsufficientFunds),
        Err(e) => Err(TransferError::BalanceCheckFailed(e))
    }
}
```

#### Runtime Errors (System Level)
```ccl
// These are reported to the runtime for investigation
enum RuntimeError {
    OutOfMana,
    ContractNotFound,
    NetworkTimeout,
    StorageCorruption,
    InvalidSignature
}
```

### 11.2 Error Reporting and Legal Evidence

All errors generate legal receipts:

```ccl
struct ErrorReceipt {
    contract_address: address,
    function_name: string,
    error_type: ErrorType,
    error_message: string,
    caller: did,
    timestamp: timestamp,
    call_stack: [string],
    contract_state_hash: [u8; 32],
    signature: Signature
}
```

### 11.3 Error Recovery Patterns

#### Circuit Breaker for External Calls
```ccl
state circuit_breaker: CircuitState = CircuitState::Closed;
state failure_count: int = 0;

fn protected_external_call(target: address, function: string) -> Result<(), CallError> {
    match circuit_breaker {
        CircuitState::Open => Err(CallError::CircuitOpen),
        _ => {
            match call_external(target, function) {
                Ok(result) => {
                    failure_count = 0;
                    circuit_breaker = CircuitState::Closed;
                    Ok(result)
                },
                Err(e) => {
                    failure_count += 1;
                    if failure_count >= 5 {
                        circuit_breaker = CircuitState::Open;
                    }
                    Err(e)
                }
            }
        }
    }
}
```

---

## 12 · Contract Versioning and Upgrade Semantics

### 12.1 Contract Versioning

Every contract has explicit version metadata:

```ccl
contract HousingCollective {
    version: "2.1.0";
    previous_version: "2.0.5";
    upgrade_policy: UpgradePolicy::Democratic;
    
    // Contract implementation
}
```

### 12.2 Upgrade Mechanisms

#### Democratic Upgrade Process
```ccl
proposal UpgradeContract {
    new_contract_code: string;
    migration_plan: MigrationPlan;
    eligible: Member;
    threshold: supermajority(2/3);
    quorum: 75%;
    
    execution: {
        // Validate new contract
        let new_contract = compile_and_validate(new_contract_code)?;
        
        // Execute migration
        let migration_result = execute_migration(migration_plan, new_contract)?;
        
        // Deploy new version
        deploy_contract_upgrade(new_contract, migration_result)?;
        
        emit ContractUpgraded {
            old_version: version,
            new_version: new_contract.version,
            migration_hash: hash(migration_plan)
        };
    };
}
```

#### Migration Data Structures
```ccl
struct MigrationPlan {
    state_mappings: [(string, string)],    // old_field -> new_field
    data_transformations: [DataTransform], // Custom transformation logic
    backwards_compatibility: bool,
    rollback_plan: Option<RollbackPlan>
}

struct DataTransform {
    source_field: string,
    target_field: string,
    transform_function: string // CCL function name
}
```

#### Automated Migration Utilities
```ccl
// Built-in migration helpers
fn migrate_state<T, U>(old_state: T, transform: fn(T) -> U) -> U {
    transform(old_state)
}

fn preserve_balances(old_contract: address, new_contract: address) {
    for (account, balance) in old_contract.get_all_balances() {
        new_contract.set_balance(account, balance);
    }
}
```

### 12.3 Deprecation and Sunset Process

```ccl
proposal DeprecateContract {
    sunset_date: timestamp;
    replacement_contract: Option<address>;
    data_export_plan: ExportPlan;
    
    execution: {
        contract_status = ContractStatus::Deprecated;
        sunset_timestamp = sunset_date;
        
        // Begin read-only mode after sunset
        if now() > sunset_date {
            enable_read_only_mode();
        }
        
        emit ContractDeprecated {
            sunset_date: sunset_date,
            replacement: replacement_contract
        };
    };
}
```

---

## 13 · Interoperability and Legal Bridging

### 13.1 External Law Integration

CCL contracts can interface with traditional legal systems during transition periods:

#### Legal Entity Binding
```ccl
contract CooperativeLLC {
    scope: "local:california:sacramento";
    legal_entity: LegalEntity {
        jurisdiction: "California, USA",
        entity_type: "Limited Liability Company", 
        registration_number: "LLC-2024-001234",
        registered_agent: "Alice Cooper, Esq."
    };
    
    // CCL governance binds the legal entity
    proposal AmendByLaws {
        new_bylaws: string;
        eligible: Member;
        threshold: supermajority(2/3);
        
        execution: {
            // Update internal governance
            bylaws = new_bylaws;
            
            // Generate legal filing documents
            let filing_docs = generate_legal_documents(new_bylaws, legal_entity);
            
            emit LegalFilingRequired {
                documents: filing_docs,
                deadline: now() + 30.days,
                jurisdiction: legal_entity.jurisdiction
            };
        };
    }
}
```

#### Legal Evidence Export
```ccl
fn export_legal_evidence(case_id: string, date_range: (timestamp, timestamp)) -> LegalEvidence {
    let relevant_receipts = query_receipts(date_range);
    let governance_actions = query_governance_actions(date_range);
    
    LegalEvidence {
        case_id: case_id,
        contract_address: contract_address(),
        time_period: date_range,
        receipts: relevant_receipts,
        governance_actions: governance_actions,
        chain_of_custody: generate_custody_chain(relevant_receipts),
        legal_certification: generate_legal_cert()
    }
}
```

### 13.2 Cross-Jurisdictional Recognition

```ccl
// Recognition across different legal systems
struct LegalRecognition {
    recognizing_jurisdiction: string,
    recognized_actions: [ActionType],
    legal_weight: LegalWeight,
    conditions: [string]
}

enum LegalWeight {
    FullyBinding,
    PrimeFacieEvidence,
    SupportingEvidence,
    InformationalOnly
}
```

### 13.3 Regulatory Compliance Modules

```ccl
import "compliance::gdpr";
import "compliance::tax_reporting";
import "compliance::anti_money_laundering";

contract RegulatedCoop {
    compliance_modules: [
        ComplianceModule::GDPR,
        ComplianceModule::TaxReporting("US-IRS"),
        ComplianceModule::AML("FinCEN")
    ];
    
    fn process_member_data(member: did, data: PersonalData) {
        // GDPR compliance checks
        require(gdpr::has_consent(member, data.data_types));
        
        // Process data
        store_member_data(member, data);
        
        // Generate compliance audit trail
        emit ComplianceEvent {
            module: "GDPR",
            action: "DataProcessed",
            subject: member,
            legal_basis: "Consent"
        };
    }
}
```

---

## 14 · Security Test Scenarios and Edge Cases

### 14.1 Reentrancy Protection Tests

```ccl
// Test: Prevent reentrancy attacks
contract ReentrancyTest {
    state balances: map<did, token<USD>>;
    state reentrancy_guard: bool = false;
    
    fn vulnerable_withdraw(amount: token<USD>) {
        require(!reentrancy_guard);
        reentrancy_guard = true;
        
        let caller_balance = balances[caller()];
        require(caller_balance >= amount);
        
        // External call that could reenter
        external_transfer(caller(), amount);
        
        // State update after external call (vulnerable)
        balances[caller()] = caller_balance - amount;
        
        reentrancy_guard = false;
    }
    
    fn secure_withdraw(amount: token<USD>) {
        require(!reentrancy_guard);
        reentrancy_guard = true;
        
        let caller_balance = balances[caller()];
        require(caller_balance >= amount);
        
        // State update before external call (secure)
        balances[caller()] = caller_balance - amount;
        
        external_transfer(caller(), amount);
        
        reentrancy_guard = false;
    }
}
```

### 14.2 Token Overflow Protection

```ccl
// Test: Prevent integer overflow in token operations
fn safe_add_tokens(a: token<USD>, b: token<USD>) -> Result<token<USD>, MathError> {
    let result = a.checked_add(b)?;
    require(result >= a && result >= b); // Overflow check
    Ok(result)
}

fn safe_multiply_tokens(amount: token<USD>, multiplier: float) -> Result<token<USD>, MathError> {
    require(multiplier >= 0.0);
    require(multiplier <= MAX_MULTIPLIER);
    
    let result = amount.checked_mul(multiplier)?;
    require(result >= amount || multiplier == 0.0);
    Ok(result)
}
```

### 14.3 Multi-Level Delegation Abuse Prevention

```ccl
// Test: Prevent delegation chain manipulation
contract DelegationSecurity {
    state delegations: map<did, Delegation>;
    state delegation_history: [DelegationEvent];
    
    fn delegate_vote_secure(delegate_to: did, scope: string) {
        let caller_did = caller();
        
        // Prevent self-delegation
        require(caller_did != delegate_to);
        
        // Check delegation chain length
        let chain_length = calculate_delegation_chain_length(delegate_to);
        require(chain_length < MAX_DELEGATION_CHAIN);
        
        // Prevent circular delegation
        require(!creates_delegation_cycle(caller_did, delegate_to));
        
        // Rate limit delegation changes
        let recent_changes = count_recent_delegation_changes(caller_did, 24.hours);
        require(recent_changes < MAX_DELEGATION_CHANGES_PER_DAY);
        
        // Record delegation
        delegations[caller_did] = Delegation {
            delegate: delegate_to,
            scope: scope,
            timestamp: now(),
            revocable: true
        };
        
        // Audit trail
        delegation_history.push(DelegationEvent {
            delegator: caller_did,
            delegate: delegate_to,
            action: "Created",
            timestamp: now()
        });
    }
    
    fn calculate_delegation_chain_length(delegate: did) -> int {
        let mut current = delegate;
        let mut length = 0;
        let mut visited = [];
        
        while delegations.contains_key(current) && length < MAX_DELEGATION_CHAIN {
            if visited.contains(current) {
                // Cycle detected
                return MAX_DELEGATION_CHAIN;
            }
            
            visited.push(current);
            current = delegations[current].delegate;
            length += 1;
        }
        
        length
    }
}
```

### 14.4 Economic Attack Scenarios

```ccl
// Test: Prevent flash loan attacks and economic manipulation
contract EconomicSecurity {
    state price_history: [PricePoint];
    state trade_volume: map<timestamp, token<USD>>;
    
    fn secure_price_oracle() -> token<USD> {
        let recent_prices = get_recent_prices(1.hour);
        let median_price = calculate_median(recent_prices);
        let price_volatility = calculate_volatility(recent_prices);
        
        // Reject if price too volatile
        require(price_volatility < MAX_PRICE_VOLATILITY);
        
        // Use time-weighted average to prevent manipulation
        let twap = calculate_twap(recent_prices);
        require(abs(median_price - twap) < PRICE_DEVIATION_THRESHOLD);
        
        twap
    }
    
    fn detect_wash_trading(trader: did, time_window: timestamp) -> bool {
        let trades = get_trader_activity(trader, time_window);
        let self_trades = trades.filter(|t| t.buyer == t.seller);
        let self_trade_ratio = self_trades.len() as float / trades.len() as float;
        
        self_trade_ratio > WASH_TRADING_THRESHOLD
    }
}
```

---

## 15 · Standard Library Specification

### 15.1 Core Modules (Required)

#### std::membership
```ccl
module std::membership {
    // Membership verification (foundation of all governance rights)
    fn verify_membership(member: did, credential: MembershipCredential) -> bool;
    fn issue_membership_credential(issuer: did, subject: did, scope: string) -> MembershipCredential;
    fn revoke_membership(issuer: did, credential_id: string) -> bool;
    fn get_member_credentials(member: did) -> [MembershipCredential];
    
    // Membership queries
    fn is_member(member: did, scope: string) -> bool;
    fn get_membership_scope(member: did) -> [string];
    fn count_members(scope: string) -> int;
    
    // Types
    struct MembershipCredential {
        subject: did,
        issuer: did,
        credential_type: string,
        scope: string,
        issued_at: timestamp,
        expires_at: Option<timestamp>,
        revoked: bool,
        signature: Signature
    }
    
    // Version: 1.0.0
    // Upgradable: Security-critical, requires supermajority
}
```

#### std::governance
```ccl
module std::governance {
    // Required functions (membership-based only)
    fn calculate_quorum(votes: [Vote], eligible_members: [did]) -> bool;
    fn tally_votes(votes: [Vote]) -> VoteTally;
    fn check_threshold(tally: VoteTally, threshold: VoteThreshold) -> bool;
    fn verify_voting_eligibility(voter: did) -> bool; // Checks membership
    
    // Representation management (non-tradable delegation)
    fn issue_representation_token(delegator: did, delegate: did, scope: string) -> RepresentationToken;
    fn revoke_representation(delegator: did, delegate: did, scope: string) -> bool;
    fn get_active_representations(delegator: did) -> [RepresentationToken];
    
    // Types
    enum VoteType { Majority, Supermajority(float), Consensus, Unanimous, Quadratic }
    struct Vote { voter: did, choice: VoteChoice, timestamp: timestamp } // No weight field
    struct VoteTally { yes: int, no: int, abstain: int, total_members: int }
    
    // Version: 1.0.0
    // Upgradable: Yes, via governance proposal
}
```

#### std::economics  
```ccl
module std::economics {
    // Token operations (value/access/delegation only, NOT voting)
    fn transfer_tokens(from: did, to: did, amount: token<T>) -> Result<(), TransferError>;
    fn mint_tokens(to: did, amount: token<T>) -> Result<(), MintError>;
    fn burn_tokens(from: did, amount: token<T>) -> Result<(), BurnError>;
    fn get_balance(account: did) -> token<T>;
    
    // Mana management (computation and rate-limiting only)
    fn charge_mana(account: did, amount: token<Mana>, action: string) -> Result<(), ManaError>;
    fn regenerate_mana(account: did) -> token<Mana>;
    fn get_mana_balance(account: did) -> token<Mana>;
    fn get_membership_base_rate(account: did) -> token<Mana>; // Based on membership
    fn get_reputation_bonus(account: did) -> token<Mana>; // Always >= 0, bonus only
    
    // Mana policy enforcement
    fn is_computational_action(action: string) -> bool;
    fn calculate_mana_cost(action: string, params: [any]) -> token<Mana>;
    
    // Economic calculations
    fn calculate_tax(amount: token<T>, rate: float) -> token<T>;
    fn compound_interest(principal: token<T>, rate: float, periods: int) -> token<T>;
    
    // Version: 1.0.0
    // Upgradable: Via federation consensus
}
```

#### std::identity
```ccl
module std::identity {
    // DID operations
    fn verify_did(did: did) -> bool;
    fn resolve_did_document(did: did) -> Option<DidDocument>;
    fn create_did_from_key(public_key: PublicKey) -> did;
    
    // Credential management
    fn verify_credential(credential: VerifiableCredential, trusted_issuers: [did]) -> bool;
    fn issue_credential(issuer: did, subject: did, claims: [Claim]) -> VerifiableCredential;
    fn revoke_credential(credential_id: string, issuer: did) -> bool;
    
    // Signature operations
    fn verify_signature(message: [u8], signature: Signature, public_key: PublicKey) -> bool;
    fn sign_message(message: [u8], private_key: PrivateKey) -> Signature;
    
    // Version: 1.0.0  
    // Upgradable: Security-critical, requires supermajority
}
```

### 15.2 Extended Modules (Optional)

#### std::federation
```ccl
module std::federation {
    fn join_federation(federation: address) -> Result<(), FederationError>;
    fn leave_federation(federation: address) -> Result<(), FederationError>;
    fn discover_federations(scope_filter: string) -> [FederationInfo];
    fn cross_federation_call(target: address, function: string, args: [u8]) -> Result<[u8], CallError>;
    
    // Version: 1.0.0
    // Upgradable: Yes, with federation approval
}
```

#### std::reputation
```ccl
module std::reputation {
    fn calculate_reputation(account: did, time_window: timestamp) -> float;
    fn update_reputation(account: did, event: ReputationEvent) -> float;
    fn get_reputation_history(account: did) -> [ReputationPoint];
    
    // Version: 1.0.0
    // Upgradable: Algorithm updates via governance
}
```

#### std::time
```ccl
module std::time {
    fn parse_duration(duration_str: string) -> Duration;
    fn add_duration(timestamp: timestamp, duration: Duration) -> timestamp;
    fn time_until(target: timestamp) -> Duration;
    fn is_business_day(date: timestamp, jurisdiction: string) -> bool;
    
    // Version: 1.0.0
    // Upgradable: Timezone/calendar updates
}
```

### 15.3 Module Upgrade Policy

```ccl
// Standard library upgrade governance
proposal UpgradeStandardLibrary {
    module_name: string;
    new_version: string;
    upgrade_type: UpgradeType;
    
    execution: {
        match upgrade_type {
            UpgradeType::SecurityPatch => {
                // Immediate deployment, minimal approval
                require_threshold(majority);
                deploy_std_module(module_name, new_version);
            },
            UpgradeType::FeatureAddition => {
                // Standard approval process
                require_threshold(supermajority(0.6));
                deploy_std_module(module_name, new_version);
            },
            UpgradeType::BreakingChange => {
                // High bar for breaking changes
                require_threshold(supermajority(0.8));
                require_quorum(80%);
                deploy_std_module_with_migration(module_name, new_version);
            }
        }
    };
}
```

---

## 16 · Security Considerations

### 16.1 Determinism Enforcement

The compiler **rejects** non-deterministic operations:

```ccl
// FORBIDDEN - these will cause compilation errors
fn non_deterministic_operations() {
    let time = system_time();        // ERROR: Use provided timestamp
    let random = random_number();    // ERROR: Use seeded PRNG
    let file = read_file("config");  // ERROR: No file I/O
    let response = http_get("api");  // ERROR: No network I/O
}

// ALLOWED - deterministic alternatives
fn deterministic_operations(timestamp: timestamp, seed: int) {
    let time = timestamp;            // OK: Use provided timestamp
    let prng = SeededRng::new(seed); // OK: Seeded randomness
    let config = host_get_config();  // OK: Host-provided data
}
```

### 16.2 Access Control

```ccl
// Role-based access control
fn sensitive_operation() {
    require(caller_has_role(Admin));           // Role check
    require(caller_mana() >= token<Mana>(100)); // Mana check
    require(caller() != target);               // Self-operation check
    
    // Operation implementation
}
```

### 16.3 Reentrancy Protection

```ccl
state reentrancy_guard: bool = false;

fn protected_function() {
    require(!reentrancy_guard);
    reentrancy_guard = true;
    
    // External call that might reenter
    external_contract.some_function();
    
    reentrancy_guard = false;
}
```

---

## 17 · Conclusion

The Cooperative Contract Language (CCL) provides a complete framework for encoding legal contracts, governance systems, and economic rules as deterministic, verifiable code. By treating code as law, CCL enables communities and cooperatives to operate with transparent, enforceable agreements that evolve through democratic processes.

CCL's federation system allows for seamless scaling from local cooperatives to global governance networks, while maintaining the autonomy and self-determination of each participating community.

This specification serves as the foundation for implementing CCL compilers, runtimes, and tooling that will support the cooperative digital economy of the InterCooperative Network.

---

## 18 · Examples

### 18.1 Basic Housing Collective

```ccl
import "std::governance";
import "std::economics";

contract HousingCollective {
    scope: "local:brooklyn:district5"
    version: "1.0.0"
    
    role Member {
        can: [vote, propose, view_financials]
        requires: [background_check: true, deposit_paid: true]
    }
    
    role BoardMember extends Member {
        can: [execute_proposals, manage_finances]
        requires: [elected_by: Member, term_limit: 2.years]
    }
    
    state treasury: token<USD> = token<USD>(50000);
    state monthly_dues: token<USD> = token<USD>(800);
    state members: [did];
    state maintenance_fund: token<USD> = token<USD>(10000);
    
    proposal MaintenanceExpense {
        description: string
        amount: token<USD>
        vendor: did
        eligible: Member
        quorum: 40%
        threshold: majority
        
        execution: {
            require(amount <= maintenance_fund);
            transfer(to: vendor, amount: amount);
            maintenance_fund -= amount;
            emit MaintenanceExpenseApproved { 
                amount: amount, 
                vendor: vendor, 
                description: description 
            };
        }
    }
    
    proposal EmergencyExpense {
        description: string
        amount: token<USD>
        vendor: did
        eligible: BoardMember
        threshold: majority
        max_amount: token<USD>(5000)
        
        execution: {
            require(amount <= max_amount);
            transfer(to: vendor, amount: amount);
            treasury -= amount;
            emit EmergencyExpenseApproved { 
                amount: amount, 
                vendor: vendor 
            };
        }
    }
    
    fn pay_monthly_dues() {
        require(caller_has_role(Member));
        require(get_balance(caller()) >= monthly_dues);
        
        transfer(from: caller(), to: contract_address(), amount: monthly_dues);
        treasury += monthly_dues;
        
        emit DuesPaid { member: caller(), amount: monthly_dues, month: current_month() };
    }
    
    fn add_member(new_member: did, deposit: token<USD>) {
        require(caller_has_role(BoardMember));
        require(deposit >= token<USD>(2000));
        
        members.push(new_member);
        treasury += deposit;
        
        emit MemberAdded { member: new_member, deposit: deposit };
    }
}
```

### 18.2 Worker Cooperative

```ccl
import "std::governance";
import "std::economics";

contract TechWorkerCoop {
    scope: "local:san_francisco:tech_coop"
    version: "1.0.0"
    
    role WorkerOwner {
        can: [vote, propose, work, receive_patronage]
        requires: [probation_period_complete: true]
    }
    
    role Steward extends WorkerOwner {
        can: [hire, manage_projects, allocate_resources]
        requires: [elected_by: WorkerOwner, experience_years: 3]
    }
    
    state revenue_pool: token<USD>;
    state worker_hours: map<did, int>;
    state hourly_wages: map<did, token<USD>>;
    state patronage_pool: token<USD>;
    
    proposal HireWorker {
        candidate: did
        initial_wage: token<USD>
        eligible: WorkerOwner
        quorum: 60%
        threshold: consensus
        
        execution: {
            members.push(candidate);
            hourly_wages[candidate] = initial_wage;
            worker_hours[candidate] = 0;
            emit WorkerHired { worker: candidate, wage: initial_wage };
        }
    }
    
    proposal DistributePatronage {
        eligible: WorkerOwner
        threshold: majority
        frequency: quarterly
        
        execution: {
            let total_hours = sum(worker_hours.values());
            
            for (worker, hours) in worker_hours.iter() {
                let share = (hours as float) / (total_hours as float);
                let patronage = patronage_pool * share;
                
                transfer(to: worker, amount: patronage);
                emit PatronageDistributed { 
                    worker: worker, 
                    hours: hours, 
                    amount: patronage 
                };
            }
            
            patronage_pool = token<USD>(0);
            worker_hours.clear(); // Reset for next quarter
        }
    }
    
    fn log_work_hours(hours: int, project: string) {
        require(caller_has_role(WorkerOwner));
        require(hours > 0 && hours <= 40); // Max 40 hours per week
        
        worker_hours[caller()] += hours;
        let wage = hourly_wages[caller()] * hours;
        
        transfer(to: caller(), amount: wage);
        revenue_pool -= wage;
        
        emit WorkLogged { 
            worker: caller(), 
            hours: hours, 
            project: project, 
            wage: wage 
        };
    }
    
    fn record_revenue(amount: token<USD>, client: string) {
        require(caller_has_role(Steward));
        
        revenue_pool += amount;
        
        // Allocate to patronage (surplus after wages/expenses)
        let patronage_allocation = amount * 0.3; // 30% to patronage
        patronage_pool += patronage_allocation;
        
        emit RevenueRecorded { 
            amount: amount, 
            client: client, 
            patronage_allocated: patronage_allocation 
        };
    }
}
```

### 18.3 Regional Federation

```ccl
import "std::federation";
import "std::governance";

contract NortheastCoopFederation {
    scope: "region:northeast"
    version: "1.0.0"
    
    role MemberCoop {
        can: [participate_in_federation, receive_support]
        requires: [validated_coop: true, democratic_governance: true]
    }
    
    role FederationDelegate {
        can: [vote_on_federation_matters, represent_coop]
        requires: [elected_by_coop: true, term_limit: 1.year]
    }
    
    state member_coops: [address];
    state mutual_aid_fund: token<USD>;
    state shared_resources: [Resource];
    
    struct Resource {
        name: string,
        owner_coop: address,
        available: bool,
        cost_per_use: token<USD>
    }
    
    proposal AddMemberCoop {
        candidate_coop: address
        sponsor_coop: address
        eligible: FederationDelegate
        quorum: 70%
        threshold: consensus
        
        execution: {
            member_coops.push(candidate_coop);
            emit CoopAdded { 
                coop: candidate_coop, 
                sponsor: sponsor_coop 
            };
        }
    }
    
    proposal MutualAidRequest {
        requesting_coop: address
        amount_needed: token<USD>
        reason: string
        repayment_plan: string
        eligible: FederationDelegate
        quorum: 50%
        threshold: majority
        
        execution: {
            require(mutual_aid_fund >= amount_needed);
            require(is_member_coop(requesting_coop));
            
            transfer(to: requesting_coop, amount: amount_needed);
            mutual_aid_fund -= amount_needed;
            
            emit MutualAidProvided { 
                recipient: requesting_coop, 
                amount: amount_needed, 
                reason: reason 
            };
        }
    }
    
    fn share_resource(resource_name: string, cost_per_use: token<USD>) {
        require(is_member_coop(caller()));
        
        shared_resources.push(Resource {
            name: resource_name,
            owner_coop: caller(),
            available: true,
            cost_per_use: cost_per_use
        });
        
        emit ResourceShared { 
            owner: caller(), 
            resource: resource_name, 
            cost: cost_per_use 
        };
    }
    
    fn use_shared_resource(resource_name: string) {
        require(is_member_coop(caller()));
        
        let resource = find_resource(resource_name);
        require(resource.available);
        require(resource.owner_coop != caller()); // Can't use your own resource
        
        let cost = resource.cost_per_use;
        transfer(to: resource.owner_coop, amount: cost);
        
        emit ResourceUsed { 
            user: caller(), 
            owner: resource.owner_coop, 
            resource: resource_name, 
            cost: cost 
        };
    }
}
```

---

## 19 · Privacy, Confidentiality, and Consent

### 19.1 Private State and Redacted Records

Contracts may define private state variables that are only readable by authorized roles or via explicit audit triggers.

```ccl
state member_private_data: map<did, PersonalData> private;

fn get_my_private_data() -> PersonalData {
    require(caller() == self);
    return member_private_data[caller()];
}

fn audit_member_data(target: did) -> PersonalData {
    require(caller_has_role(Auditor));
    emit AuditAccessed { 
        target: target, 
        auditor: caller(), 
        timestamp: now() 
    };
    return member_private_data[target];
}
```

**Privacy Rules:**
- All private data must be encrypted at rest (by default, with contract-scoped keys)
- Data access events generate audit receipts for forensic traceability
- Private state cannot be read by unauthorized roles or contracts
- Audit access requires explicit role authorization and generates permanent logs

### 19.2 Consent and Data Use Patterns

Explicit consent patterns can be encoded for any personal or sensitive operation.

```ccl
fn process_member_data(
    member: did, 
    data: PersonalData, 
    consent_token: ConsentToken
) {
    require(consent::has_valid_consent(member, data.data_type, consent_token));
    
    // Process data
    store_member_data(member, data);
    
    emit DataProcessed { 
        subject: member, 
        by: caller(), 
        timestamp: now(),
        purpose: data.processing_purpose
    };
}

fn revoke_consent(data_type: DataType, processor: did) {
    consent::revoke_consent(caller(), data_type, processor);
    
    emit ConsentRevoked {
        subject: caller(),
        data_type: data_type,
        processor: processor,
        timestamp: now()
    };
}
```

**Consent Framework:**
- **Opt-in by default**: No data processing without explicit consent
- **Purpose limitation**: Consent scoped to specific use cases
- **Revocable**: Subjects can withdraw consent at any time
- **Expiration**: Consent tokens have time limits and must be renewed
- **Granular**: Different consent levels for different data types

### 19.3 GDPR and Privacy Compliance

```ccl
import "compliance::gdpr";

fn gdpr_compliant_data_processing(
    subject: did,
    data: PersonalData,
    legal_basis: LegalBasis
) {
    require(gdpr::validate_legal_basis(subject, data.data_type, legal_basis));
    
    match legal_basis {
        LegalBasis::Consent => {
            require(gdpr::has_active_consent(subject, data.data_type));
        },
        LegalBasis::LegitimateInterest => {
            require(gdpr::legitimate_interest_assessment_passed(data.data_type));
        },
        LegalBasis::ContractualNecessity => {
            require(gdpr::required_for_contract_performance(subject, data.data_type));
        }
    }
    
    // Log processing activity
    gdpr::log_processing_activity(ProcessingRecord {
        subject: subject,
        data_type: data.data_type,
        processor: caller(),
        legal_basis: legal_basis,
        purpose: data.purpose,
        timestamp: now()
    });
    
    // Process data
    process_data_internal(subject, data);
}

fn handle_gdpr_subject_request(request_type: SubjectRightType, subject: did) {
    require(verify_subject_identity(subject));
    
    match request_type {
        SubjectRightType::Access => {
            let data = get_all_subject_data(subject);
            emit SubjectDataProvided { subject: subject, data_hash: hash(data) };
        },
        SubjectRightType::Rectification => {
            // Allow subject to correct their data
            enable_data_correction_mode(subject);
        },
        SubjectRightType::Erasure => {
            // Right to be forgotten (with limitations)
            schedule_data_erasure(subject);
        },
        SubjectRightType::Portability => {
            let portable_data = export_subject_data(subject);
            emit DataPortabilityProvided { subject: subject, export_hash: hash(portable_data) };
        }
    }
}
```

---

## 20 · Zero-Knowledge Proof (ZKP) and Selective Disclosure

### 20.1 ZKP-Backed Credential Checks

Contracts may require ZKPs instead of, or in addition to, normal credentials for access or voting.

```ccl
fn vote_on_secret_ballot(
    proposal_id: int,
    vote_choice: VoteChoice,
    membership_proof: ZKProof, 
    nullifier: Nullifier
) {
    require(zkp::verify_membership(membership_proof, "eligible_voters"));
    require(!zkp::nullifier_used(nullifier)); // Prevent double-vote
    
    zkp::mark_nullifier(nullifier);

    // Record anonymous vote
    proposal_votes[proposal_id].push(AnonymousVote {
        choice: vote_choice,
        nullifier: nullifier,
        timestamp: now()
    });
    
    emit AnonymousVoteCast { 
        proposal_id: proposal_id, 
        nullifier: nullifier, 
        timestamp: now() 
    };
}

fn prove_mana_sufficiency_anonymous(
    required_amount: token<Mana>,
    mana_proof: ZKProof,
    action: string
) -> bool {
    // Prove mana >= required_amount without revealing actual balance or identity
    let proof_valid = zkp::verify_range_proof(
        mana_proof,
        "mana_balance",
        min: required_amount.value(),
        max: None
    );
    
    if proof_valid {
        emit AnonymousManaProofVerified {
            required_amount: required_amount,
            action: action,
            timestamp: now()
        };
    }
    
    proof_valid
}
```

**ZKP Use Cases:**
- **Anonymous voting**: Prove membership eligibility without revealing identity
- **Anonymous mana usage**: Prove sufficient capacity without revealing balance or identity
- **Membership verification**: Prove credentials without revealing details
- **Selective disclosure**: Reveal only necessary information to auditors

> **Note:** ZKP functions focus on proving **membership credentials**, not token holdings. Anonymous participation is based on verified membership in communities and cooperatives.
- **Threshold proofs**: Prove values above/below limits without revealing amounts

### 20.2 Selective Disclosure

Data subjects can reveal only specific fields or attestations, even to auditors.

```ccl
fn prove_income_qualification(
    income_proof: ZKProof,
    threshold: token<USD>
) -> bool {
    // Prove income > threshold without disclosing actual value
    zkp::verify_range_proof(
        income_proof, 
        "annual_income", 
        min: threshold.value(),
        max: None
    )
}

fn prove_age_verification(
    age_proof: ZKProof,
    minimum_age: int
) -> bool {
    // Prove age >= minimum without revealing exact age
    zkp::verify_range_proof(
        age_proof,
        "age",
        min: minimum_age,
        max: None
    )
}

fn selective_audit_disclosure(
    auditor: did,
    disclosure_proof: ZKProof,
    requested_fields: [string]
) {
    require(caller_has_role(auditor, Auditor));
    
    // Verify proof allows disclosure of only requested fields
    require(zkp::verify_selective_disclosure(
        disclosure_proof,
        requested_fields,
        audit_scope: get_audit_scope(auditor)
    ));
    
    // Provide only disclosed fields
    let disclosed_data = extract_disclosed_fields(disclosure_proof, requested_fields);
    
    emit SelectiveDisclosureProvided {
        auditor: auditor,
        fields: requested_fields,
        data_hash: hash(disclosed_data),
        timestamp: now()
    };
}
```

### 20.3 ZKP Standard Library (std::zkp)

```ccl
module std::zkp {
    /// Verify membership in a group without revealing identity
    /// Note: This proves membership credentials, NOT token holdings
    fn verify_membership(proof: ZKProof, membership_group: string) -> bool;
    
    /// Verify a value is within a range without revealing the value
    fn verify_range_proof(
        proof: ZKProof, 
        field: string, 
        min: int, 
        max: Option<int>
    ) -> bool;
    
    /// Check if nullifier has been used (prevents double-spending/voting)
    fn nullifier_used(nullifier: Nullifier) -> bool;
    
    /// Mark nullifier as used
    fn mark_nullifier(nullifier: Nullifier);
    
    /// Verify selective disclosure of specific fields
    fn verify_selective_disclosure(
        proof: ZKProof,
        fields: [string],
        disclosure_scope: string
    ) -> bool;
    
    /// Generate anonymous mana charge proof (for computational actions only)
    fn prove_mana_charge(
        amount: token<Mana>,
        membership_proof: ZKProof
    ) -> ZKProof;
    
    /// Verify anonymous reputation update authority (based on membership)
    fn verify_reputation_reporter(
        membership_proof: ZKProof,
        reputation_scope: string
    ) -> bool;
    
    // Version: 1.0.0
    // Note: All ZKP functions verify membership-based authority, never token ownership
}
```

**ZKP Properties:**
- All ZKP operations are deterministic and reproducible
- Proofs are anchored in the DAG for permanent verification
- Circuit definitions are versioned and upgradeable via governance
- Trusted setup parameters are transparent and auditable

---

## 21 · Soft Law: Mediation, Restorative Justice, and Community Review

### 21.1 Mediation Workflows

Contracts may define mediation procedures for disputes before (or instead of) formal adjudication.

```ccl
state mediation_cases: [MediationCase];
state available_mediators: [did];

struct MediationCase {
    id: int,
    parties: [did],
    mediator: did,
    status: MediationStatus,
    context: string,
    created_at: timestamp,
    resolution: Option<string>,
    outcome: Option<MediationOutcome>
}

enum MediationStatus {
    Pending,
    InProgress,
    Resolved,
    Abandoned
}

enum MediationOutcome {
    AgreementReached,
    PartialResolution,
    NoAgreement,
    EscalatedToReview
}

fn propose_mediation_case(
    other_party: did, 
    context: string,
    preferred_mediator: Option<did>
) -> int {
    require(caller() != other_party);
    
    let mediator = match preferred_mediator {
        Some(med) if is_qualified_mediator(med) => med,
        _ => select_random_mediator()
    };
    
    let case_id = mediation_cases.len();
    mediation_cases.push(MediationCase {
        id: case_id,
        parties: [caller(), other_party],
        mediator: mediator,
        status: MediationStatus::Pending,
        context: context,
        created_at: now(),
        resolution: None,
        outcome: None
    });
    
    // Charge mana for mediation request
    charge_mana(caller(), token<Mana>(20), "mediation_request");
    
    emit MediationInitiated { 
        case_id: case_id,
        parties: [caller(), other_party], 
        mediator: mediator, 
        context: context, 
        timestamp: now() 
    };
    
    case_id
}

fn resolve_mediation(
    case_id: int, 
    resolution: string, 
    outcome: MediationOutcome
) {
    let mut case = mediation_cases[case_id];
    require(caller() == case.mediator);
    require(case.status == MediationStatus::InProgress);
    
    case.status = MediationStatus::Resolved;
    case.resolution = Some(resolution);
    case.outcome = Some(outcome);
    mediation_cases[case_id] = case;
    
    // Update reputation based on outcome
    match outcome {
        MediationOutcome::AgreementReached => {
            // Reward all participants
            for party in case.parties {
                update_reputation(
                    party,
                    ReputationEventType::MediationParticipation,
                    case.mediator,
                    "Successful mediation participation"
                );
            }
            update_reputation(
                case.mediator,
                ReputationEventType::MediationSuccess,
                contract_address(),
                "Successful mediation facilitation"
            );
        },
        MediationOutcome::EscalatedToReview => {
            // Initiate community review process
            initiate_community_review(case.parties, case.context);
        },
        _ => {
            // Neutral outcome - no reputation change
        }
    }
    
    emit MediationResolved { 
        case_id: case_id, 
        resolution: resolution, 
        outcome: outcome, 
        timestamp: now() 
    };
}
```

### 21.2 Community Review and Peer Panels

Create community panels for transformative or restorative justice, with peer-selected or randomized jurors.

```ccl
state review_panels: [ReviewPanel];
state panel_pool: [did]; // Eligible panel members

struct ReviewPanel {
    id: int,
    target: did,
    alleged_violation: string,
    panel_members: [did],
    status: ReviewStatus,
    created_at: timestamp,
    verdict: Option<string>,
    recommendations: Option<string>,
    votes: map<did, PanelVote>
}

enum ReviewStatus {
    Constituting,
    Active,
    Deliberating,
    Closed
}

struct PanelVote {
    verdict: PanelVerdict,
    reasoning: string,
    recommendations: [string],
    timestamp: timestamp
}

enum PanelVerdict {
    NoViolation,
    MinorViolation,
    ModerateViolation,
    SevereViolation
}

fn convene_review_panel(
    target: did, 
    alleged_violation: string,
    evidence: [string]
) -> int {
    require(caller_has_role(Member) || caller_has_role(Mediator));
    require(target != caller());
    
    // Select random panel members (excluding target and reporter)
    let panel_size = 5;
    let excluded = [target, caller()];
    let panel_members = random_select_from_pool(panel_pool, panel_size, excluded);
    
    let panel_id = review_panels.len();
    review_panels.push(ReviewPanel {
        id: panel_id,
        target: target,
        alleged_violation: alleged_violation,
        panel_members: panel_members,
        status: ReviewStatus::Constituting,
        created_at: now(),
        verdict: None,
        recommendations: None,
        votes: map::new()
    });
    
    // Charge mana for panel convening
    charge_mana(caller(), token<Mana>(100), "convene_review_panel");
    
    // Notify panel members
    for member in panel_members {
        emit PanelMemberSelected { 
            panel_id: panel_id, 
            member: member, 
            target: target,
            timestamp: now() 
        };
    }
    
    emit ReviewPanelConstituted { 
        panel_id: panel_id,
        target: target, 
        panel_members: panel_members, 
        alleged_violation: alleged_violation,
        timestamp: now() 
    };
    
    panel_id
}

fn accept_panel_duty(panel_id: int) {
    let mut panel = review_panels[panel_id];
    require(panel.panel_members.contains(caller()));
    require(panel.status == ReviewStatus::Constituting);
    
    // Track acceptance and activate panel when all members accept
    let acceptance_count = count_panel_acceptances(panel_id);
    if acceptance_count + 1 >= panel.panel_members.len() {
        panel.status = ReviewStatus::Active;
        review_panels[panel_id] = panel;
        
        emit ReviewPanelActivated { 
            panel_id: panel_id, 
            timestamp: now() 
        };
    }
    
    // Reward panel participation
    update_reputation(
        caller(),
        ReputationEventType::CommunityServiceParticipation,
        contract_address(),
        format!("Accepted panel duty for panel {}", panel_id)
    );
}

fn submit_panel_vote(
    panel_id: int, 
    verdict: PanelVerdict, 
    reasoning: string,
    recommendations: [string]
) {
    let mut panel = review_panels[panel_id];
    require(panel.panel_members.contains(caller()));
    require(panel.status == ReviewStatus::Active || panel.status == ReviewStatus::Deliberating);
    require(!panel.votes.contains_key(caller()));
    
    panel.votes[caller()] = PanelVote {
        verdict: verdict,
        reasoning: reasoning,
        recommendations: recommendations,
        timestamp: now()
    };
    
    // Check if all votes are in
    if panel.votes.len() >= panel.panel_members.len() {
        panel.status = ReviewStatus::Deliberating;
        finalize_panel_decision(panel_id);
    }
    
    review_panels[panel_id] = panel;
    
    emit PanelVoteSubmitted { 
        panel_id: panel_id, 
        voter: caller(), 
        verdict: verdict,
        timestamp: now() 
    };
}

fn finalize_panel_decision(panel_id: int) {
    let mut panel = review_panels[panel_id];
    require(panel.status == ReviewStatus::Deliberating);
    
    // Tally votes and determine consensus
    let vote_tally = tally_panel_votes(panel.votes);
    let consensus_verdict = determine_consensus_verdict(vote_tally);
    let consensus_recommendations = aggregate_recommendations(panel.votes);
    
    panel.verdict = Some(format!("{:?}", consensus_verdict));
    panel.recommendations = Some(consensus_recommendations);
    panel.status = ReviewStatus::Closed;
    review_panels[panel_id] = panel;
    
    // Apply consequences based on verdict
    apply_panel_decision(panel.target, consensus_verdict, consensus_recommendations);
    
    // Reward panel members for service
    for member in panel.panel_members {
        update_reputation(
            member,
            ReputationEventType::CommunityServiceCompletion,
            contract_address(),
            "Completed community review panel service"
        );
    }
    
    emit PanelDecisionFinalized { 
        panel_id: panel_id, 
        target: panel.target,
        verdict: consensus_verdict, 
        recommendations: consensus_recommendations, 
        timestamp: now() 
    };
}

fn apply_panel_decision(
    target: did, 
    verdict: PanelVerdict, 
    recommendations: string
) {
    match verdict {
        PanelVerdict::NoViolation => {
            // Clear any pending reputation penalties
            update_reputation(
                target,
                ReputationEventType::ExonerationByPeers,
                contract_address(),
                "Cleared of allegations by peer review"
            );
        },
        PanelVerdict::MinorViolation => {
            update_reputation(
                target,
                ReputationEventType::MinorViolation,
                contract_address(),
                "Minor violation found by peer review"
            );
        },
        PanelVerdict::ModerateViolation => {
            update_reputation(
                target,
                ReputationEventType::ModerateViolation,
                contract_address(),
                "Moderate violation found by peer review"
            );
            // Suggest restorative justice process
            suggest_restorative_process(target, recommendations);
        },
        PanelVerdict::SevereViolation => {
            update_reputation(
                target,
                ReputationEventType::SevereViolation,
                contract_address(),
                "Severe violation found by peer review"
            );
            // Mandatory restorative justice or potential suspension
            initiate_mandatory_restorative_process(target, recommendations);
        }
    }
}
```

### 21.3 Restorative Actions and Community Healing

Contracts can encode restorative justice—actions for harm repair, reconciliation, and accountability.

```ccl
state restorative_sessions: [RestorativeSession];
state community_service_opportunities: [ServiceOpportunity];

struct RestorativeSession {
    id: int,
    offender: did,
    harmed: did,
    facilitator: did,
    status: SessionStatus,
    scheduled_date: timestamp,
    outcome: Option<RestorativeOutcome>,
    agreement: Option<string>,
    follow_up_required: bool
}

enum SessionStatus {
    Proposed,
    Accepted,
    Scheduled,
    InProgress,
    Completed,
    Cancelled
}

enum RestorativeOutcome {
    FullReconciliation,
    PartialResolution,
    AgreementReached,
    ProcessIncomplete,
    EscalatedToAuthority
}

struct ServiceOpportunity {
    id: int,
    title: string,
    description: string,
    hours_required: int,
    skills_needed: [string],
    organizer: did,
    participants: [did],
    status: ServiceStatus
}

enum ServiceStatus {
    Open,
    InProgress,
    Completed,
    Cancelled
}

fn propose_restorative_session(
    harmed_party: did, 
    facilitator: did,
    proposed_date: timestamp
) -> int {
    let offender = caller();
    require(offender != harmed_party);
    require(is_qualified_facilitator(facilitator));
    
    let session_id = restorative_sessions.len();
    restorative_sessions.push(RestorativeSession {
        id: session_id,
        offender: offender,
        harmed: harmed_party,
        facilitator: facilitator,
        status: SessionStatus::Proposed,
        scheduled_date: proposed_date,
        outcome: None,
        agreement: None,
        follow_up_required: false
    });
    
    emit RestorativeSessionProposed { 
        session_id: session_id,
        offender: offender, 
        harmed: harmed_party, 
        facilitator: facilitator,
        proposed_date: proposed_date,
        timestamp: now() 
    };
    
    session_id
}

fn accept_restorative_session(session_id: int) {
    let mut session = restorative_sessions[session_id];
    require(caller() == session.harmed || caller() == session.facilitator);
    require(session.status == SessionStatus::Proposed);
    
    session.status = SessionStatus::Accepted;
    restorative_sessions[session_id] = session;
    
    emit RestorativeSessionAccepted { 
        session_id: session_id,
        accepted_by: caller(),
        timestamp: now() 
    };
}

fn schedule_restorative_session(
    session_id: int,
    final_date: timestamp
) {
    let mut session = restorative_sessions[session_id];
    require(caller() == session.facilitator);
    require(session.status == SessionStatus::Accepted);
    
    session.status = SessionStatus::Scheduled;
    session.scheduled_date = final_date;
    restorative_sessions[session_id] = session;
    
    emit RestorativeSessionScheduled { 
        session_id: session_id,
        date: final_date,
        timestamp: now() 
    };
}

fn record_restorative_outcome(
    session_id: int, 
    outcome: RestorativeOutcome,
    agreement: string,
    follow_up_required: bool
) {
    let mut session = restorative_sessions[session_id];
    require(caller() == session.facilitator);
    require(session.status == SessionStatus::InProgress || session.status == SessionStatus::Scheduled);
    
    session.status = SessionStatus::Completed;
    session.outcome = Some(outcome);
    session.agreement = Some(agreement);
    session.follow_up_required = follow_up_required;
    restorative_sessions[session_id] = session;
    
    // Update reputation based on outcome
    match outcome {
        RestorativeOutcome::FullReconciliation => {
            update_reputation(
                session.offender,
                ReputationEventType::RestorativeJusticeCompleted,
                session.facilitator,
                "Completed restorative justice process with full reconciliation"
            );
            update_reputation(
                session.harmed,
                ReputationEventType::CommunityParticipation,
                session.facilitator,
                "Participated in restorative justice process"
            );
        },
        RestorativeOutcome::AgreementReached => {
            update_reputation(
                session.offender,
                ReputationEventType::RestorativeJusticePartial,
                session.facilitator,
                "Reached agreement in restorative justice process"
            );
        },
        RestorativeOutcome::ProcessIncomplete => {
            update_reputation(
                session.offender,
                ReputationEventType::RestorativeJusticeRefused,
                session.facilitator,
                "Did not complete restorative justice process"
            );
        },
        _ => {
            // Other outcomes may require follow-up
        }
    }
    
    emit RestorativeSessionCompleted { 
        session_id: session_id, 
        outcome: outcome,
        agreement: agreement,
        timestamp: now() 
    };
}

fn create_community_service_opportunity(
    title: string,
    description: string,
    hours_required: int,
    skills_needed: [string]
) -> int {
    let opportunity_id = community_service_opportunities.len();
    community_service_opportunities.push(ServiceOpportunity {
        id: opportunity_id,
        title: title,
        description: description,
        hours_required: hours_required,
        skills_needed: skills_needed,
        organizer: caller(),
        participants: [],
        status: ServiceStatus::Open
    });
    
    emit CommunityServiceOpportunityCreated { 
        opportunity_id: opportunity_id,
        title: title,
        organizer: caller(),
        timestamp: now() 
    };
    
    opportunity_id
}

fn volunteer_for_community_service(opportunity_id: int) {
    let mut opportunity = community_service_opportunities[opportunity_id];
    require(opportunity.status == ServiceStatus::Open);
    require(!opportunity.participants.contains(caller()));
    
    opportunity.participants.push(caller());
    community_service_opportunities[opportunity_id] = opportunity;
    
    update_reputation(
        caller(),
        ReputationEventType::CommunityServiceVolunteering,
        opportunity.organizer,
        format!("Volunteered for community service: {}", opportunity.title)
    );
    
    emit CommunityServiceVolunteer { 
        opportunity_id: opportunity_id,
        volunteer: caller(),
        timestamp: now() 
    };
}

fn complete_community_service(
    opportunity_id: int,
    participant: did,
    hours_completed: int
) {
    let opportunity = community_service_opportunities[opportunity_id];
    require(caller() == opportunity.organizer);
    require(opportunity.participants.contains(participant));
    require(hours_completed >= opportunity.hours_required);
    
    // Award reputation for service completion
    let reputation_bonus = match hours_completed {
        h if h >= opportunity.hours_required * 2 => 5.0, // Exceeded expectations
        h if h >= opportunity.hours_required => 3.0,     // Met requirements
        _ => 1.0                                         // Partial completion
    };
    
    update_reputation(
        participant,
        ReputationEventType::CommunityServiceCompletion,
        caller(),
        format!("Completed {} hours of community service", hours_completed)
    );
    
    emit CommunityServiceCompleted { 
        opportunity_id: opportunity_id,
        participant: participant,
        hours_completed: hours_completed,
        timestamp: now() 
    };
}
```

---

## 22 · Anonymous Restorative Governance Example

Combining privacy, ZKP, and soft law for comprehensive community justice:

```ccl
proposal AnonymousRestorativeVoting {
    description: "Anonymous community vote on proposed restorative plan";
    eligible: Member;
    voting_method: ZKPBallot;
    quorum: 50%;
    threshold: majority;
    
    execution: {
        let total_votes = count_anonymous_votes();
        let approve_votes = count_votes_for_choice(VoteChoice::Approve);
        
        if approve_votes > total_votes / 2 {
            implement_restorative_plan();
            emit RestorativePlanApproved { 
                total_votes: total_votes,
                approve_votes: approve_votes,
                timestamp: now() 
            };
        } else {
            escalate_to_review_panel();
            emit RestorativePlanRejected { 
                total_votes: total_votes,
                approve_votes: approve_votes,
                timestamp: now() 
            };
        }
    };
}

fn anonymous_restorative_vote(
    proposal_id: int,
    vote_choice: VoteChoice,
    membership_proof: ZKProof,
    mana_proof: ZKProof,
    nullifier: Nullifier
) {
    // Verify voter eligibility without revealing identity
    require(zkp::verify_membership(membership_proof, "eligible_voters"));
    
    // Verify voter has sufficient mana without revealing balance
    require(zkp::verify_range_proof(mana_proof, "mana_balance", min: 2, max: None));
    
    // Prevent double voting
    require(!zkp::nullifier_used(nullifier));
    zkp::mark_nullifier(nullifier);
    
    // Charge mana anonymously
    anonymous_mana_charge(nullifier, token<Mana>(2));
    
    // Record anonymous vote
    proposal_votes[proposal_id].push(AnonymousVote {
        choice: vote_choice,
        nullifier: nullifier,
        timestamp: now()
    });
    
    emit AnonymousRestorativeVote { 
        proposal_id: proposal_id,
        nullifier: nullifier,
        timestamp: now() 
    };
}
```

---

## 23 · Privacy and Soft Law Integration Summary

### 23.1 Complete Cooperative Justice System

The integration of privacy, ZKP, and soft law creates a comprehensive framework for community governance:

**Identity Layer (DID)**
- Cryptographically verified identity
- Can be anonymous via ZKP when needed
- Scoped to communities and federations

**Capacity Layer (Mana)**
- Merit-based participation credits
- Regenerates based on reputation
- Can be used anonymously via ZKP proofs

**Trust Layer (Reputation)**
- Community-validated trust scores
- Updated through transparent processes
- Can be reported anonymously to prevent retaliation

**Privacy Layer (ZKP)**
- Anonymous but verifiable participation
- Selective disclosure of personal information
- GDPR and consent compliance

**Justice Layer (Soft Law)**
- Community-driven conflict resolution
- Restorative justice and healing processes
- Graduated consequences with rehabilitation focus

### 23.2 Emergent Properties

This integrated system enables:

**Anonymous Accountability**: Bad actors face consequences without exposing reporters to retaliation

**Privacy-Preserving Merit**: Contributions are recognized without compromising personal privacy

**Community Healing**: Focus on restoration and rehabilitation rather than punishment

**Graduated Justice**: Soft law → mediation → community review → formal consequences

**Transformative Conflict Resolution**: Address root causes and repair relationships

**Democratic Privacy**: Transparent governance with individual privacy protection

### 23.3 Technical Implementation

All privacy and soft law features are:
- **Deterministic**: Same inputs produce same outputs
- **Verifiable**: All operations generate cryptographic proofs
- **Auditable**: Actions are logged in the DAG with proper privacy protections
- **Upgradeable**: Governance can evolve privacy and justice mechanisms
- **Interoperable**: Works across federations and scales

---

## 24 · Conclusion

The Cooperative Contract Language (CCL) provides a complete framework for encoding legal contracts, governance systems, and economic rules as deterministic, verifiable code. By treating code as law, CCL enables communities and cooperatives to operate with transparent, enforceable agreements that evolve through democratic processes.

CCL's federation system allows for seamless scaling from local cooperatives to global governance networks, while maintaining the autonomy and self-determination of each participating community.

This specification serves as the foundation for implementing CCL compilers, runtimes, and tooling that will support the cooperative digital economy of the InterCooperative Network.

---

**Version**: 0.1  
**Date**: 2024-01-15  
**Status**: Draft Specification  
**Next Review**: 2024-02-15 