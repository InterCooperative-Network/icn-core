#[cfg(test)]
mod scoped_token_tests {
    use crate::{
        burn_tokens, mint_tokens, transfer_tokens, ManaLedger, ResourceLedger,
        ResourceRepositoryAdapter, TokenClass, TokenType, TransferabilityRule,
    };
    use icn_common::{CommonError, Did, NodeScope};
    use icn_dag::{InMemoryDagStore};
    use icn_reputation::InMemoryReputationStore;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Mutex;
    use tempfile::tempdir;

    // Helper struct for in-memory resource ledger for testing
    #[derive(Default)]
    struct InMemoryResourceLedger {
        classes: Mutex<HashMap<String, TokenClass>>,
        balances: Mutex<HashMap<(String, Did), u64>>,
        transfer_history: Mutex<HashMap<String, HashMap<String, Vec<crate::TransferRecord>>>>,
    }

    impl ResourceLedger for InMemoryResourceLedger {
        fn create_class(&self, class_id: &String, class: TokenClass) -> Result<(), CommonError> {
            let mut classes = self.classes.lock().unwrap();
            if classes.contains_key(class_id) {
                return Err(CommonError::InvalidInputError(format!(
                    "Token class {} already exists",
                    class_id
                )));
            }
            classes.insert(class_id.clone(), class);
            Ok(())
        }

        fn get_class(&self, class_id: &String) -> Option<TokenClass> {
            let classes = self.classes.lock().unwrap();
            classes.get(class_id).cloned()
        }

        fn update_class(&self, class_id: &String, class: TokenClass) -> Result<(), CommonError> {
            let mut classes = self.classes.lock().unwrap();
            if let Some(existing_class) = classes.get(class_id) {
                if existing_class.issuer != class.issuer {
                    return Err(CommonError::PolicyDenied(
                        "Only issuer can update token class".into(),
                    ));
                }
                classes.insert(class_id.clone(), class);
                Ok(())
            } else {
                Err(CommonError::InvalidInputError(format!(
                    "Token class {} not found",
                    class_id
                )))
            }
        }

        fn list_classes(&self) -> Vec<(String, TokenClass)> {
            let classes = self.classes.lock().unwrap();
            classes
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }

        fn get_balance(&self, class_id: &String, did: &Did) -> u64 {
            let key = (class_id.clone(), did.clone());
            *self.balances.lock().unwrap().get(&key).unwrap_or(&0)
        }

        fn mint(&self, class_id: &String, owner: &Did, amount: u64) -> Result<(), CommonError> {
            let key = (class_id.clone(), owner.clone());
            let current = self.get_balance(class_id, owner);
            self.balances.lock().unwrap().insert(key, current + amount);
            Ok(())
        }

        fn burn(&self, class_id: &String, owner: &Did, amount: u64) -> Result<(), CommonError> {
            let key = (class_id.clone(), owner.clone());
            let current = self.get_balance(class_id, owner);
            if current < amount {
                return Err(CommonError::PolicyDenied("Insufficient balance".into()));
            }
            self.balances.lock().unwrap().insert(key, current - amount);
            Ok(())
        }

        fn transfer(
            &self,
            class_id: &String,
            from: &Did,
            to: &Did,
            amount: u64,
        ) -> Result<(), CommonError> {
            if !self.can_transfer(class_id, from, to, amount)? {
                return Err(CommonError::PolicyDenied(
                    "Transfer not allowed by token rules".into(),
                ));
            }
            self.burn(class_id, from, amount)?;
            self.mint(class_id, to, amount)?;
            Ok(())
        }

        fn can_transfer(
            &self,
            class_id: &String,
            _from: &Did,
            to: &Did,
            _amount: u64,
        ) -> Result<bool, CommonError> {
            let token_class = self.get_class(class_id).ok_or_else(|| {
                CommonError::InvalidInputError(format!("Token class {} not found", class_id))
            })?;

            match &token_class.transferability {
                TransferabilityRule::FreelyTransferable => Ok(true),
                TransferabilityRule::RestrictedTransfer {
                    authorized_recipients,
                } => Ok(authorized_recipients.contains(to)),
                TransferabilityRule::NonTransferable => Ok(false),
                TransferabilityRule::IssuerOnly => Ok(to == &token_class.issuer),
            }
        }

