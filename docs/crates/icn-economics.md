# ICN Economics (`icn-economics`) - Cooperative Economic Engine

> **A comprehensive economic system enabling fair resource allocation, diverse value exchange, and cooperative economic models within federated networks**

## Overview

The `icn-economics` crate implements ICN's sophisticated economic infrastructure, providing a complete toolkit for managing regenerating resources (mana), diverse token systems, cooperative marketplaces, mutual credit networks, time banking, and mutual aid systems. It creates the foundation for a cooperative digital economy that serves community needs rather than extracting value.

**Key Principle**: Economic systems should reward cooperation, ensure fair access to resources, and enable diverse forms of value exchange that strengthen communities.

## Core Economic Systems

### 💧 Mana System - Regenerating Resource Credits

The mana system provides fair, regenerating access to computational and network resources:

#### ManaLedger Trait - Abstract Interface
```rust
/// Core interface for mana balance management
pub trait ManaLedger: Send + Sync {
    /// Retrieve current mana balance for a DID
    fn get_balance(&self, did: &Did) -> u64;
    
    /// Set absolute balance (used for initialization)
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    
    /// Spend mana from account (with insufficient balance checks)
    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    
    /// Credit mana to account (regeneration and rewards)
    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    
    /// Credit all accounts with additional mana (global regeneration)
    fn credit_all(&self, amount: u64) -> Result<(), CommonError>;
    
    /// List all account DIDs (for bulk operations)
    fn all_accounts(&self) -> Vec<Did>;
}
```

#### Storage Backend Options
```rust
// File-based persistence for simple deployments
let mana_ledger = FileManaLedger::new(PathBuf::from("./mana_ledger.json"))?;

// Sled embedded database (default)
let mana_ledger = SledManaLedger::new(PathBuf::from("./mana_db"))?;

// SQLite for structured queries
let mana_ledger = SqliteManaLedger::new(PathBuf::from("./mana.db"))?;

// RocksDB for high performance
let mana_ledger = RocksdbManaLedger::new(PathBuf::from("./mana_rocksdb"))?;
```

#### Policy Enforcement
```rust
/// Resource policy enforcer with spending limits
pub struct ResourcePolicyEnforcer<L: ManaLedger> {
    pub const MAX_SPEND_LIMIT: u64 = 1000; // Maximum single transaction
}

impl<L: ManaLedger> ResourcePolicyEnforcer<L> {
    /// Spend mana with policy validation
    pub fn spend_mana(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        // Validate non-zero amount
        if amount == 0 {
            return Err(CommonError::PolicyDenied("Amount must be greater than zero"));
        }
        
        // Check available balance
        let available = self.adapter.get_balance(did);
        if available < amount {
            return Err(CommonError::PolicyDenied(format!("Insufficient mana for DID {did}")));
        }
        
        // Enforce spending limits
        if amount > Self::MAX_SPEND_LIMIT {
            return Err(CommonError::PolicyDenied(format!("Spend amount exceeds limit")));
        }
        
        self.adapter.spend_mana(did, amount)
    }
}
```

### 🪙 Resource Token System - Diverse Value Representation

The resource ledger supports multiple token types for different economic models:

#### Token Types
```rust
/// Comprehensive token type system
pub enum TokenType {
    Fungible,        // Traditional currencies and credits
    NonFungible,     // Unique certificates and assets
    SemiFungible,    // Hybrid tokens with unique properties
    TimeBanking,     // Labor hour tokens with equal value
    MutualCredit,    // Community-issued credit tokens
    LocalCurrency,   // Geographic/community restricted currency
    BulkPurchasing,  // Collective buying power tokens
}
```

#### Token Class Definition
```rust
/// Complete token class specification
pub struct TokenClass {
    pub name: String,                           // Human-readable name
    pub description: String,                    // Detailed description
    pub symbol: String,                        // Trading symbol (e.g., "TIME")
    pub decimals: u8,                          // Decimal precision
    pub token_type: TokenType,                 // Type of token
    pub transferability: TransferabilityRule,   // Transfer restrictions
    pub scoping_rules: ScopingRules,           // Geographic/community limits
    pub issuer: Did,                           // Token issuer DID
    pub created_at: u64,                       // Creation timestamp
    pub metadata: HashMap<String, String>,      // Additional properties
}
```

