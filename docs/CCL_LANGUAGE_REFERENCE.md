# Cooperative Contract Language (CCL) Reference

This reference summarizes all syntax currently supported by the CCL compiler.
It is intended for cooperative developers writing governance policies and
economic logic for ICN nodes.

## Basic Concepts

CCL source files consist of function definitions, optional struct definitions,
and policy rules. Whitespace and `//` comments are ignored.

```ccl
// simple entry point
fn run() -> Integer { return 0; }
```

### Types

Built in primitive types:

- `Integer` – 64‑bit signed integer
- `Mana` – alias of `Integer` for economic values
- `Bool`  – boolean (`true` / `false`)
- `String` – UTF‑8 text stored in memory
- `Array<T>` – dynamic arrays
- `Option` and `Result` – for nullable values and error handling
- `Did` – decentralized identifier
- `Proposal` and `Vote` – governance primitives

User defined structs can bundle fields:

```ccl
struct Member { id: Did, name: String }
```

### Functions

Functions declare typed parameters and return a type:

```ccl
fn add(a: Integer, b: Integer) -> Integer {
    return a + b;
}
```

### Statements

Supported statements inside blocks:

- variable binding with `let`
- expression statements
- `return` expressions
- `if` / `else` conditional blocks
- `while` loops
- `for` loops

For loops iterate over arrays or other iterable values:

```ccl
for item in numbers {
    log_value(item);
}
```

### Expressions

Expressions support numeric and boolean operators, string concatenation, array
literals, and function calls. Arrays use helper functions such as `array_push`
and `array_len` for mutation.

Option and Result values use the variants `Some`, `None`, `Ok` and `Err`.
Pattern matching inspects these variants:

```ccl
fn divide(a: Integer, b: Integer) -> Result<Integer> {
    if b == 0 { return Err(1); }
    return Ok(a / b);
}

let result = divide(10, 2);
match result {
    Ok(v) => log_success(v),
    Err(e) => log_error(e),
}
```

`match` expressions are compiled into WebAssembly using nested blocks. Each arm
compares the matched value against its pattern and jumps out of the block when a
match succeeds. The current compiler supports literal patterns, variable
bindings and enum variant patterns.

### Policy Rules

Policy oriented contracts can declare rules which evaluate an expression and then
perform an action:

```ccl
rule charge_high when cost > 100 then charge cost
rule allow_basic when cost <= 100 then allow
```

Actions are `allow`, `deny`, or `charge <expr>`.

### Token Economics Functions

CCL provides comprehensive token economics capabilities for cooperative economies:

#### Basic Token Operations
```ccl
// Create a new token class
let token_class = create_token_class("COOP", "Cooperative Token", "COOP", issuer_did);

// Mint tokens to accounts
let minted = mint_tokens("COOP", member_did, 1000);

// Transfer tokens between accounts
let transferred = transfer_tokens("COOP", from_did, to_did, 500);

// Burn tokens from circulation
let burned = burn_tokens("COOP", account_did, 100);

// Check token balance
let balance = get_token_balance("COOP", account_did);
```

#### Reputation-Integrated Economics
```ccl
// Calculate reputation-adjusted pricing
let adjusted_price = price_by_reputation(base_price, member_reputation);

// Credit mana based on reputation
let credited = credit_by_reputation(member_did, base_amount);

// Mint tokens with reputation cost adjustment
let minted = mint_tokens_with_reputation("COOP", recipient_did, amount, issuer_did);
```

#### Time Banking Operations
```ccl
// Record verified work hours
let time_record = record_time_work(worker_did, "Garden maintenance", 8, verifier_did);

// Mint time-based tokens
let time_tokens = mint_time_tokens(time_record, worker_did, 8);
```

#### Mutual Credit System
```ccl
// Establish credit line between members
let credit_line = create_credit_line(creditor_did, debtor_did, 5000, 300); // 3% interest

// Extend credit within established line
let credit_extended = extend_mutual_credit(credit_line, 1000, "Equipment purchase");
```

#### Marketplace Operations
```ccl
// Create marketplace offer
let offer = create_marketplace_offer(
    seller_did, 
    "Organic vegetables", 
    50, 
    10, 
    "COOP"
);

// Execute marketplace transaction
let transaction = execute_marketplace_transaction(offer_id, bid_id, executor_did);
```

#### Scoped Token Operations
```ccl
// Create geographically scoped tokens
let scoped_token = create_scoped_token(
    "LOCAL", 
    "Local Currency", 
    "LOC", 
    issuer_did, 
    "geographic", 
    "bioregion_cascade"
);

// Transfer with scope validation
let scoped_transfer = transfer_scoped("LOCAL", from_did, to_did, amount, "bioregion_cascade");

// Verify transfer constraints
let valid = verify_token_constraints("LOCAL", actor_did, "transfer", target_scope);
```

