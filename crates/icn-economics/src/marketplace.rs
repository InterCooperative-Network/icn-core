use crate::TokenClassId;
use icn_common::{CommonError, Did, SystemTimeProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an offer to sell goods or services for tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceOffer {
    /// Unique identifier for this offer.
    pub offer_id: String,
    /// DID of the account making the offer.
    pub seller: Did,
    /// Type of item being offered.
    pub item_type: ItemType,
    /// Description of the item or service.
    pub description: String,
    /// Quantity available.
    pub quantity: u64,
    /// Price per unit in the specified token class.
    pub price_per_unit: u64,
    /// Token class that payment should be made in.
    pub payment_token_class: TokenClassId,
    /// Geographic or community scope for this offer.
    pub scope: Option<String>,
    /// Cross-cooperative federation scope (None = local only).
    pub federation_scope: Option<String>,
    /// Cross-cooperative trust level required for buyers.
    pub trust_level_required: u32,
    /// Unix timestamp when offer was created.
    pub created_at: u64,
    /// Unix timestamp when offer expires (None = no expiration).
    pub expires_at: Option<u64>,
    /// Current status of the offer.
    pub status: OfferStatus,
    /// Optional metadata about the offer.
    pub metadata: HashMap<String, String>,
}

/// Represents a bid to buy goods or services with tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceBid {
    /// Unique identifier for this bid.
    pub bid_id: String,
    /// DID of the account making the bid.
    pub buyer: Did,
    /// Offer this bid is responding to.
    pub offer_id: String,
    /// Quantity requested.
    pub quantity: u64,
    /// Price per unit offered (can be different from asking price).
    pub price_per_unit: u64,
    /// Token class for payment.
    pub payment_token_class: TokenClassId,
    /// Cross-cooperative federation the buyer belongs to.
    pub buyer_federation: Option<String>,
    /// Trust attestations from the buyer's federation.
    pub trust_attestations: Vec<String>,
    /// Unix timestamp when bid was created.
    pub created_at: u64,
    /// Unix timestamp when bid expires.
    pub expires_at: u64,
    /// Current status of the bid.
    pub status: BidStatus,
    /// Optional metadata about the bid.
    pub metadata: HashMap<String, String>,
}

/// Types of items that can be traded in the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    /// Physical goods.
    PhysicalGood {
        /// Category of the good.
        category: String,
        /// Condition (new, used, etc.).
        condition: String,
    },
    /// Services offered by individuals or organizations.
    Service {
        /// Type of service.
        service_type: String,
        /// Duration or unit of service.
        duration: Option<String>,
    },
    /// Digital goods or content.
    DigitalGood {
        /// Type of digital content.
        content_type: String,
        /// License terms.
        license: String,
    },
    /// Labor hours or time banking.
    LaborHours {
        /// Type of work or skill.
        skill_type: String,
        /// Experience level.
        experience_level: String,
    },
    /// Bulk purchasing opportunity.
    BulkPurchase {
        /// Target product for bulk buying.
        target_product: String,
        /// Minimum quantity needed to activate.
        minimum_quantity: u64,
    },
}

/// Status of a marketplace offer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OfferStatus {
    /// Offer is active and accepting bids.
    Active,
    /// Offer is temporarily paused.
    Paused,
    /// Offer has been fulfilled.
    Fulfilled,
    /// Offer has been cancelled.
    Cancelled,
    /// Offer has expired.
    Expired,
}

/// Status of a marketplace bid.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BidStatus {
    /// Bid is active and waiting for response.
    Active,
    /// Bid has been accepted by seller.
    Accepted,
    /// Bid has been rejected by seller.
    Rejected,
    /// Bid has been withdrawn by buyer.
    Withdrawn,
    /// Bid has expired.
    Expired,
}

