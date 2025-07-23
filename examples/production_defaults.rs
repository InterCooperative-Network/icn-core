//! Example demonstrating the new production-by-default RuntimeContext API
//!
//! This example shows how to use the updated RuntimeContext constructors
//! with clear separation between production and testing configurations.

use icn_common::Did;
use icn_runtime::context::RuntimeContext;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🚀 ICN Runtime Context Examples");
    println!("================================");

    // Example 1: Testing Context (explicit and safe)
    println!("\n🧪 Creating Testing Context");
    let test_did = Did::from_str("did:key:zTestExample")?;
    let test_ctx = RuntimeContext::new_for_testing(test_did, Some(1000))?;
    println!("✅ Testing context created with 1000 initial mana");
    
    // Verify it uses stub services (production validation should fail)
    match test_ctx.validate_production_services() {
        Err(e) => println!("✅ Production validation correctly rejected stub services: {}", e),
        Ok(_) => println!("❌ Production validation should have failed!"),
    }

    // Example 2: Production Context (requires explicit services or libp2p feature)
    println!("\n🏭 Attempting Production Context");
    match RuntimeContext::new() {
        Ok(_) => println!("✅ Production context created successfully"),
        Err(e) => {
            println!("ℹ️  Production context requires explicit configuration: {}", e);
            
            #[cfg(feature = "enable-libp2p")]
            {
                println!("💡 Try using RuntimeContext::new_async() for full libp2p support");
            }
            
            #[cfg(not(feature = "enable-libp2p"))]
            {
                println!("💡 Enable the 'enable-libp2p' feature for automatic production setup");
            }
        }
    }

    // Example 3: Async Production Context (with libp2p feature)
    #[cfg(feature = "enable-libp2p")]
    {
        println!("\n🌐 Creating Async Production Context");
        match RuntimeContext::new_async().await {
            Ok(prod_ctx) => {
                println!("✅ Async production context created with real libp2p networking");
                
                // Validate that it passes production checks
                match prod_ctx.validate_production_services() {
                    Ok(_) => println!("✅ Production validation passed"),
                    Err(e) => println!("❌ Production validation failed: {}", e),
                }
            }
            Err(e) => println!("❌ Failed to create async production context: {}", e),
        }
    }

    // Example 4: Deprecated Methods (still work but with warnings)
    println!("\n⚠️  Using Deprecated Methods");
    #[allow(deprecated)]
    let legacy_ctx = RuntimeContext::new_with_stubs("did:key:zLegacyTest")?;
    println!("✅ Deprecated method still works (forwards to new_for_testing)");

    println!("\n🎉 All examples completed successfully!");
    println!("\n📖 Key Takeaways:");
    println!("   • Use RuntimeContext::new_for_testing() for tests (explicit and clear)");
    println!("   • Use RuntimeContext::new() or new_async() for production (safe defaults)");  
    println!("   • Production contexts automatically validate service configuration");
    println!("   • Deprecated methods still work but forward to new explicit methods");

    Ok(())
}