#### Transfer Rules
```rust
/// Transfer restriction policies
pub enum TransferabilityRule {
    FreelyTransferable,                        // No restrictions
    RestrictedTransfer {                       // Limited recipients
        authorized_recipients: HashSet<Did>,
    },
    NonTransferable,                           // Cannot be transferred
    IssuerOnly,                                // Only issuer can receive
}
```

#### ResourceLedger Implementation
```rust
/// Core resource token management interface
pub trait ResourceLedger: Send + Sync {
    /// Create new token class
    fn create_class(&self, class_id: &TokenClassId, class: TokenClass) -> Result<(), CommonError>;
    
    /// Increase account balance (token creation)
    fn mint(&self, class_id: &TokenClassId, owner: &Did, amount: u64) -> Result<(), CommonError>;
    
    /// Decrease account balance (token destruction)
    fn burn(&self, class_id: &TokenClassId, owner: &Did, amount: u64) -> Result<(), CommonError>;
    
    /// Transfer tokens between accounts
    fn transfer(&self, class_id: &TokenClassId, from: &Did, to: &Did, amount: u64) -> Result<(), CommonError>;
    
    /// Get account balance for specific token class
    fn get_balance(&self, class_id: &TokenClassId, owner: &Did) -> u64;
    
    /// Validate transfer permissions
    fn can_transfer(&self, class_id: &TokenClassId, from: &Did, to: &Did, amount: u64) -> Result<bool, CommonError>;
    
    /// Get transaction history
    fn get_transfer_history(&self, class_id: &TokenClassId, did: &Did) -> Vec<TransferRecord>;
}
```

## Specialized Economic Models

### ⏰ Time Banking - Equal Labor Value Exchange

Time banking enables communities to exchange labor hours on an equal basis, recognizing all work as equally valuable:

#### Time Record Management
```rust
/// Work performed in time banking system
pub struct TimeRecord {
    pub record_id: String,              // Unique identifier
    pub worker: Did,                    // Person who performed work
    pub beneficiary: Did,              // Person who received benefit
    pub work_type: String,             // Type of work performed
    pub description: String,           // Detailed description
    pub hours: f64,                    // Hours worked (fractional)
    pub skill_level: String,           // Required skill level
    pub performed_at: u64,             // When work was done
    pub status: TimeRecordStatus,      // Current status
}

/// Time record status lifecycle
pub enum TimeRecordStatus {
    Recorded,   // Work performed and recorded
    Verified,   // Verified by beneficiary
    Disputed,   // Under dispute
    Cancelled,  // Record cancelled
}
```

#### Time Banking Operations
```rust
/// Record work and mint time tokens
pub fn record_and_mint_time_tokens<L: ResourceLedger, T: TimeBankingStore>(
    resource_ledger: &L,
    time_store: &T,
    time_token_class: &TokenClassId,
    worker: &Did,
    beneficiary: &Did,
    work_type: String,
    description: String,
    hours: f64,
    skill_level: String,
) -> Result<String, CommonError> {
    // Validate token class is for time banking
    let token_class = resource_ledger.get_class(time_token_class)?;
    if token_class.token_type != TokenType::TimeBanking {
        return Err(CommonError::InvalidInputError("Token class not for time banking"));
    }
    
    // Create time record
    let record_id = format!("time_{}_{}", worker, SystemTimeProvider.unix_seconds());
    let time_record = TimeRecord {
        record_id: record_id.clone(),
        worker: worker.clone(),
        beneficiary: beneficiary.clone(),
        work_type,
        description,
        hours,
        skill_level,
        performed_at: SystemTimeProvider.unix_seconds(),
        recorded_at: SystemTimeProvider.unix_seconds(),
        status: TimeRecordStatus::Recorded,
        metadata: HashMap::new(),
    };
    
    // Record the time
    time_store.record_time(time_record)?;
    
    // Convert hours to token units (2 decimal places)
    let token_amount = (hours * 100.0) as u64;
    
    // Mint time tokens to worker
    resource_ledger.mint(time_token_class, worker, token_amount)?;
    
    Ok(record_id)
}
```

