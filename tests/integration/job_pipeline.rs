// tests/integration/job_pipeline.rs

use std::{sync::Arc, time::Duration};

use icn_identity::{generate_ed25519_keypair, did_key_from_verifying_key}; // Adjusted to actual function names
use icn_runtime::{
    context::{RuntimeContext, StubSigner, StubMeshNetworkService, RuntimeStubDagStore}, // Corrected: RuntimeStubDagStore
    host_submit_mesh_job,
};
use icn_mesh::{JobSpec, JobState}; // Added JobState
use serde_json::json;
use tokio::time::sleep;
use tokio::sync::Mutex;
use icn_common::Did; // Added Did
use std::str::FromStr; // For Did::from_str

#[tokio::test]
async fn end_to_end_mesh_job_execution() {
    // 1. Create test identity and runtime context
    let (sk, pk) = generate_ed25519_keypair();
    let did_string = did_key_from_verifying_key(&pk);
    let did = Did::from_str(&did_string).expect("Failed to parse DID string"); // Create Did struct

    let signer = StubSigner::new_with_keys(sk, pk);
    let net = Arc::new(StubMeshNetworkService::default()); // Assumes Default trait
    let dag = Arc::new(tokio::sync::Mutex::new(RuntimeStubDagStore::default()));   // Assumes Default trait
    
    // Using a simplified constructor or assuming one exists that takes these components.
    // The user provided `new_for_test` which I will add.
    let ctx = Arc::new(RuntimeContext::new_for_test(did.clone(), signer, net.clone(), dag.clone()));
    ctx.spawn_mesh_job_manager().await;

    // 2. Submit a basic Echo job
    let job_spec = JobSpec::Echo {
        payload: "hello world".to_owned(),
        input_cids: Vec::new(),
        output_cids: Vec::new(),
        required_resources: Resources::default(),
    }; // Assumes JobSpec::Echo variant
    let manifest_cid = "bafybeigdyrzt7dpbrm3kmhgtr5mk6yzqq3wj7owxsbs2hlkzbfio4ilv5e"; // any CID string
    
    // Constructing the JSON payload for ActualMeshJob as expected by host_submit_mesh_job
    let submit_json = json!({
        "manifest_cid": manifest_cid,
        "spec": job_spec, // This needs to match how ActualMeshJob serializes its spec
        "cost_mana": 1,
        // id, creator_did, signature are handled by host_submit_mesh_job or prior steps
    });

    let submit_res = host_submit_mesh_job(&ctx, &submit_json.to_string()).await; // Pass Arc as reference
    assert!(submit_res.is_ok(), "Job submission failed: {submit_res:?}");
    let submitted_job_id = submit_res.unwrap();

    // 3. Allow job manager loop to process the pipeline
    // Increased sleep to give more time for the full pipeline including potential receipt handling.
    sleep(Duration::from_secs(5)).await; 

    // 4. Check for anchored receipt in stub DAG
    let all_dag_items = dag.lock().await.all();
    
    // We need to find the specific receipt for our job_id or verify its presence.
    // A generic check for !all_dag_items.is_empty() is a good start.
    // For a more robust check, we'd ideally parse these items as receipts and find ours.
    assert!(!all_dag_items.is_empty(), "DAG should have at least one anchored receipt");
    
    // Let's try to find our specific receipt (this is a more advanced check)
    let mut found_receipt_for_job = false;
    for (_cid, item_bytes) in all_dag_items {
        if let Ok(receipt) = serde_json::from_slice::<icn_identity::ExecutionReceipt>(&item_bytes) {
            if receipt.job_id == submitted_job_id {
                found_receipt_for_job = true;
                println!("Found receipt for job_id {}: {:?}", submitted_job_id, receipt);
                break;
            }
        }
    }
    assert!(found_receipt_for_job, "Receipt for submitted job ID {} not found in DAG", submitted_job_id);


    // 5. Confirm job state is Completed
    let job_states_map = ctx.job_states.lock().await; // Access job_states directly
    
    let final_job_state = job_states_map.get(&submitted_job_id);
    assert!(final_job_state.is_some(), "Job ID {} not found in job_states", submitted_job_id);

    match final_job_state.unwrap() {
        JobState::Completed { receipt } => {
            println!("Job {} reached Completed state with receipt: {:?}", submitted_job_id, receipt);
            assert_eq!(receipt.job_id, submitted_job_id);
        }
        other_state => {
            panic!("Job {} is in state {:?}, expected Completed", submitted_job_id, other_state);
        }
    }
    
    // An alternative check as in the original prompt, assuming JobState has is_completed()
    // assert!(
    //     job_states_map.values().any(|s| s.is_completed()),
    //     "No job reached Completed state in job_states map"
    // );

    println!("End-to-end mesh job pipeline test succeeded for job {}.", submitted_job_id);
} 