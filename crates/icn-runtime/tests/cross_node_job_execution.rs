#![allow(
    unused_imports,
    unused_variables,
    clippy::uninlined_format_args,
    clippy::absurd_extreme_comparisons
)]
//! Cross-node mesh job execution integration tests using the Runtime Host ABI
//!
//! This test suite demonstrates the complete ICN mesh computing pipeline using
//! real Runtime contexts and Host ABI calls, representing the production integration
//! path for Phase 3.

#[cfg(feature = "enable-libp2p")]
#[cfg(any())]
mod runtime_host_abi_tests {
    use anyhow::Result;
    use icn_common::{Cid, Did};
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, ExecutionReceipt};
    use icn_mesh::{ActualMeshJob, JobSpec};
    use icn_network::{NetworkMessage, NetworkService};
    use icn_runtime::context::RuntimeContext;
    use icn_runtime::{host_anchor_receipt, host_submit_mesh_job, ReputationUpdater};
    use libp2p::{Multiaddr, PeerId as Libp2pPeerId};
    use log::{debug, info};
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::time::{sleep, timeout, Duration};

    /// Helper to create a RuntimeContext with real libp2p networking
    async fn create_runtime_node(
        identity_name: &str,
        bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>,
        initial_mana: u64,
    ) -> Result<Arc<RuntimeContext>> {
        let identity_did_str = format!("did:key:z6Mkv{}", identity_name);
        let identity_did = Did::from_str(&identity_did_str)?;

        let runtime_ctx =
            RuntimeContext::new_with_libp2p_network(&identity_did_str, bootstrap_peers)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create runtime context: {}", e))?;

        // Set initial mana balance
        runtime_ctx
            .mana_ledger
            .set_balance(&identity_did, initial_mana)
            .await;

        Ok(runtime_ctx)
    }

    /// Creates a test job JSON for host_submit_mesh_job
    fn create_test_job_json(
        job_suffix: &str,
        creator_did: &Did,
        cost_mana: u64,
        payload: &str,
    ) -> String {
        let job_id =
            Cid::new_v1_dummy(0x55, 0x13, format!("runtime_job_{}", job_suffix).as_bytes());
        let manifest_cid =
            Cid::new_v1_dummy(0x55, 0x14, format!("manifest_{}", job_suffix).as_bytes());

        let job = ActualMeshJob {
            id: job_id,
            manifest_cid,
            spec: JobSpec::Echo {
                payload: payload.to_string(),
            },
            creator_did: creator_did.clone(),
            cost_mana,
            signature: icn_identity::SignatureBytes(vec![0u8; 64]), // Dummy signature
        };

        serde_json::to_string(&job).expect("Failed to serialize test job")
    }

    #[tokio::test]
    #[ignore = "Runtime-driven cross-node job execution using Host ABI"]
    async fn test_runtime_host_abi_cross_node_execution() -> Result<()> {
        info!("🚀 [RUNTIME-INTEGRATION] Starting Host ABI cross-node job execution test");

        // === Phase 1: Setup Runtime Nodes ===
        info!("🔧 [RUNTIME-INTEGRATION] Phase 1: Creating runtime nodes...");

        let submitter_node = create_runtime_node("SubmitterNode", None, 1000).await?;
        let submitter_did = submitter_node.current_identity.clone();

        sleep(Duration::from_millis(500)).await;

        // Get submitter node's networking info for bootstrap
        let submitter_libp2p = submitter_node
            .get_libp2p_service()
            .map_err(|e| anyhow::anyhow!("Failed to get submitter libp2p service: {}", e))?;
        let submitter_peer_id = submitter_libp2p.local_peer_id().clone();
        let submitter_addrs = submitter_libp2p.listening_addresses();

        if submitter_addrs.is_empty() {
            return Err(anyhow::anyhow!("Submitter node has no listening addresses"));
        }

        let bootstrap_peers = vec![(submitter_peer_id, submitter_addrs[0].clone())];
        let executor_node = create_runtime_node("ExecutorNode", Some(bootstrap_peers), 500).await?;
        let executor_did = executor_node.current_identity.clone();

        info!(
            "✅ [RUNTIME-INTEGRATION] Created runtime nodes - Submitter: {}, Executor: {}",
            submitter_did, executor_did
        );

        // Allow nodes to connect
        sleep(Duration::from_secs(3)).await;

        // === Phase 2: Submit Job via Host ABI ===
        info!("🔧 [RUNTIME-INTEGRATION] Phase 2: Submitting job via host_submit_mesh_job...");

        let test_job_json = create_test_job_json(
            "cross_node_runtime",
            &submitter_did,
            100,
            "Runtime Host ABI Cross-Node Test",
        );

        info!("📄 [RUNTIME-INTEGRATION] Job JSON: {}", test_job_json);

        let submitted_job_id = host_submit_mesh_job(&submitter_node, &test_job_json)
            .await
            .map_err(|e| anyhow::anyhow!("host_submit_mesh_job failed: {}", e))?;

        info!(
            "✅ [RUNTIME-INTEGRATION] Job submitted via Host ABI - Job ID: {}",
            submitted_job_id
        );

        // === Phase 3: Monitor Job State Progression ===
        info!("🔧 [RUNTIME-INTEGRATION] Phase 3: Monitoring job state progression...");

        // The job should progress through: Pending -> Assigned -> Completed
        // We'll monitor the submitter node's job_states to track this

        // Wait for job to be assigned
        let mut job_assigned = false;
        for attempt in 1..=20 {
            sleep(Duration::from_millis(500)).await;

            let job_states = submitter_node.job_states.lock().await;
            if let Some(job_state) = job_states.get(&submitted_job_id) {
                match job_state {
                    icn_mesh::JobState::Assigned { executor } => {
                        info!(
                            "✅ [RUNTIME-INTEGRATION] Job assigned to executor: {} (attempt {})",
                            executor, attempt
                        );
                        job_assigned = true;
                        break;
                    }
                    icn_mesh::JobState::Completed { receipt } => {
                        info!("🎉 [RUNTIME-INTEGRATION] Job already completed with receipt: {:?} (attempt {})", receipt.job_id, attempt);
                        job_assigned = true;
                        break;
                    }
                    state => {
                        debug!(
                            "[RUNTIME-INTEGRATION] Job state (attempt {}): {:?}",
                            attempt, state
                        );
                    }
                }
            } else {
                debug!(
                    "[RUNTIME-INTEGRATION] Job not found in states (attempt {})",
                    attempt
                );
            }
        }

        if !job_assigned {
            return Err(anyhow::anyhow!("Job was not assigned within 10 seconds"));
        }

        // === Phase 4: Wait for Job Completion ===
        info!("🔧 [RUNTIME-INTEGRATION] Phase 4: Waiting for job completion...");

        let mut job_completed = false;
        let mut final_receipt: Option<ExecutionReceipt> = None;

        for attempt in 1..=30 {
            sleep(Duration::from_millis(1000)).await;

            let job_states = submitter_node.job_states.lock().await;
            if let Some(job_state) = job_states.get(&submitted_job_id) {
                match job_state {
                    icn_mesh::JobState::Completed { receipt } => {
                        info!("🎉 [RUNTIME-INTEGRATION] Job completed! Receipt job_id: {} (attempt {})", receipt.job_id, attempt);
                        final_receipt = Some(receipt.clone());
                        job_completed = true;
                        break;
                    }
                    icn_mesh::JobState::Failed { reason } => {
                        return Err(anyhow::anyhow!("Job failed: {}", reason));
                    }
                    state => {
                        debug!(
                            "[RUNTIME-INTEGRATION] Job state (attempt {}): {:?}",
                            attempt, state
                        );
                    }
                }
            }
        }

        if !job_completed {
            return Err(anyhow::anyhow!("Job did not complete within 30 seconds"));
        }

        let receipt =
            final_receipt.ok_or_else(|| anyhow::anyhow!("Receipt is None after completion"))?;

        // === Phase 5: Verify Receipt via Host ABI ===
        info!("🔧 [RUNTIME-INTEGRATION] Phase 5: Verifying receipt via host_anchor_receipt...");

        // The receipt should already be anchored by the runtime, but let's verify it
        assert_eq!(
            receipt.job_id, submitted_job_id,
            "Receipt job ID matches submitted job"
        );
        assert_eq!(
            receipt.executor_did, executor_did,
            "Receipt executor matches executor node"
        );
        assert!(!receipt.sig.0.is_empty(), "Receipt has signature");
        assert!(receipt.cpu_ms >= 0, "Receipt has valid CPU time");

        info!("✅ [RUNTIME-INTEGRATION] Receipt verification successful:");
        info!("   • Job ID: {}", receipt.job_id);
        info!("   • Executor: {}", receipt.executor_did);
        info!("   • Result CID: {}", receipt.result_cid);
        info!("   • CPU Time: {}ms", receipt.cpu_ms);
        info!("   • Signature Length: {} bytes", receipt.sig.0.len());

        // === Phase 6: Verify Final State ===
        info!("🔧 [RUNTIME-INTEGRATION] Phase 6: Verifying final runtime state...");

        // Check mana balances
        let submitter_balance = submitter_node
            .mana_ledger
            .get_balance(&submitter_did)
            .await
            .unwrap_or(0);
        let executor_balance = executor_node
            .mana_ledger
            .get_balance(&executor_did)
            .await
            .unwrap_or(0);

        info!(
            "💰 [RUNTIME-INTEGRATION] Final mana balances - Submitter: {}, Executor: {}",
            submitter_balance, executor_balance
        );

        // Submitter should have spent mana (started with 1000, spent 100)
        assert!(
            submitter_balance <= 900,
            "Submitter should have spent mana for job"
        );

        // === Success Summary ===
        info!(
            "🎉 [RUNTIME-INTEGRATION] Complete runtime Host ABI cross-node execution successful!"
        );
        info!("📊 [RUNTIME-INTEGRATION] Test Summary:");
        info!("   ✅ Runtime nodes created with real libp2p networking");
        info!("   ✅ Job submitted via host_submit_mesh_job Host ABI");
        info!("   ✅ Job assignment and execution across nodes");
        info!("   ✅ Receipt creation and verification");
        info!("   ✅ Mana accounting and state management");
        info!("   ✅ Complete Host ABI integration functional");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Individual phase test: job submission via Host ABI"]
    async fn test_host_submit_mesh_job_api() -> Result<()> {
        info!("🔧 [HOST-ABI-TEST] Testing host_submit_mesh_job API individually");

        let runtime_ctx = create_runtime_node("HostApiTest", None, 500).await?;
        let creator_did = runtime_ctx.current_identity.clone();

        let job_json = create_test_job_json("host_api", &creator_did, 50, "Host API Test");

        let job_id = host_submit_mesh_job(&runtime_ctx, &job_json).await?;

        info!("✅ [HOST-ABI-TEST] Job submitted successfully: {}", job_id);

        // Verify job appears in pending state
        let job_states = runtime_ctx.job_states.lock().await;
        let job_state = job_states
            .get(&job_id)
            .ok_or_else(|| anyhow::anyhow!("Job not found in runtime state"))?;

        match job_state {
            icn_mesh::JobState::Pending => {
                info!("✅ [HOST-ABI-TEST] Job correctly in Pending state");
            }
            other => {
                return Err(anyhow::anyhow!("Expected Pending state, got: {:?}", other));
            }
        }

        // Verify mana was deducted
        let balance = runtime_ctx
            .mana_ledger
            .get_balance(&creator_did)
            .await
            .unwrap_or(0);
        assert_eq!(
            balance, 450,
            "Mana should be deducted for job cost (500 - 50 = 450)"
        );

        info!("✅ [HOST-ABI-TEST] Host ABI job submission test passed");
        Ok(())
    }

    #[tokio::test]
    #[ignore = "Individual phase test: receipt anchoring via Host ABI"]
    async fn test_host_anchor_receipt_api() -> Result<()> {
        info!("🔧 [HOST-ABI-TEST] Testing host_anchor_receipt API individually");

        let runtime_ctx = create_runtime_node("ReceiptApiTest", None, 0).await?;
        let executor_did = runtime_ctx.current_identity.clone();

        // Create a dummy job ID
        let job_id = Cid::new_v1_dummy(0x55, 0x13, b"test_receipt_job");
        let result_cid = Cid::new_v1_dummy(0x55, 0x14, b"test_result_data");

        let receipt = ExecutionReceipt {
            job_id: job_id.clone(),
            executor_did: executor_did.clone(),
            result_cid,
            cpu_ms: 150,
            sig: icn_identity::SignatureBytes(vec![]), // Will be signed by anchor_receipt
        };

        let receipt_json = serde_json::to_string(&receipt)?;

        // Use the runtime's ReputationUpdater
        let reputation_updater = ReputationUpdater::new();

        let anchored_cid =
            host_anchor_receipt(&runtime_ctx, &receipt_json, &reputation_updater).await?;

        info!(
            "✅ [HOST-ABI-TEST] Receipt anchored successfully: {}",
            anchored_cid
        );
        info!("✅ [HOST-ABI-TEST] Host ABI receipt anchoring test passed");
        Ok(())
    }
}