### 🤝 Mutual Credit - Community-Issued Money

Mutual credit allows communities to create money by extending credit to trusted members:

#### Credit Line Management
```rust
/// Credit line extended to community member
pub struct CreditLine {
    pub credit_id: String,                  // Unique identifier
    pub account: Did,                       // Account receiving credit
    pub token_class: TokenClassId,          // Token class for credit
    pub credit_limit: u64,                  // Maximum credit allowed
    pub credit_used: u64,                   // Currently used credit
    pub interest_rate: u16,                 // Interest rate (basis points)
    pub created_at: u64,                    // Creation timestamp
    pub expires_at: Option<u64>,            // Optional expiration
    pub status: CreditLineStatus,           // Current status
    pub credit_score: CreditScore,          // Credit scoring factors
}

/// Credit line status
pub enum CreditLineStatus {
    Active,     // Available for use
    Suspended,  // Temporarily unavailable
    Closed,     // Permanently closed
    Default,    // In default state
}
```

#### Credit Scoring System
```rust
/// Comprehensive credit scoring for mutual credit
pub struct CreditScore {
    pub score: u16,                     // Overall score (0-1000)
    pub community_reputation: u16,      // Community standing
    pub payment_history: u16,           // Past payment performance
    pub network_trust: u16,             // Network relationship trust
    pub economic_activity: u16,         // Recent economic activity
    pub last_updated: u64,              // Score calculation timestamp
}

/// Calculate dynamic credit score
pub fn calculate_credit_score<C: MutualCreditStore>(
    credit_store: &C,
    account: &Did,
    community_reputation: u16,
) -> CreditScore {
    let credit_history = credit_store.get_credit_history(account);
    
    // Calculate payment history (percentage of repaid transactions)
    let total_transactions = credit_history.len() as f64;
    let repaid_transactions = credit_history
        .iter()
        .filter(|tx| tx.status == CreditTransactionStatus::Repaid)
        .count() as f64;
    
    let payment_history = if total_transactions > 0.0 {
        ((repaid_transactions / total_transactions) * 1000.0) as u16
    } else {
        500 // Default neutral score
    };
    
    // Calculate network trust (unique trading partners)
    let mut unique_partners = std::collections::HashSet::new();
    for tx in &credit_history {
        if &tx.creditor == account {
            unique_partners.insert(&tx.debtor);
        } else {
            unique_partners.insert(&tx.creditor);
        }
    }
    let network_trust = std::cmp::min(unique_partners.len() as u16 * 100, 1000);
    
    // Economic activity (transaction volume in last 90 days)
    let ninety_days_ago = SystemTimeProvider.unix_seconds() - (90 * 24 * 60 * 60);
    let total_volume: u64 = credit_history
        .iter()
        .filter(|tx| tx.created_at > ninety_days_ago)
        .map(|tx| tx.amount)
        .sum();
    let economic_activity = std::cmp::min((total_volume / 100) as u16, 1000);
    
    // Calculate overall score (weighted average)
    let score = (payment_history + community_reputation + network_trust + economic_activity) / 4;
    
    CreditScore {
        score,
        community_reputation,
        payment_history,
        network_trust,
        economic_activity,
        last_updated: SystemTimeProvider.unix_seconds(),
    }
}
```