/// Represents a completed marketplace transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceTransaction {
    /// Unique identifier for this transaction.
    pub transaction_id: String,
    /// Offer that was fulfilled.
    pub offer_id: String,
    /// Winning bid.
    pub bid_id: String,
    /// Seller DID.
    pub seller: Did,
    /// Buyer DID.
    pub buyer: Did,
    /// Item type and details.
    pub item_type: ItemType,
    /// Quantity traded.
    pub quantity: u64,
    /// Final price per unit.
    pub price_per_unit: u64,
    /// Total price paid.
    pub total_price: u64,
    /// Token class used for payment.
    pub payment_token_class: TokenClassId,
    /// Unix timestamp when transaction was completed.
    pub completed_at: u64,
    /// Status of the transaction.
    pub status: TransactionStatus,
    /// Delivery or fulfillment details.
    pub fulfillment: FulfillmentDetails,
}

/// Status of a marketplace transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    /// Payment has been made, awaiting fulfillment.
    Pending,
    /// Transaction is being fulfilled.
    InProgress,
    /// Transaction has been completed successfully.
    Completed,
    /// Transaction was cancelled before completion.
    Cancelled,
    /// There's a dispute that needs resolution.
    Disputed,
}

/// Details about how an item or service will be delivered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentDetails {
    /// Method of delivery or service provision.
    pub method: FulfillmentMethod,
    /// Expected delivery or completion date.
    pub expected_date: Option<u64>,
    /// Actual delivery or completion date.
    pub actual_date: Option<u64>,
    /// Tracking information or notes.
    pub tracking_info: Option<String>,
}

/// Methods for fulfilling marketplace transactions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FulfillmentMethod {
    /// Physical delivery to an address.
    PhysicalDelivery {
        /// Delivery address.
        address: String,
    },
    /// Digital delivery via download or email.
    DigitalDelivery {
        /// Delivery mechanism.
        method: String,
    },
    /// In-person service or pickup.
    InPerson {
        /// Location for service or pickup.
        location: String,
    },
    /// Remote service provision.
    Remote,
}

/// Trait for marketplace functionality.
pub trait MarketplaceStore: Send + Sync {
    /// Create a new offer in the marketplace.
    fn create_offer(&self, offer: MarketplaceOffer) -> Result<(), CommonError>;
    /// Get an offer by ID.
    fn get_offer(&self, offer_id: &str) -> Option<MarketplaceOffer>;
    /// Update an existing offer.
    fn update_offer(&self, offer: MarketplaceOffer) -> Result<(), CommonError>;
    /// List all offers matching criteria.
    fn list_offers(&self, filter: OfferFilter) -> Vec<MarketplaceOffer>;
    /// Create a new bid on an offer.
    fn create_bid(&self, bid: MarketplaceBid) -> Result<(), CommonError>;
    /// Get a bid by ID.
    fn get_bid(&self, bid_id: &str) -> Option<MarketplaceBid>;
    /// Update an existing bid.
    fn update_bid(&self, bid: MarketplaceBid) -> Result<(), CommonError>;
    /// List all bids for an offer.
    fn list_bids_for_offer(&self, offer_id: &str) -> Vec<MarketplaceBid>;
    /// Record a completed transaction.
    fn record_transaction(&self, transaction: MarketplaceTransaction) -> Result<(), CommonError>;
    /// Get transaction history for a user.
    fn get_transaction_history(&self, did: &Did) -> Vec<MarketplaceTransaction>;
    /// Validate cross-cooperative bid based on trust requirements.
    fn validate_cross_cooperative_bid(&self, bid: &MarketplaceBid, offer: &MarketplaceOffer) -> Result<bool, CommonError>;
    /// Get offers from other federations.
    fn get_federated_offers(&self, federation: &str) -> Vec<MarketplaceOffer>;
}

