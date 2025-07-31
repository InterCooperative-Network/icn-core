#[cfg(feature = "enable-libp2p")]
mod simple_verification_test {

    use icn_common::{Cid, DagBlock, Did};
    use icn_mesh::{JobKind, JobSpec, Resources};
    use icn_runtime::RuntimeContext;
    use std::time::Duration;
    use tokio::time::sleep; // Moved this line to keep necessary imports

    #[tokio::test]
    async fn simple_icn_core_verification() {
        println!("🚀 ICN Core Simple Verification Test");

        // Test 1: Create runtime context
        println!("📋 Test 1: Creating RuntimeContext...");
        let test_did = Did::new("key", "test_node");
        let rt_ctx = RuntimeContext::new_testing(test_did.clone(), Some(1000))
            .expect("Failed to create test RuntimeContext");

        // Test 2: Verify mana management
        println!("📋 Test 2: Testing mana management...");
        let initial_balance = rt_ctx
            .get_mana(&test_did)
            .await
            .expect("Failed to get initial mana balance");
        assert_eq!(initial_balance, 1000, "Initial mana balance should be 1000");

        // Spend some mana
        rt_ctx
            .spend_mana(&test_did, 100)
            .await
            .expect("Failed to spend mana");

        let balance_after_spend = rt_ctx
            .get_mana(&test_did)
            .await
            .expect("Failed to get balance after spend");
        assert_eq!(
            balance_after_spend, 900,
            "Balance should be 900 after spending 100"
        );

        // Credit some mana back
        rt_ctx
            .credit_mana(&test_did, 50)
            .await
            .expect("Failed to credit mana");

        let final_balance = rt_ctx
            .get_mana(&test_did)
            .await
            .expect("Failed to get final balance");
        assert_eq!(final_balance, 950, "Final balance should be 950");

        // Test 3: Verify DAG store
        println!("📋 Test 3: Testing DAG store...");
        let test_data = b"Hello ICN World!";
        let test_cid = Cid::new_v1_sha256(0x55, test_data);

        let dag_block = DagBlock {
            cid: test_cid.clone(),
            data: test_data.to_vec(),
            links: vec![],
            timestamp: 0,
            author_did: test_did.clone(),
            signature: None,
            scope: None,
        };

        let mut dag_store = rt_ctx.dag_store.store.lock().await;
        let put_result = dag_store.put(&dag_block).await;
        assert!(put_result.is_ok(), "DAG put should succeed");

        let get_result = dag_store.get(&test_cid).await;
        assert!(get_result.is_ok(), "DAG get should succeed");

        if let Ok(Some(retrieved_block)) = get_result {
            assert_eq!(
                retrieved_block.data, test_data,
                "Retrieved data should match stored data"
            );
        }
        drop(dag_store);

        // Test 4: Submit a simple mesh job using the new API
        println!("📋 Test 4: Testing mesh job submission...");
        let manifest_cid = Cid::new_v1_sha256(0x55, b"test_manifest");
        let spec = icn_mesh::JobSpec {
            kind: icn_mesh::JobKind::Echo {
                payload: "Simple verification test job".into(),
            },
            inputs: vec![],
            outputs: vec!["result".into()],
            required_resources: icn_mesh::Resources {
                cpu_cores: 1,
                memory_mb: 128,
                storage_mb: 0,
            },
            required_capabilities: vec![],
            required_trust_scope: None,
            min_executor_reputation: None,
            allowed_federations: vec![],
        };
        let spec_bytes = bincode::serialize(&spec).unwrap();
        let submit_result = rt_ctx
            .handle_submit_job(manifest_cid, spec_bytes, 100)
            .await;
        assert!(submit_result.is_ok(), "Job submission should succeed");

        if let Ok(job_id) = submit_result {
            println!("✅ Job submitted with ID: {}", job_id);

            // Wait a bit and check job status
            sleep(Duration::from_secs(1)).await;

            let job_status = rt_ctx.get_job_status(&job_id).await;
            assert!(job_status.is_ok(), "Should be able to get job status");

            if let Ok(status) = job_status {
                println!("📊 Job status: {:?}", status);
            }
        }

        println!("✅ ICN Core Simple Verification Complete!");
        println!("🎉 All core functionality verified:");
        println!("   ✓ RuntimeContext creation");
        println!("   ✓ Mana management (spend/credit/balance)");
        println!("   ✓ DAG store (put/get)");
        println!("   ✓ Mesh job submission");
    }
}

#[cfg(not(feature = "enable-libp2p"))]
#[tokio::test]
async fn simple_verification_stub() {
    println!("❌ Simple verification test requires the 'enable-libp2p' feature.");
    println!("Run with: cargo test --features enable-libp2p");
}