### Identity and Federation Functions

CCL provides comprehensive decentralized identity and federation management:

#### DID Operations
```ccl
// Create new decentralized identifier
let new_did = create_did("key", ""); // or create_did("web", "example.com")

// Resolve DID to document
let did_document = resolve_did(member_did);

// Update DID document
let updated = update_did_document(member_did, new_document_json, signature);

// Verify DID signature
let signature_valid = verify_did_signature(signer_did, message, signature);
```

#### Credential Management
```ccl
// Issue verifiable credential
let credential = issue_credential(
    issuer_did,
    holder_did,
    "MembershipCredential",
    "{\"level\": \"full\", \"joined\": \"2025-01-01\"}",
    1735689600 // expiration timestamp
);

// Verify credential authenticity
let credential_valid = verify_credential(credential_json, expected_issuer);

// Revoke issued credential
let revoked = revoke_credential(issuer_did, credential_id, "Member left cooperative");
```

#### Cooperative Membership
```ccl
// Create cooperative membership credential
let membership = create_cooperative_membership(
    member_did,
    "coop_cascade_commons",
    "full_member",
    5 // membership level
);

// Verify membership and authorization
let authorized = verify_cooperative_membership(member_did, "coop_cascade_commons", 3);
```

#### Federation Operations
```ccl
// Discover available federations
let federations = discover_federations("bioregional", 10);

// Join federation
let joined = join_federation(member_did, "bioregion_cascade", application_details);

// Leave federation
let left = leave_federation(member_did, "bioregion_cascade", "Relocating");

// Verify cross-federation credentials
let cross_valid = verify_cross_federation(
    verifier_did,
    "source_federation",
    "target_federation",
    "MembershipCredential"
);
```

#### Key Management
```ccl
// Rotate DID keys
let rotated = rotate_keys(member_did, new_public_key, old_key_signature);

// Create secure key backup
let backup_id = backup_keys(member_did, "multisig", backup_parameters);

// Recover keys from backup
let recovered = recover_keys(member_did, backup_id, recovery_proof);
```

#### Federation Coordination
```ccl
// Get federation metadata
let federation_info = get_federation_metadata("bioregion_cascade");

// Verify federation membership
let is_member = verify_federation_membership(member_did, "bioregion_cascade");

// Coordinate cross-federation action
let coordination_id = coordinate_cross_federation_action(
    coordinator_did,
    ["federation_a", "federation_b"],
    "resource_sharing",
    action_parameters
);
```

### DAG Storage Functions

CCL provides comprehensive functions for interacting with the content-addressed DAG storage system:

#### Basic DAG Operations
```ccl
// Store data and get its Content Identifier (CID)
let cid = dag_put("Hello, DAG World!");

// Retrieve data using its CID
let data = dag_get(cid);

// Calculate CID without storing
let computed_cid = calculate_cid("test data");

// Pin content to prevent garbage collection
let pinned = dag_pin(cid);

// Unpin content to allow garbage collection
let unpinned = dag_unpin(cid);
```

#### Contract State Persistence
```ccl
// Save contract state with versioning
let contract_id = "my_contract_v1";
let state_data = "{'balance': 1000, 'members': 42}";
let version = 1;
let state_cid = save_contract_state(contract_id, state_data, version);

// Load contract state by ID and version
let loaded_state = load_contract_state(contract_id, version);

// Create new contract version
let new_code_cid = dag_put("fn updated_logic() -> Integer { return 42; }");
let new_version = version_contract(contract_id, new_code_cid, "Performance improvements");
```

#### Advanced DAG Operations
```ccl
// Create links between DAG objects
let metadata_cid = dag_put("{'author': 'ICN', 'version': '1.0'}");
let linked_cid = dag_link(code_cid, metadata_cid, "metadata");

// Resolve paths within DAG structures
let resolved_data = dag_resolve_path(linked_cid, "metadata");

// List all links in a DAG object
let link_names = dag_list_links(linked_cid);
```

### Credential Verification

Use `require_proof(expr)` to validate a zero-knowledge credential proof. The
argument must be a JSON string containing a `ZkCredentialProof`. The expression
returns `true` when the proof verifies.

```ccl
let ok = require_proof(claim_json);
```

### Imports

External files can be imported with an alias:

```ccl
import "./common.ccl" as common;
```

### Entry Point

Contracts executed as mesh jobs expose a `run` function. Additional helper
functions may be defined as needed.

See `icn-ccl/examples/` for real‑world contract templates demonstrating the
language. Examples `age_proof.ccl` and `reputation_proof.ccl` show credential
verification.