        fn get_transfer_history(&self, class_id: &String, did: &Did) -> Vec<crate::TransferRecord> {
            let history = self.transfer_history.lock().unwrap();
            history
                .get(class_id)
                .and_then(|class_history| class_history.get(&did.to_string()))
                .cloned()
                .unwrap_or_default()
        }
    }

    // Helper struct for in-memory mana ledger for testing
    #[derive(Default)]
    struct InMemoryManaLedger {
        balances: Mutex<HashMap<Did, u64>>,
    }

    impl ManaLedger for InMemoryManaLedger {
        fn get_balance(&self, did: &Did) -> u64 {
            *self.balances.lock().unwrap().get(did).unwrap_or(&0)
        }

        fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            self.balances.lock().unwrap().insert(did.clone(), amount);
            Ok(())
        }

        fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            let balance = balances
                .get_mut(did)
                .ok_or_else(|| CommonError::PolicyDenied("Account not found".into()))?;
            if *balance < amount {
                return Err(CommonError::PolicyDenied("Insufficient mana".into()));
            }
            *balance -= amount;
            Ok(())
        }

        fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            let entry = balances.entry(did.clone()).or_insert(0);
            *entry = entry.saturating_add(amount);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_enhanced_token_class_creation() {
        let resource_ledger = InMemoryResourceLedger::default();
        let issuer = Did::from_str("did:key:issuer123").unwrap();

        // Test creating different token types
        let fungible_token = TokenClass::new_fungible(
            "Test Coin".to_string(),
            "A test fungible token".to_string(),
            "TEST".to_string(),
            2,
            issuer.clone(),
        );

        let time_token = TokenClass::new_time_banking(
            "Community Hours".to_string(),
            "Time banking for community work".to_string(),
            issuer.clone(),
            Some("LocalCommunity".to_string()),
        );

        let mutual_credit = TokenClass::new_mutual_credit(
            "Community Credit".to_string(),
            "Mutual credit system".to_string(),
            "CC".to_string(),
            issuer.clone(),
            "CreditUnion".to_string(),
            10000,
        );

        // Create token classes
        resource_ledger
            .create_class(&"fungible".to_string(), fungible_token.clone())
            .unwrap();
        resource_ledger
            .create_class(&"time".to_string(), time_token.clone())
            .unwrap();
        resource_ledger
            .create_class(&"credit".to_string(), mutual_credit.clone())
            .unwrap();

        // Verify they were created correctly
        let retrieved_fungible = resource_ledger.get_class(&"fungible".to_string()).unwrap();
        assert_eq!(retrieved_fungible.name, "Test Coin");
        assert_eq!(retrieved_fungible.token_type, TokenType::Fungible);
        assert_eq!(
            retrieved_fungible.transferability,
            TransferabilityRule::FreelyTransferable
        );

        let retrieved_time = resource_ledger.get_class(&"time".to_string()).unwrap();
        assert_eq!(retrieved_time.token_type, TokenType::TimeBanking);
        assert_eq!(
            retrieved_time.scoping_rules.community_scope,
            Some("LocalCommunity".to_string())
        );

        let retrieved_credit = resource_ledger.get_class(&"credit".to_string()).unwrap();
        assert_eq!(retrieved_credit.token_type, TokenType::MutualCredit);
        assert_eq!(
            retrieved_credit.metadata.get("credit_limit"),
            Some(&"10000".to_string())
        );

        // Test listing all classes
        let all_classes = resource_ledger.list_classes();
        assert_eq!(all_classes.len(), 3);

        println!("✅ Enhanced token class creation tests passed!");
    }

    #[tokio::test]
    async fn test_transferability_rules() {
        let resource_ledger = InMemoryResourceLedger::default();
        let issuer = Did::from_str("did:key:issuer123").unwrap();
        let alice = Did::from_str("did:key:alice456").unwrap();
        let bob = Did::from_str("did:key:bob789").unwrap();
        let charlie = Did::from_str("did:key:charlie").unwrap();

        // Create a restricted transfer token
        let mut restricted_token = TokenClass::new_fungible(
            "Restricted Token".to_string(),
            "Only certain accounts can receive".to_string(),
            "REST".to_string(),
            0,
            issuer.clone(),
        );
        restricted_token.transferability = TransferabilityRule::RestrictedTransfer {
            authorized_recipients: vec![alice.clone(), bob.clone()],
        };

        // Create a non-transferable token
        let mut non_transferable = TokenClass::new_fungible(
            "Non-Transferable".to_string(),
            "Cannot be transferred".to_string(),
            "NT".to_string(),
            0,
            issuer.clone(),
        );
        non_transferable.transferability = TransferabilityRule::NonTransferable;

        resource_ledger
            .create_class(&"restricted".to_string(), restricted_token)
            .unwrap();
        resource_ledger
            .create_class(&"nontrans".to_string(), non_transferable)
            .unwrap();

        // Test restricted transfers
        assert!(resource_ledger
            .can_transfer(&"restricted".to_string(), &alice, &bob, 100)
            .unwrap());
        assert!(!resource_ledger
            .can_transfer(&"restricted".to_string(), &alice, &charlie, 100)
            .unwrap());

        // Test non-transferable tokens
        assert!(!resource_ledger
            .can_transfer(&"nontrans".to_string(), &alice, &bob, 100)
            .unwrap());

        println!("✅ Transferability rules tests passed!");
    }

    #[tokio::test]
    async fn test_scoped_token_mint_transfer_burn_lifecycle() {
        // Set up test environment
        let resource_ledger = InMemoryResourceLedger::default();
        let mana_ledger = InMemoryManaLedger::default();
        let reputation_store = InMemoryReputationStore::new();

        // Create DAG store for anchoring token events
        let dag_store = Box::new(InMemoryDagStore::new());
        let mut token_repo = ResourceRepositoryAdapter::with_dag_store(resource_ledger, dag_store);

        // Set up test actors
        let issuer = Did::from_str("did:key:issuer123").unwrap();
        let alice = Did::from_str("did:key:alice456").unwrap();
        let bob = Did::from_str("did:key:bob789").unwrap();
        let scope = Some(NodeScope("test_coop".to_string()));

        // Create a token class first
        let coop_shares = TokenClass::new_fungible(
            "Cooperative Shares".to_string(),
            "Shares in the test cooperative".to_string(),
            "COOP".to_string(),
            0,
            issuer.clone(),
        );

        token_repo
            .ledger()
            .create_class(&"coop_shares".to_string(), coop_shares)
            .unwrap();

        // Authorize issuer for this scope
        token_repo.add_issuer(scope.clone().unwrap(), issuer.clone());

        // Set up initial mana balances
        mana_ledger.set_balance(&issuer, 1000).unwrap();
        mana_ledger.set_balance(&alice, 100).unwrap();
        mana_ledger.set_balance(&bob, 100).unwrap();

        // Set up reputation scores
        reputation_store.set_score(issuer.clone(), 100);
        reputation_store.set_score(alice.clone(), 50);
        reputation_store.set_score(bob.clone(), 30);

        let class_id = "coop_shares";

        // Test 1: Mint tokens to Alice
        println!("=== Test 1: Minting tokens ===");
        let mint_result = mint_tokens(
            &token_repo,
            &mana_ledger,
            &issuer,
            class_id,
            50,
            &alice,
            scope.clone(),
        );
        assert!(
            mint_result.is_ok(),
            "Minting should succeed: {:?}",
            mint_result
        );

        // Verify Alice's balance
        let alice_balance = token_repo
            .ledger()
            .get_balance(&class_id.to_string(), &alice);
        assert_eq!(alice_balance, 50, "Alice should have 50 tokens");

        // Verify issuer paid mana fee
        assert_eq!(
            mana_ledger.get_balance(&issuer),
            999,
            "Issuer should have paid 1 mana"
        );

        // Test 2: Transfer tokens from Alice to Bob
        println!("=== Test 2: Transferring tokens ===");
        let transfer_result = transfer_tokens(
            &token_repo,
            &mana_ledger,
            &issuer,
            class_id,
            20,
            &alice,
            &bob,
            scope.clone(),
        );
        assert!(
            transfer_result.is_ok(),
            "Transfer should succeed: {:?}",
            transfer_result
        );

        // Verify balances after transfer
        let alice_balance = token_repo
            .ledger()
            .get_balance(&class_id.to_string(), &alice);
        let bob_balance = token_repo.ledger().get_balance(&class_id.to_string(), &bob);
        assert_eq!(
            alice_balance, 30,
            "Alice should have 30 tokens after transfer"
        );
        assert_eq!(bob_balance, 20, "Bob should have 20 tokens after transfer");

        // Verify issuer paid another mana fee
        assert_eq!(
            mana_ledger.get_balance(&issuer),
            998,
            "Issuer should have paid another 1 mana"
        );

        // Test 3: Burn tokens from Bob
        println!("=== Test 3: Burning tokens ===");
        let burn_result = burn_tokens(
            &token_repo,
            &mana_ledger,
            &issuer,
            class_id,
            10,
            &bob,
            scope.clone(),
        );
        assert!(
            burn_result.is_ok(),
            "Burning should succeed: {:?}",
            burn_result
        );

        // Verify Bob's balance after burning
        let bob_balance = token_repo.ledger().get_balance(&class_id.to_string(), &bob);
        assert_eq!(bob_balance, 10, "Bob should have 10 tokens after burning");

        // Verify issuer paid another mana fee
        assert_eq!(
            mana_ledger.get_balance(&issuer),
            997,
            "Issuer should have paid another 1 mana"
        );

        println!("✅ All scoped token lifecycle tests passed!");
    }

    #[test]
    fn test_token_helper_functions() {
        let issuer = Did::from_str("did:key:issuer123").unwrap();

        // Test creating different specialized token types
        let time_token = TokenClass::new_time_banking(
            "Community Time Bank".to_string(),
            "Hours contributed to community".to_string(),
            issuer.clone(),
            Some("TestCommunity".to_string()),
        );

        assert_eq!(time_token.token_type, TokenType::TimeBanking);
        assert_eq!(time_token.symbol, "TIME");
        assert_eq!(time_token.decimals, 2);
        assert_eq!(time_token.metadata.get("unit"), Some(&"hours".to_string()));

        let local_currency = TokenClass::new_local_currency(
            "Ithaca Hours".to_string(),
            "Local currency for Ithaca community".to_string(),
            "ITH".to_string(),
            issuer.clone(),
            "Ithaca, NY".to_string(),
        );

        assert_eq!(local_currency.token_type, TokenType::LocalCurrency);
        assert_eq!(
            local_currency.scoping_rules.geographic_scope,
            Some("Ithaca, NY".to_string())
        );

        let bulk_token = TokenClass::new_bulk_purchasing(
            "Bulk Solar Panels".to_string(),
            "Collective purchase of solar panels".to_string(),
            issuer.clone(),
            "Solar Panels 300W".to_string(),
            10,
        );

        assert_eq!(bulk_token.token_type, TokenType::BulkPurchasing);
        assert_eq!(
            bulk_token.transferability,
            TransferabilityRule::NonTransferable
        );
        assert_eq!(
            bulk_token.metadata.get("target_product"),
            Some(&"Solar Panels 300W".to_string())
        );

        println!("✅ Token helper function tests passed!");
    }

    #[test]
    fn test_marketplace_functionality() {
        use crate::{
            InMemoryMarketplaceStore, ItemType, LaborHoursConfig, MarketplaceBid, MarketplaceOffer, PhysicalGoodConfig, ServiceConfig,
            MarketplaceStore, OfferFilter, OfferStatus,
        };

        let marketplace = InMemoryMarketplaceStore::new();
        let seller = Did::from_str("did:key:seller123").unwrap();
        let buyer = Did::from_str("did:key:buyer456").unwrap();

        // Create an offer for physical goods
        let offer = MarketplaceOffer::new_physical_good(
            PhysicalGoodConfig {
                offer_id: "offer_001".to_string(),
                seller: seller.clone(),
                description: "Organic tomatoes from local farm".to_string(),
                category: "vegetables".to_string(),
                condition: "fresh".to_string(),
                quantity: 100, // 100 units
                price_per_unit: 5,   // 5 tokens per unit
                payment_token_class: "local_currency".to_string(),
            }
        );

        // Create the offer
        marketplace.create_offer(offer.clone()).unwrap();

        // Verify offer was created
        let retrieved_offer = marketplace.get_offer("offer_001").unwrap();
        assert_eq!(retrieved_offer.seller, seller);
        assert_eq!(retrieved_offer.quantity, 100);
        assert_eq!(retrieved_offer.price_per_unit, 5);

        // Create a bid
        let bid = MarketplaceBid::new_bid(
            "bid_001".to_string(),
            buyer.clone(),
            "offer_001".to_string(),
            20, // Want 20 units
            5,  // Willing to pay 5 tokens per unit
            "local_currency".to_string(),
            24, // Expires in 24 hours
        );

        marketplace.create_bid(bid.clone()).unwrap();

        // Verify bid was created
        let retrieved_bid = marketplace.get_bid("bid_001").unwrap();
        assert_eq!(retrieved_bid.buyer, buyer);
        assert_eq!(retrieved_bid.quantity, 20);
        assert_eq!(retrieved_bid.price_per_unit, 5);

        // List bids for the offer
        let bids_for_offer = marketplace.list_bids_for_offer("offer_001");
        assert_eq!(bids_for_offer.len(), 1);
        assert_eq!(bids_for_offer[0].bid_id, "bid_001");

        // Test filtering offers
        let filter = OfferFilter {
            seller: Some(seller.clone()),
            status: Some(OfferStatus::Active),
            ..Default::default()
        };
        let filtered_offers = marketplace.list_offers(filter);
        assert_eq!(filtered_offers.len(), 1);
        assert_eq!(filtered_offers[0].offer_id, "offer_001");

        println!("✅ Marketplace functionality tests passed!");
    }

    #[test]
    fn test_marketplace_item_types() {
        use crate::{ItemType, MarketplaceOffer};

        let seller = Did::from_str("did:key:seller123").unwrap();

        // Test service offer
        let service_offer = MarketplaceOffer::new_service(
            ServiceConfig {
                offer_id: "service_001".to_string(),
                seller: seller.clone(),
                description: "Web development services".to_string(),
                category: "software_development".to_string(),
                duration_hours: 40, // 40 hours available
                quantity: 1,
                price_per_unit: 50, // 50 tokens per hour
                payment_token_class: "time_banking".to_string(),
            }
        );

        if let ItemType::Service {
            service_type,
            duration,
        } = &service_offer.item_type
        {
            assert_eq!(service_type, "software_development");
            assert_eq!(duration, &Some("per_hour".to_string()));
        } else {
            panic!("Expected Service item type");
        }

        // Test labor hours offer
        let labor_offer = MarketplaceOffer::new_labor_hours(
            LaborHoursConfig {
                offer_id: "labor_001".to_string(),
                seller: seller.clone(),
                description: "Carpentry work".to_string(),
                skill_level: "experienced".to_string(),
                duration_hours: 80, // 80 hours
                quantity: 1,
                price_per_unit: 25, // 25 tokens per hour
                payment_token_class: "time_banking".to_string(),
            }
        );

        if let ItemType::LaborHours {
            skill_type,
            experience_level,
        } = &labor_offer.item_type
        {
            assert_eq!(skill_type, "carpentry");
            assert_eq!(experience_level, "experienced");
        } else {
            panic!("Expected LaborHours item type");
        }

        println!("✅ Marketplace item type tests passed!");
    }
}
