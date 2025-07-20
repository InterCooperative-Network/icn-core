# CCL Standard Library - Economics and Identity Functions

This document describes the critical economics and identity functions implemented in the CCL (Cooperative Contract Language) standard library. These functions enable CCL contracts to power real cooperative digital infrastructure with token economies, reputation systems, and decentralized identity management.

## Economics Functions

### Token System Operations

#### `create_token_class(class_id: String, name: String, symbol: String, issuer: Did) -> Bool`
Creates a new token class with specified properties. The issuer DID controls minting and burning operations.

**Example:**
```ccl
let success = create_token_class(
    "work_hours",
    "Cooperative Work Hours", 
    "CWH",
    @admin_did
);
```

#### `mint_tokens(class_id: String, recipient: Did, amount: Integer) -> Bool`
Mints new tokens of the specified class to the recipient account. Requires issuer authorization.

**Example:**
```ccl
let minted = mint_tokens("work_hours", @worker_did, 8);
```

#### `transfer_tokens(class_id: String, from: Did, to: Did, amount: Integer) -> Bool`
Transfers tokens between accounts with proper authorization and balance validation.

**Example:**
```ccl
let transferred = transfer_tokens("work_hours", @alice_did, @bob_did, 5);
```

#### `burn_tokens(class_id: String, from: Did, amount: Integer) -> Bool`
Destroys tokens from circulation, reducing the total supply.

**Example:**
```ccl
let burned = burn_tokens("work_hours", @member_did, 2);
```

#### `get_token_balance(class_id: String, account: Did) -> Integer`
Returns the current token balance for the specified account and token class.

**Example:**
```ccl
let balance = get_token_balance("work_hours", @member_did);
```

### Reputation-Linked Functions

#### `price_by_reputation(base_price: Integer, reputation_score: Integer) -> Integer`
Calculates adjusted pricing based on reputation. Higher reputation results in lower prices, enabling cooperative discount structures.

**Formula:** `adjusted_price = base_price * 100 / (100 + reputation)`

**Example:**
```ccl
let discounted_price = price_by_reputation(100, 50); // Returns ~67 (33% discount)
```

#### `credit_by_reputation(account: Did, base_amount: Integer) -> Bool`
Credits mana to an account based on their reputation multiplier, enabling fair resource regeneration.

**Example:**
```ccl
let credited = credit_by_reputation(@member_did, 10); // Credits 10 * reputation_score
```

#### `mint_tokens_with_reputation(class_id: String, recipient: Did, amount: Integer, issuer: Did) -> Bool`
Mints tokens with mana costs adjusted by the issuer's reputation, incentivizing high-reputation token creation.

**Example:**
```ccl
let minted = mint_tokens_with_reputation("work_hours", @worker_did, 5, @supervisor_did);
```

### Time Banking Functions

#### `record_time_work(worker: Did, work_description: String, hours_worked: Integer, verifier: Did) -> String`
Records time-based work contributions for time banking systems. Returns a unique time record ID.

**Example:**
```ccl
let record_id = record_time_work(
    @worker_did,
    "Community garden maintenance",
    4,
    @supervisor_did
);
```

#### `mint_time_tokens(time_record_id: String, worker: Did, hours: Integer) -> Bool`
Mints time-based tokens for verified work hours, enabling hour-for-hour exchange systems.

**Example:**
```ccl
let time_tokens_minted = mint_time_tokens(record_id, @worker_did, 4);
```

### Mutual Credit Functions

#### `create_credit_line(creditor: Did, debtor: Did, credit_limit: Integer, interest_rate_bps: Integer) -> String`
Establishes a mutual credit relationship between community members. Returns a credit line ID.

**Example:**
```ccl
let credit_line_id = create_credit_line(
    @member_a_did,
    @member_b_did, 
    1000, // credit limit
    200   // 2% interest rate (200 basis points)
);
```

#### `extend_mutual_credit(credit_line_id: String, amount: Integer, purpose: String) -> Bool`
Extends credit within an established mutual credit line for a specific purpose.

**Example:**
```ccl
let credit_extended = extend_mutual_credit(
    credit_line_id,
    250,
    "Equipment purchase for shared workshop"
);
```

### Marketplace Functions

#### `create_marketplace_offer(seller: Did, item_type: String, quantity: Integer, price_per_unit: Integer, payment_token_class: String) -> String`
Creates a marketplace offer for goods or services. Returns an offer ID.

**Example:**
```ccl
let offer_id = create_marketplace_offer(
    @farmer_did,
    "organic_vegetables",
    10,
    5,
    "local_credits"
);
```