/// Filter criteria for searching marketplace offers.
#[derive(Debug, Clone, Default)]
pub struct OfferFilter {
    /// Filter by item type.
    pub item_type: Option<ItemType>,
    /// Filter by price range.
    pub price_range: Option<(u64, u64)>,
    /// Filter by token class.
    pub payment_token_class: Option<TokenClassId>,
    /// Filter by seller.
    pub seller: Option<Did>,
    /// Filter by scope.
    pub scope: Option<String>,
    /// Filter by federation scope.
    pub federation_scope: Option<String>,
    /// Filter by required trust level.
    pub trust_level: Option<u32>,
    /// Filter by status.
    pub status: Option<OfferStatus>,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

/// In-memory marketplace store for testing and development.
#[derive(Default)]
pub struct InMemoryMarketplaceStore {
    offers: std::sync::Mutex<HashMap<String, MarketplaceOffer>>,
    bids: std::sync::Mutex<HashMap<String, MarketplaceBid>>,
    transactions: std::sync::Mutex<HashMap<String, MarketplaceTransaction>>,
}

impl InMemoryMarketplaceStore {
    /// Create a new in-memory marketplace store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl MarketplaceStore for InMemoryMarketplaceStore {
    fn create_offer(&self, offer: MarketplaceOffer) -> Result<(), CommonError> {
        let mut offers = self.offers.lock().unwrap();
        if offers.contains_key(&offer.offer_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Offer {} already exists",
                offer.offer_id
            )));
        }
        offers.insert(offer.offer_id.clone(), offer);
        Ok(())
    }

    fn get_offer(&self, offer_id: &str) -> Option<MarketplaceOffer> {
        let offers = self.offers.lock().unwrap();
        offers.get(offer_id).cloned()
    }

    fn update_offer(&self, offer: MarketplaceOffer) -> Result<(), CommonError> {
        let mut offers = self.offers.lock().unwrap();
        offers.insert(offer.offer_id.clone(), offer);
        Ok(())
    }

    fn list_offers(&self, filter: OfferFilter) -> Vec<MarketplaceOffer> {
        let offers = self.offers.lock().unwrap();
        let mut results: Vec<MarketplaceOffer> = offers
            .values()
            .filter(|offer| {
                // Apply filters
                if let Some(ref item_type) = filter.item_type {
                    if &offer.item_type != item_type {
                        return false;
                    }
                }
                if let Some((min_price, max_price)) = filter.price_range {
                    if offer.price_per_unit < min_price || offer.price_per_unit > max_price {
                        return false;
                    }
                }
                if let Some(ref token_class) = filter.payment_token_class {
                    if &offer.payment_token_class != token_class {
                        return false;
                    }
                }
                if let Some(ref seller) = filter.seller {
                    if &offer.seller != seller {
                        return false;
                    }
                }
                if let Some(ref scope) = filter.scope {
                    if offer.scope.as_ref() != Some(scope) {
                        return false;
                    }
                }
                if let Some(ref federation_scope) = filter.federation_scope {
                    if offer.federation_scope.as_ref() != Some(federation_scope) {
                        return false;
                    }
                }
                if let Some(trust_level) = filter.trust_level {
                    if offer.trust_level_required > trust_level {
                        return false;
                    }
                }
                if let Some(ref status) = filter.status {
                    if &offer.status != status {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Sort by creation date (newest first)
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        results
    }

    fn create_bid(&self, bid: MarketplaceBid) -> Result<(), CommonError> {
        let mut bids = self.bids.lock().unwrap();
        if bids.contains_key(&bid.bid_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Bid {} already exists",
                bid.bid_id
            )));
        }
        bids.insert(bid.bid_id.clone(), bid);
        Ok(())
    }

    fn get_bid(&self, bid_id: &str) -> Option<MarketplaceBid> {
        let bids = self.bids.lock().unwrap();
        bids.get(bid_id).cloned()
    }

    fn update_bid(&self, bid: MarketplaceBid) -> Result<(), CommonError> {
        let mut bids = self.bids.lock().unwrap();
        bids.insert(bid.bid_id.clone(), bid);
        Ok(())
    }

    fn list_bids_for_offer(&self, offer_id: &str) -> Vec<MarketplaceBid> {
        let bids = self.bids.lock().unwrap();
        let mut results: Vec<MarketplaceBid> = bids
            .values()
            .filter(|bid| bid.offer_id == offer_id)
            .cloned()
            .collect();

        // Sort by bid amount (highest first)
        results.sort_by(|a, b| b.price_per_unit.cmp(&a.price_per_unit));
        results
    }

    fn record_transaction(&self, transaction: MarketplaceTransaction) -> Result<(), CommonError> {
        let mut transactions = self.transactions.lock().unwrap();
        if transactions.contains_key(&transaction.transaction_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Transaction {} already exists",
                transaction.transaction_id
            )));
        }
        transactions.insert(transaction.transaction_id.clone(), transaction);
        Ok(())
    }

    fn get_transaction_history(&self, did: &Did) -> Vec<MarketplaceTransaction> {
        let transactions = self.transactions.lock().unwrap();
        let mut results: Vec<MarketplaceTransaction> = transactions
            .values()
            .filter(|tx| &tx.seller == did || &tx.buyer == did)
            .cloned()
            .collect();

        // Sort by completion date (newest first)
        results.sort_by(|a, b| b.completed_at.cmp(&a.completed_at));
        results
    }

    fn validate_cross_cooperative_bid(&self, bid: &MarketplaceBid, offer: &MarketplaceOffer) -> Result<bool, CommonError> {
        // Check if offer allows cross-cooperative bids
        if offer.federation_scope.is_none() {
            // Local-only offer
            return Ok(bid.buyer_federation.is_none());
        }

        // Check trust level requirements
        if let Some(buyer_federation) = &bid.buyer_federation {
            // Cross-cooperative bid - validate trust attestations
            if bid.trust_attestations.len() < offer.trust_level_required as usize {
                return Ok(false);
            }

            // Validate federation scope compatibility
            if let Some(required_federation) = &offer.federation_scope {
                // Special federation scopes
                if required_federation == "*" || required_federation == "global" {
                    // Global offer - any federation can participate
                } else if required_federation.ends_with("-network") {
                    // Network scope - any federation in the network can participate
                    // For now, we'll be permissive and allow any federation
                } else if buyer_federation != required_federation {
                    // Specific federation required and buyer doesn't match
                    return Ok(false);
                }
            }

            // Additional validation could include:
            // - Verifying trust attestation signatures
            // - Checking federation reputation
            // - Validating cross-cooperative exchange rates
            
            return Ok(true);
        }

        // Local bid on cross-cooperative offer is always allowed
        Ok(true)
    }

    fn get_federated_offers(&self, federation: &str) -> Vec<MarketplaceOffer> {
        let offers = self.offers.lock().unwrap();
        offers
            .values()
            .filter(|offer| {
                offer.federation_scope.as_ref().map_or(false, |scope| {
                    scope == federation || scope == "*"
                }) && offer.status == OfferStatus::Active
            })
            .cloned()
            .collect()
    }
}

