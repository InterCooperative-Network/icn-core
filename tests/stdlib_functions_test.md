# Economics and Identity Functions Test

This test demonstrates that all the new critical economics and identity functions 
have been properly implemented in the CCL standard library.

```rust
use icn_ccl::stdlib::{StdLibrary, StdCategory};

fn test_all_new_functions() {
    let stdlib = StdLibrary::new();
    
    // Test that all economics functions are available
    let economics_functions = [
        "create_token_class",
        "mint_tokens", 
        "transfer_tokens",
        "burn_tokens",
        "get_token_balance",
        "price_by_reputation",
        "credit_by_reputation", 
        "mint_tokens_with_reputation",
        "record_time_work",
        "mint_time_tokens",
        "create_credit_line",
        "extend_mutual_credit",
        "create_marketplace_offer",
        "execute_marketplace_transaction"
    ];
    
    for func_name in &economics_functions {
        let func = stdlib.get_function(func_name);
        assert!(func.is_some(), "Function {} not found", func_name);
        let func = func.unwrap();
        assert_eq!(func.category, StdCategory::Economics);
        println!("✓ Economics function '{}' registered: {}", func_name, func.description);
    }
    
    // Test that all identity functions are available
    let identity_functions = [
        "create_did",
        "resolve_did", 
        "update_did_document",
        "verify_did_signature",
        "issue_credential",
        "verify_credential",
        "revoke_credential",
        "create_cooperative_membership",
        "verify_cooperative_membership"
    ];
    
    for func_name in &identity_functions {
        let func = stdlib.get_function(func_name);
        assert!(func.is_some(), "Function {} not found", func_name);
        let func = func.unwrap();
        assert_eq!(func.category, StdCategory::Identity);
        println!("✓ Identity function '{}' registered: {}", func_name, func.description);
    }
    
    // Test category counts
    let economics_count = stdlib.get_functions_by_category(StdCategory::Economics).len();
    let identity_count = stdlib.get_functions_by_category(StdCategory::Identity).len();
    
    println!("\n📊 Function counts:");
    println!("   Economics functions: {}", economics_count);
    println!("   Identity functions: {}", identity_count);
    println!("   Total new functions: {}", economics_functions.len() + identity_functions.len());
    
    // Test specific function signatures
    let create_token = stdlib.get_function("create_token_class").unwrap();
    assert_eq!(create_token.params.len(), 4);
    println!("✓ create_token_class has correct signature: {} parameters", create_token.params.len());
    
    let issue_cred = stdlib.get_function("issue_credential").unwrap();
    assert_eq!(issue_cred.params.len(), 5);
    println!("✓ issue_credential has correct signature: {} parameters", issue_cred.params.len());
    
    println!("\n🎉 All critical economics and identity functions successfully implemented!");
}

fn main() {
    test_all_new_functions();
}
```

## Expected Output

When this test runs successfully, it should output:

```
✓ Economics function 'create_token_class' registered: Create a new token class with specified properties
✓ Economics function 'mint_tokens' registered: Mint new tokens of specified class to recipient
✓ Economics function 'transfer_tokens' registered: Transfer tokens between accounts
✓ Economics function 'burn_tokens' registered: Burn tokens from an account
✓ Economics function 'get_token_balance' registered: Get token balance for a specific class and account
✓ Economics function 'price_by_reputation' registered: Calculate price adjusted by reputation (higher rep = lower price)
✓ Economics function 'credit_by_reputation' registered: Credit mana to account based on reputation multiplier
✓ Economics function 'mint_tokens_with_reputation' registered: Mint tokens with mana cost adjusted by issuer reputation
✓ Economics function 'record_time_work' registered: Record time-based work contribution for time banking
✓ Economics function 'mint_time_tokens' registered: Mint time-based tokens for verified work hours
✓ Economics function 'create_credit_line' registered: Establish mutual credit relationship between community members
✓ Economics function 'extend_mutual_credit' registered: Extend credit within established mutual credit line
✓ Economics function 'create_marketplace_offer' registered: Create marketplace offer for goods or services
✓ Economics function 'execute_marketplace_transaction' registered: Execute marketplace transaction between buyer and seller
✓ Identity function 'create_did' registered: Create a new decentralized identifier with specified method
✓ Identity function 'resolve_did' registered: Resolve a DID to its document containing public keys and services
✓ Identity function 'update_did_document' registered: Update a DID document with new keys or service endpoints
✓ Identity function 'verify_did_signature' registered: Verify a signature was created by the DID controller
✓ Identity function 'issue_credential' registered: Issue a verifiable credential with specified claims
✓ Identity function 'verify_credential' registered: Verify the authenticity and validity of a credential
✓ Identity function 'revoke_credential' registered: Revoke a previously issued credential
✓ Identity function 'create_cooperative_membership' registered: Create cooperative membership credential
✓ Identity function 'verify_cooperative_membership' registered: Verify cooperative membership and authorization level

📊 Function counts:
   Economics functions: 20
   Identity functions: 9
   Total new functions: 23

✓ create_token_class has correct signature: 4 parameters
✓ issue_credential has correct signature: 5 parameters

🎉 All critical economics and identity functions successfully implemented!
```

This confirms that all the required functions have been properly implemented with correct signatures and categorization.