#### Credit Extension Process
```rust
/// Extend mutual credit with validation
pub fn extend_mutual_credit<L: ResourceLedger, C: MutualCreditStore>(
    resource_ledger: &L,
    credit_store: &C,
    creditor: &Did,
    debtor: &Did,
    token_class: &TokenClassId,
    amount: u64,
    purpose: String,
    repayment_period_days: u64,
) -> Result<String, CommonError> {
    // Validate token class is for mutual credit
    let token_class_info = resource_ledger.get_class(token_class)?;
    if token_class_info.token_type != TokenType::MutualCredit {
        return Err(CommonError::InvalidInputError("Token class not for mutual credit"));
    }
    
    // Check credit limit availability
    let credit_lines = credit_store.get_account_credit_lines(debtor);
    let total_credit_limit: u64 = credit_lines
        .iter()
        .filter(|cl| cl.token_class == *token_class && cl.status == CreditLineStatus::Active)
        .map(|cl| cl.credit_limit)
        .sum();
    
    let total_credit_used: u64 = credit_lines
        .iter()
        .filter(|cl| cl.token_class == *token_class && cl.status == CreditLineStatus::Active)
        .map(|cl| cl.credit_used)
        .sum();
    
    if total_credit_used + amount > total_credit_limit {
        return Err(CommonError::PolicyDenied("Credit limit exceeded"));
    }
    
    // Create credit transaction
    let transaction_id = format!("credit_{}_{}", debtor, SystemTimeProvider.unix_seconds());
    let now = SystemTimeProvider.unix_seconds();
    let due_date = now + (repayment_period_days * 24 * 60 * 60);
    
    let transaction = MutualCreditTransaction {
        transaction_id: transaction_id.clone(),
        creditor: creditor.clone(),
        debtor: debtor.clone(),
        token_class: token_class.clone(),
        amount,
        interest_rate: 0, // Community mutual credit often has no interest
        purpose,
        created_at: now,
        due_date,
        status: CreditTransactionStatus::Active,
        repayments: Vec::new(),
    };
    
    // Record transaction
    credit_store.record_credit_transaction(transaction)?;
    
    // Issue tokens to debtor
    resource_ledger.mint(token_class, debtor, amount)?;
    
    Ok(transaction_id)
}
```

### 🛒 Cooperative Marketplace - Community Commerce

The marketplace enables fair trade within and across communities using diverse token types:

#### Marketplace Structures
```rust
/// Marketplace offer for goods or services
pub struct MarketplaceOffer {
    pub offer_id: String,                       // Unique identifier
    pub seller: Did,                           // Seller DID
    pub item_type: ItemType,                   // Type of item offered
    pub description: String,                   // Item description
    pub quantity: u64,                         // Available quantity
    pub price_per_unit: u64,                   // Price per unit
    pub payment_token_class: TokenClassId,     // Accepted payment token
    pub scope: Option<String>,                 // Geographic/community scope
    pub created_at: u64,                       // Creation timestamp
    pub expires_at: Option<u64>,               // Optional expiration
    pub status: OfferStatus,                   // Current status
}

/// Marketplace bid for offers
pub struct MarketplaceBid {
    pub bid_id: String,                         // Unique identifier
    pub buyer: Did,                            // Buyer DID
    pub offer_id: String,                      // Target offer
    pub quantity: u64,                         // Requested quantity
    pub price_per_unit: u64,                   // Offered price
    pub payment_token_class: TokenClassId,     // Payment token
    pub created_at: u64,                       // Creation timestamp
    pub expires_at: u64,                       // Bid expiration
    pub status: BidStatus,                     // Current status
}
```

#### Item Types
```rust
/// Comprehensive item classification system
pub enum ItemType {
    PhysicalGood {
        category: String,       // Product category
        condition: String,      // New, used, refurbished, etc.
    },
    Service {
        service_type: String,   // Type of service offered
        duration: Option<String>, // Duration or unit
    },
    DigitalGood {
        content_type: String,   // Software, media, etc.
        license: String,        // License terms
    },
    LaborHours {
        skill_type: String,     // Required skills
        experience_level: String, // Experience required
    },
    BulkPurchase {
        target_product: String,   // Product for bulk buying
        minimum_quantity: u64,    // Minimum for activation
    },
}
```