/// Configuration for creating a physical good marketplace offer
#[derive(Debug, Clone)]
pub struct PhysicalGoodConfig {
    pub offer_id: String,
    pub seller: Did,
    pub description: String,
    pub category: String,
    pub condition: String,
    pub quantity: u64,
    pub price_per_unit: u64,
    pub payment_token_class: TokenClassId,
}

/// Configuration for creating a service marketplace offer
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub offer_id: String,
    pub seller: Did,
    pub description: String,
    pub category: String,
    pub duration_hours: u64,
    pub quantity: u64,
    pub price_per_unit: u64,
    pub payment_token_class: TokenClassId,
}

/// Configuration for creating a labor hours marketplace offer
#[derive(Debug, Clone)]
pub struct LaborHoursConfig {
    pub offer_id: String,
    pub seller: Did,
    pub description: String,
    pub skill_level: String,
    pub duration_hours: u64,
    pub quantity: u64,
    pub price_per_unit: u64,
    pub payment_token_class: TokenClassId,
}

/// Helper functions for creating marketplace items.
impl MarketplaceOffer {
    /// Create a new marketplace offer for physical goods.
    pub fn new_physical_good(config: PhysicalGoodConfig) -> Self {
        Self {
            offer_id: config.offer_id,
            seller: config.seller,
            item_type: ItemType::PhysicalGood {
                category: config.category,
                condition: config.condition,
            },
            description: config.description,
            quantity: config.quantity,
            price_per_unit: config.price_per_unit,
            payment_token_class: config.payment_token_class,
            scope: None,
            federation_scope: None, // Default to local only
            trust_level_required: 0, // No trust requirements for basic goods
            created_at: 0, // Will be set by the marketplace
            expires_at: None,
            status: OfferStatus::Active,
            metadata: HashMap::new(),
        }
    }

