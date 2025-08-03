use icn_common::Did;
use icn_economics::{
    marketplace::{
        InMemoryMarketplaceStore, ItemType, MarketplaceBid, MarketplaceOffer, MarketplaceStore,
        OfferFilter, OfferStatus,
    },
    TokenClassId,
};
use std::str::FromStr;

#[test]
fn cross_cooperative_marketplace_flow() {
    let marketplace = InMemoryMarketplaceStore::new();
    
    // Create a cross-cooperative offer from Federation A
    let seller = Did::from_str("did:federation-a:seller").unwrap();
    let offer = MarketplaceOffer::new_cross_cooperative(
        "offer-001".to_string(),
        seller.clone(),
        ItemType::Service {
            service_type: "consulting".to_string(),
            duration: Some("4 hours".to_string()),
        },
        "Technical consulting services".to_string(),
        10, // quantity
        50, // price per unit
        "consulting-credits".to_string(),
        "federation-network".to_string(), // Available to federation network
        2, // Trust level required
    );
    
    marketplace.create_offer(offer.clone()).unwrap();
    
    // Create a cross-cooperative bid from Federation B
    let buyer = Did::from_str("did:federation-b:buyer").unwrap();
    let trust_attestations = vec![
        "federation-b-trust-level-3".to_string(),
        "cooperative-network-verified".to_string(),
        "reputation-score-85".to_string(),
    ];
    
    let bid = MarketplaceBid::new_cross_cooperative_bid(
        "bid-001".to_string(),
        buyer.clone(),
        "offer-001".to_string(),
        5, // quantity requested
        50, // matching price
        "consulting-credits".to_string(),
        "federation-b".to_string(), // Different federation, but should work with network scope
        trust_attestations,
        24, // expires in 24 hours
    );
    
    marketplace.create_bid(bid.clone()).unwrap();
    
    // Validate the cross-cooperative bid
    let validation_result = marketplace
        .validate_cross_cooperative_bid(&bid, &offer)
        .unwrap();
    
    assert!(validation_result, "Cross-cooperative bid should be valid");
    
    // Test federation filtering
    let federated_offers = marketplace.get_federated_offers("federation-network");
    assert_eq!(federated_offers.len(), 1);
    assert_eq!(federated_offers[0].offer_id, "offer-001");
    
    // Test filtering by federation scope
    let filter = OfferFilter {
        federation_scope: Some("federation-network".to_string()),
        ..Default::default()
    };
    let filtered_offers = marketplace.list_offers(filter);
    assert_eq!(filtered_offers.len(), 1);
    
    // Test trust level filtering
    let filter = OfferFilter {
        trust_level: Some(1), // Lower trust level
        ..Default::default()
    };
    let filtered_offers = marketplace.list_offers(filter);
    assert_eq!(filtered_offers.len(), 0); // Should not return offer requiring trust level 2
    
    let filter = OfferFilter {
        trust_level: Some(3), // Higher trust level
        ..Default::default()
    };
    let filtered_offers = marketplace.list_offers(filter);
    assert_eq!(filtered_offers.len(), 1); // Should return the offer
}

#[test]
fn local_marketplace_flow() {
    let marketplace = InMemoryMarketplaceStore::new();
    
    // Create a local-only offer
    let seller = Did::from_str("did:local:seller").unwrap();
    let offer = MarketplaceOffer::new_service(icn_economics::marketplace::ServiceConfig {
        offer_id: "local-offer-001".to_string(),
        seller: seller.clone(),
        description: "Local gardening service".to_string(),
        category: "gardening".to_string(),
        duration_hours: 2,
        quantity: 5,
        price_per_unit: 25,
        payment_token_class: "garden-credits".to_string(),
    });
    
    marketplace.create_offer(offer.clone()).unwrap();
    
    // Create a local bid
    let buyer = Did::from_str("did:local:buyer").unwrap();
    let bid = MarketplaceBid::new_bid(
        "local-bid-001".to_string(),
        buyer.clone(),
        "local-offer-001".to_string(),
        3, // quantity
        25, // price
        "garden-credits".to_string(),
        48, // expires in 48 hours
    );
    
    marketplace.create_bid(bid.clone()).unwrap();
    
    // Validate local bid (should pass)
    let validation_result = marketplace
        .validate_cross_cooperative_bid(&bid, &offer)
        .unwrap();
    
    assert!(validation_result, "Local bid should be valid");
    
    // Create a cross-cooperative bid on local offer (should fail)
    let cross_coop_buyer = Did::from_str("did:federation-c:buyer").unwrap();
    let cross_coop_bid = MarketplaceBid::new_cross_cooperative_bid(
        "cross-bid-001".to_string(),
        cross_coop_buyer,
        "local-offer-001".to_string(),
        2,
        25,
        "garden-credits".to_string(),
        "federation-c".to_string(),
        vec!["trust-attestation".to_string()],
        24,
    );
    
    // This should fail because the offer doesn't allow cross-cooperative bids
    let validation_result = marketplace
        .validate_cross_cooperative_bid(&cross_coop_bid, &offer)
        .unwrap();
    
    assert!(!validation_result, "Cross-cooperative bid on local offer should be invalid");
}