#### Transaction Execution
```rust
/// Execute marketplace transaction with full validation
pub fn execute_marketplace_transaction<L: ResourceLedger, M: ManaLedger, S: MarketplaceStore>(
    resource_repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    marketplace_store: &S,
    offer_id: &str,
    bid_id: &str,
    executor: &Did, // Marketplace operator or participant
) -> Result<MarketplaceTransaction, CommonError> {
    // Get and validate offer and bid
    let offer = marketplace_store.get_offer(offer_id)?;
    let bid = marketplace_store.get_bid(bid_id)?;
    
    // Validate transaction compatibility
    if bid.offer_id != offer.offer_id {
        return Err(CommonError::InvalidInputError("Bid does not match offer"));
    }
    
    if bid.status != BidStatus::Active || offer.status != OfferStatus::Active {
        return Err(CommonError::PolicyDenied("Offer or bid not active"));
    }
    
    if bid.quantity > offer.quantity {
        return Err(CommonError::PolicyDenied("Bid quantity exceeds available"));
    }
    
    // Calculate total transaction value
    let total_price = bid.price_per_unit * bid.quantity;
    
    // Execute token transfer from buyer to seller
    resource_repo.transfer(
        executor,
        &bid.payment_token_class,
        total_price,
        &bid.buyer,
        &offer.seller,
        None, // Cross-scope transactions allowed
    )?;
    
    // Charge mana fee for marketplace facilitation
    charge_mana(mana_ledger, executor, TOKEN_FEE)?;
    
    // Create transaction record
    let transaction = MarketplaceTransaction {
        transaction_id: format!("tx_{}_{}", offer_id, bid_id),
        offer_id: offer.offer_id.clone(),
        bid_id: bid.bid_id.clone(),
        seller: offer.seller.clone(),
        buyer: bid.buyer.clone(),
        item_type: offer.item_type.clone(),
        quantity: bid.quantity,
        price_per_unit: bid.price_per_unit,
        total_price,
        payment_token_class: bid.payment_token_class.clone(),
        completed_at: SystemTimeProvider.unix_seconds(),
        status: TransactionStatus::Pending,
        fulfillment: FulfillmentDetails {
            method: FulfillmentMethod::Remote,
            expected_date: None,
            actual_date: None,
            tracking_info: None,
        },
    };
    
    // Record transaction and update offer/bid status
    marketplace_store.record_transaction(transaction.clone())?;
    
    // Update offer quantity
    let mut updated_offer = offer.clone();
    updated_offer.quantity -= bid.quantity;
    if updated_offer.quantity == 0 {
        updated_offer.status = OfferStatus::Fulfilled;
    }
    marketplace_store.update_offer(updated_offer)?;
    
    // Update bid status
    let mut updated_bid = bid.clone();
    updated_bid.status = BidStatus::Accepted;
    marketplace_store.update_bid(updated_bid)?;
    
    Ok(transaction)
}
```

### 🤲 Mutual Aid - Emergency Resource Coordination

Built-in mutual aid capabilities for crisis response and community support:

```rust
/// Grant mutual aid tokens for emergency assistance
pub fn grant_mutual_aid<L: ResourceLedger, M: ManaLedger>(
    resource_repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    grantor: &Did,
    recipient: &Did,
    amount: u64,
    reason: &str,
) -> Result<(), CommonError> {
    // Mutual aid uses special MUTUAL_AID_CLASS token
    resource_repo.mint(grantor, MUTUAL_AID_CLASS, amount, recipient, None)?;
    
    // Record aid transaction (reduced mana cost for mutual aid)
    let aid_fee = TOKEN_FEE / 2; // Half price for mutual aid
    charge_mana(mana_ledger, grantor, aid_fee)?;
    
    info!("Granted {} mutual aid tokens from {} to {} for: {}", 
          amount, grantor, recipient, reason);
    
    Ok(())
}

/// Use mutual aid tokens for emergency needs
pub fn use_mutual_aid<L: ResourceLedger>(
    resource_repo: &ResourceRepositoryAdapter<L>,
    user: &Did,
    amount: u64,
    purpose: &str,
) -> Result<(), CommonError> {
    // Burn mutual aid tokens when used
    resource_repo.burn(user, MUTUAL_AID_CLASS, amount, user, None)?;
    
    info!("Used {} mutual aid tokens by {} for: {}", 
          amount, user, purpose);
    
    Ok(())
}
```

## Advanced Economic Features

### 🎯 Bounty System - Incentivized Collaboration