    /// Create a new marketplace offer for services.
    pub fn new_service(config: ServiceConfig) -> Self {
        Self {
            offer_id: config.offer_id,
            seller: config.seller,
            item_type: ItemType::Service {
                service_type: config.category,
                duration: Some(format!("{} hours", config.duration_hours)),
            },
            description: config.description,
            quantity: config.quantity,
            price_per_unit: config.price_per_unit,
            payment_token_class: config.payment_token_class,
            scope: None,
            federation_scope: None, // Default to local only
            trust_level_required: 1, // Services require basic trust
            created_at: 0, // Will be set by the marketplace
            expires_at: None,
            status: OfferStatus::Active,
            metadata: HashMap::new(),
        }
    }

    /// Create a new marketplace offer for labor hours.
    pub fn new_labor_hours(config: LaborHoursConfig) -> Self {
        Self {
            offer_id: config.offer_id,
            seller: config.seller,
            item_type: ItemType::LaborHours {
                skill_type: format!("{} ({}h)", config.skill_level, config.duration_hours),
                experience_level: config.skill_level,
            },
            description: config.description,
            quantity: config.quantity,
            price_per_unit: config.price_per_unit,
            payment_token_class: config.payment_token_class,
            scope: None,
            federation_scope: None, // Default to local only
            trust_level_required: 2, // Labor requires higher trust
            created_at: 0, // Will be set by the marketplace
            expires_at: None,
            status: OfferStatus::Active,
            metadata: HashMap::new(),
        }
    }

    /// Create a new cross-cooperative marketplace offer.
    pub fn new_cross_cooperative(
        offer_id: String,
        seller: Did,
        item_type: ItemType,
        description: String,
        quantity: u64,
        price_per_unit: u64,
        payment_token_class: TokenClassId,
        federation_scope: String,
        trust_level_required: u32,
    ) -> Self {
        Self {
            offer_id,
            seller,
            item_type,
            description,
            quantity,
            price_per_unit,
            payment_token_class,
            scope: None,
            federation_scope: Some(federation_scope),
            trust_level_required,
            created_at: 0, // Will be set by the marketplace
            expires_at: None,
            status: OfferStatus::Active,
            metadata: HashMap::new(),
        }
    }
}

impl MarketplaceBid {
    /// Create a new local bid on a marketplace offer.
    pub fn new_bid(
        bid_id: String,
        buyer: Did,
        offer_id: String,
        quantity: u64,
        price_per_unit: u64,
        payment_token_class: TokenClassId,
        expires_in_hours: u64,
    ) -> Self {
        let now = SystemTimeProvider.unix_seconds();
        Self {
            bid_id,
            buyer,
            offer_id,
            quantity,
            price_per_unit,
            payment_token_class,
            buyer_federation: None,
            trust_attestations: Vec::new(),
            created_at: now,
            expires_at: now + (expires_in_hours * 3600),
            status: BidStatus::Active,
            metadata: HashMap::new(),
        }
    }

    /// Create a new cross-cooperative bid on a marketplace offer.
    pub fn new_cross_cooperative_bid(
        bid_id: String,
        buyer: Did,
        offer_id: String,
        quantity: u64,
        price_per_unit: u64,
        payment_token_class: TokenClassId,
        buyer_federation: String,
        trust_attestations: Vec<String>,
        expires_in_hours: u64,
    ) -> Self {
        let now = SystemTimeProvider.unix_seconds();
        Self {
            bid_id,
            buyer,
            offer_id,
            quantity,
            price_per_unit,
            payment_token_class,
            buyer_federation: Some(buyer_federation),
            trust_attestations,
            created_at: now,
            expires_at: now + (expires_in_hours * 3600),
            status: BidStatus::Active,
            metadata: HashMap::new(),
        }
    }
}