#### `execute_marketplace_transaction(offer_id: String, bid_id: String, executor: Did) -> String`
Executes a marketplace transaction between buyer and seller. Returns a transaction ID.

**Example:**
```ccl
let transaction_id = execute_marketplace_transaction(
    offer_id,
    bid_id,
    @marketplace_operator_did
);
```

## Identity Functions

### DID Management

#### `create_did(method: String, identifier: String) -> Did`
Generates a new decentralized identifier using the specified method ("key", "web", "peer").

**Example:**
```ccl
let new_did = create_did("key", ""); // Creates did:key:...
let web_did = create_did("web", "coop.example.com"); // Creates did:web:coop.example.com
```

#### `resolve_did(did: Did) -> String`
Resolves a DID to its document containing public keys and service endpoints. Returns DID document as JSON.

**Example:**
```ccl
let did_document = resolve_did(@member_did);
```

#### `update_did_document(did: Did, new_document: String, signature: String) -> Bool`
Updates a DID document with new keys or service endpoints, requiring controller signature.

**Example:**
```ccl
let updated = update_did_document(@member_did, new_doc_json, controller_signature);
```

#### `verify_did_signature(signer_did: Did, message: String, signature: String) -> Bool`
Verifies that a signature was created by the DID controller, enabling trustless authentication.

**Example:**
```ccl
let is_authentic = verify_did_signature(@signer_did, "Hello ICN", signature_hex);
```

### Credential Management

#### `issue_credential(issuer: Did, holder: Did, credential_type: String, claims: String, expiration: Integer) -> String`
Issues a verifiable credential with specified claims. Returns credential as JSON.

**Example:**
```ccl
let credential = issue_credential(
    @coop_admin_did,
    @member_did,
    "membership",
    "{\"level\": \"active\", \"joined\": \"2024-01-01\"}",
    1735689600 // expiration timestamp
);
```

#### `verify_credential(credential: String, expected_issuer: Did) -> Bool`
Verifies the authenticity and validity of a credential, including signature and expiration.

**Example:**
```ccl
let is_valid = verify_credential(credential_json, @trusted_issuer_did);
```

#### `revoke_credential(issuer: Did, credential_id: String, reason: String) -> Bool`
Revokes a previously issued credential, adding it to the revocation registry.

**Example:**
```ccl
let revoked = revoke_credential(
    @issuer_did,
    "cred_12345",
    "Membership expired"
);
```

### Cooperative-Specific Functions

#### `create_cooperative_membership(member: Did, cooperative_id: String, membership_type: String, level: Integer) -> String`
Creates a cooperative membership credential with specified type and authorization level.

**Example:**
```ccl
let membership = create_cooperative_membership(
    @new_member_did,
    "food_coop_001",
    "active_member",
    3
);
```

#### `verify_cooperative_membership(member: Did, cooperative_id: String, required_level: Integer) -> Bool`
Verifies cooperative membership and checks if the member meets the required authorization level.

**Example:**
```ccl
let can_vote = verify_cooperative_membership(
    @member_did,
    "food_coop_001",
    2 // minimum level for voting rights
);
```

## Integration Examples

### Complete Cooperative Token Economy

```ccl
contract cooperative_economy {
    init {
        // Set up token classes
        create_token_class("work_hours", "Work Hours", "WH", @admin_did);
        create_token_class("local_currency", "Local Currency", "LC", @admin_did);
    }
    
    fn process_work_session(worker: Did, hours: Integer) -> Bool {
        // Verify membership
        let is_member = verify_cooperative_membership(worker, "coop_001", 1);
        if !is_member { return false; }
        
        // Record work
        let record_id = record_time_work(worker, "General labor", hours, @supervisor_did);
        
        // Mint tokens with reputation bonus
        let tokens_minted = mint_tokens_with_reputation("work_hours", worker, hours, @supervisor_did);
        
        // Credit mana based on reputation
        let mana_credited = credit_by_reputation(worker, hours);
        
        return tokens_minted && mana_credited;
    }
    
    fn cooperative_trade(seller: Did, buyer: Did, item: String, price: Integer) -> String {
        // Create marketplace offer
        let offer_id = create_marketplace_offer(seller, item, 1, price, "local_currency");
        
        // Execute transaction (simplified - normally would include bidding)
        let tx_id = execute_marketplace_transaction(offer_id, "bid_123", @marketplace_did);
        
        return tx_id;
    }
}
```

This implementation provides the foundational functions needed for real-world cooperative digital infrastructure, enabling communities to create fair, transparent, and extractive-free economic systems using CCL contracts.