```rust
/// Bounty for community work and contributions
pub struct Bounty {
    pub id: u64,                    // Unique bounty ID
    pub description: String,        // Work description
    pub amount: u64,               // Reward amount
    pub class_id: String,          // Token class for reward
    pub issuer: Did,               // Bounty creator
    pub claimant: Option<Did>,     // Who claimed it
    pub paid: bool,                // Payment status
}

/// Bounty management system
pub struct BountyManager<L: ManaLedger, R: ResourceLedger> {
    pub bounties: HashMap<u64, Bounty>,
    pub repo: ResourceRepositoryAdapter<R>,
    pub mana: L,
    next_id: u64,
}

impl<L: ManaLedger, R: ResourceLedger> BountyManager<L, R> {
    /// Create new bounty for community work
    pub fn create_bounty(
        &mut self,
        issuer: Did,
        class_id: &str,
        amount: u64,
        description: &str,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        
        self.bounties.insert(id, Bounty {
            id,
            description: description.into(),
            amount,
            class_id: class_id.into(),
            issuer,
            claimant: None,
            paid: false,
        });
        
        id
    }
    
    /// Claim bounty for completion
    pub fn claim_bounty(&mut self, id: u64, claimant: Did) -> Result<(), CommonError> {
        let bounty = self.bounties.get_mut(&id)
            .ok_or_else(|| CommonError::ResourceNotFound("bounty".into()))?;
            
        if bounty.paid {
            return Err(CommonError::PolicyDenied("already paid".into()));
        }
        
        bounty.claimant = Some(claimant);
        Ok(())
    }
}
```

### 📈 Reputation-Based Economics

Economic operations that consider reputation for fair pricing and access:

```rust
/// Calculate mana price based on reputation (better reputation = lower costs)
pub fn price_by_reputation(base_price: u64, reputation: u64) -> u64 {
    let denom = 100u128 + reputation as u128;
    let num = (base_price as u128) * 100u128;
    (num / denom) as u64
}

/// Credit mana based on reputation scores
pub fn credit_by_reputation(
    ledger: &dyn ManaLedger,
    reputation_store: &dyn ReputationStore,
    base_amount: u64,
) -> Result<(), CommonError> {
    for did in ledger.all_accounts() {
        let reputation = reputation_store.get_reputation(&did);
        let credit_amount = reputation.saturating_mul(base_amount);
        ledger.credit(&did, credit_amount)?;
    }
    Ok(())
}

/// Mint tokens with reputation-based pricing
pub fn mint_tokens_with_reputation<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    reputation_store: &dyn ReputationStore,
    issuer: &Did,
    class_id: &str,
    amount: u64,
    recipient: &Did,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    let reputation = reputation_store.get_reputation(issuer);
    let adjusted_cost = price_by_reputation(TOKEN_FEE, reputation);
    charge_mana(mana_ledger, issuer, adjusted_cost)?;
    repo.mint(issuer, class_id, amount, recipient, scope)
}
```

### 📊 Economic Analytics and Exploration

```rust
/// Economic flow statistics for community insights
pub struct FlowStats {
    pub total_volume: u64,              // Total transaction volume
    pub transaction_count: usize,       // Number of transactions
    pub unique_participants: usize,     // Unique participating DIDs
    pub average_transaction_size: f64,  // Average transaction value
    pub flow_velocity: f64,             // Economic velocity measure
}

/// Ledger explorer for economic analysis
pub trait LedgerExplorer {
    /// Analyze economic flows over time period
    fn analyze_flows(&self, start_time: u64, end_time: u64) -> FlowStats;
    
    /// Get top participants by volume
    fn top_participants(&self, limit: usize) -> Vec<(Did, u64)>;
    
    /// Calculate economic health metrics
    fn calculate_health_metrics(&self) -> HashMap<String, f64>;
}
```

## Practical Usage Examples

### Basic Mana Management
```rust
use icn_economics::{FileManaLedger, charge_mana, credit_mana};
use icn_common::Did;
use std::path::PathBuf;

// Create persistent mana ledger
let mana_ledger = FileManaLedger::new(PathBuf::from("./mana.json"))?;

// Setup initial balances
let alice = Did::from_str("did:example:alice")?;
let bob = Did::from_str("did:example:bob")?;

mana_ledger.set_balance(&alice, 1000)?;
mana_ledger.set_balance(&bob, 500)?;

// Charge mana for resource usage
charge_mana(&mana_ledger, &alice, 100)?;

// Credit mana for contributions
credit_mana(&mana_ledger, &bob, 200)?;

// Check updated balances
assert_eq!(mana_ledger.get_balance(&alice), 900);
assert_eq!(mana_ledger.get_balance(&bob), 700);
```

### Multi-Token Resource System
```rust
use icn_economics::{FileResourceLedger, TokenClass, TokenType, ResourceRepositoryAdapter};

// Create resource ledger
let resource_ledger = FileResourceLedger::new(PathBuf::from("./tokens.json"))?;
let resource_repo = ResourceRepositoryAdapter::new(resource_ledger);

// Create time banking token class
let time_class = TokenClass::new_time_banking(
    "Community Hours".to_string(),
    "Labor hours in our community".to_string(),
    alice.clone(),
    Some("neighborhood_coop".to_string()),
);

resource_repo.ledger().create_class("TIME", time_class)?;

// Create mutual credit token class
let credit_class = TokenClass::new_mutual_credit(
    "Community Credit".to_string(),
    "Local mutual credit currency".to_string(),
    "CREDIT".to_string(),
    2, // 2 decimal places
    alice.clone(),
    Some("neighborhood_coop".to_string()),
    1000000, // 1M token supply cap
);

resource_repo.ledger().create_class("CREDIT", credit_class)?;

// Mint time tokens for work performed
resource_repo.mint(&alice, "TIME", 800, &bob, None)?; // 8.00 hours

// Mint credit tokens for community member
resource_repo.mint(&alice, "CREDIT", 50000, &bob, None)?; // 500.00 credits
```

### Marketplace Transaction
```rust
use icn_economics::marketplace::*;

// Create marketplace offer
let offer = MarketplaceOffer {
    offer_id: "offer_001".to_string(),
    seller: alice.clone(),
    item_type: ItemType::Service {
        service_type: "Web Development".to_string(),
        duration: Some("40 hours".to_string()),
    },
    description: "Custom website development for small business".to_string(),
    quantity: 1,
    price_per_unit: 4000, // 40.00 TIME tokens (40 hours)
    payment_token_class: "TIME".to_string(),
    scope: Some("tech_coop".to_string()),
    created_at: SystemTimeProvider.unix_seconds(),
    expires_at: Some(SystemTimeProvider.unix_seconds() + 86400 * 30), // 30 days
    status: OfferStatus::Active,
    metadata: HashMap::new(),
};

// Create marketplace bid
let bid = MarketplaceBid {
    bid_id: "bid_001".to_string(),
    buyer: bob.clone(),
    offer_id: "offer_001".to_string(),
    quantity: 1,
    price_per_unit: 4000,
    payment_token_class: "TIME".to_string(),
    created_at: SystemTimeProvider.unix_seconds(),
    expires_at: SystemTimeProvider.unix_seconds() + 86400 * 7, // 7 days
    status: BidStatus::Active,
    metadata: HashMap::new(),
};

// Execute transaction
let marketplace_store = InMemoryMarketplaceStore::new();
marketplace_store.create_offer(offer)?;
marketplace_store.create_bid(bid)?;

let transaction = execute_marketplace_transaction(
    &resource_repo,
    &mana_ledger,
    &marketplace_store,
    "offer_001",
    "bid_001",
    &alice, // Alice facilitates as marketplace operator
)?;

assert_eq!(transaction.status, TransactionStatus::Pending);
assert_eq!(transaction.total_price, 4000);
```

### Time Banking Implementation
```rust
use icn_economics::time_banking::*;

// Setup time banking system
let time_store = InMemoryTimeBankingStore::new();

// Record work and mint time tokens
let record_id = record_and_mint_time_tokens(
    resource_repo.ledger(),
    &time_store,
    &"TIME".to_string(),
    &alice, // Worker
    &bob,   // Beneficiary
    "Childcare".to_string(),
    "Watched children while parents attended meeting".to_string(),
    3.5, // 3.5 hours
    "Basic".to_string(),
)?;

// Verify work was recorded
let record = time_store.get_time_record(&record_id).unwrap();
assert_eq!(record.hours, 3.5);
assert_eq!(record.status, TimeRecordStatus::Recorded);

// Beneficiary verifies the work
verify_time_record(&time_store, &record_id, &bob)?;

// Check that alice received time tokens (3.5 hours = 350 token units)
assert_eq!(resource_repo.ledger().get_balance("TIME", &alice), 350);
```

### Mutual Credit System
```rust
use icn_economics::mutual_credit::*;

// Setup mutual credit system
let credit_store = InMemoryMutualCreditStore::new();

// Create credit line for community member
let credit_line = CreditLine {
    credit_id: "credit_bob_001".to_string(),
    account: bob.clone(),
    token_class: "CREDIT".to_string(),
    credit_limit: 100000, // 1000.00 credits
    credit_used: 0,
    interest_rate: 0, // No interest in mutual credit
    created_at: SystemTimeProvider.unix_seconds(),
    expires_at: None, // No expiration
    status: CreditLineStatus::Active,
    credit_score: CreditScore {
        score: 750,
        community_reputation: 800,
        payment_history: 700,
        network_trust: 600,
        economic_activity: 900,
        last_updated: SystemTimeProvider.unix_seconds(),
    },
    metadata: HashMap::new(),
};

credit_store.create_credit_line(credit_line)?;

// Extend mutual credit
let transaction_id = extend_mutual_credit(
    resource_repo.ledger(),
    &credit_store,
    &alice, // Creditor
    &bob,   // Debtor
    &"CREDIT".to_string(),
    50000, // 500.00 credits
    "Start local food delivery service".to_string(),
    90, // 90 days to repay
)?;

// Verify credit was extended
let transaction = credit_store.get_credit_transaction(&transaction_id).unwrap();
assert_eq!(transaction.amount, 50000);
assert_eq!(transaction.status, CreditTransactionStatus::Active);

// Check that bob received credit tokens
assert_eq!(resource_repo.ledger().get_balance("CREDIT", &bob), 50000);
```

## Integration Points

### 🔌 Runtime Integration
- **Host ABI Functions**: Access economic functions from WASM modules
- **Mana Enforcement**: Automatic resource consumption tracking
- **Economic Events**: Real-time transaction notifications

### 🌐 Network Integration  
- **Cross-Node Transactions**: Coordinate transactions across nodes
- **Federation Economics**: Economic relationships between federations
- **Reputation Integration**: Economic actions influence reputation scores

### 🏛️ Governance Integration
- **Democratic Resource Allocation**: Community-controlled economic policies
- **Budget Management**: Governance-approved spending and allocation
- **Economic Parameter Governance**: Community control over economic rules

## Security and Policy Enforcement

### 🛡️ Economic Security Measures
```rust
// Spending limits and validation
const MAX_SPEND_LIMIT: u64 = 1000;
const TOKEN_FEE: u64 = 1;

// Balance validation before operations
if available_balance < required_amount {
    return Err(CommonError::PolicyDenied("Insufficient balance"));
}

// Rate limiting for economic operations
if recent_transaction_count > rate_limit {
    return Err(CommonError::PolicyDenied("Rate limit exceeded"));
}
```

### 📊 Monitoring and Metrics
```rust
// Prometheus metrics for economic monitoring
static SPEND_MANA_CALLS: Counter;
static CREDIT_MANA_CALLS: Counter;
static GET_BALANCE_CALLS: Counter;

// Event tracking for audit trails
pub enum LedgerEvent {
    Debit { did: Did, amount: u64 },
    Credit { did: Did, amount: u64 },
    SetBalance { did: Did, amount: u64 },
}
```

---

**Key Insight**: ICN's economics system creates a comprehensive foundation for cooperative digital economies, supporting diverse value exchange models that strengthen communities while ensuring fair access to resources and democratic control over economic policies. 