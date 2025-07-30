use icn_ccl::{ast::StatementNode, compile_ccl_source_to_wasm, parser::parse_ccl_source, CclError};
use icn_common::{Cid, DagBlock};
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
use icn_runtime::context::{RuntimeContext, StubSigner};
use icn_runtime::executor::{JobExecutor, WasmExecutor, WasmExecutorConfig};
use std::str::FromStr;
use std::sync::Arc;

use tokio::runtime::Runtime;

fn ctx_with_temp_store(did: &str, mana: u64) -> Arc<RuntimeContext> {
    let ctx = RuntimeContext::new_with_stubs_and_mana(did, mana).unwrap();
    ctx
}

#[test]
fn parse_simple_for_loop() {
    let src = "fn test() -> Integer { let sum = 0; for item in [1, 2, 3] { sum = sum + item; } return sum; }";
    let ast = parse_ccl_source(src).expect("parse");
    if let icn_ccl::ast::AstNode::Policy(items) = ast {
        if let icn_ccl::ast::PolicyStatementNode::FunctionDef(
            icn_ccl::ast::AstNode::FunctionDefinition { body, .. },
        ) = &items[0]
        {
            match &body.statements[1] {
                StatementNode::ForLoop { .. } => {
                    // success
                }
                _ => panic!("expected for statement"),
            }
        } else {
            panic!("unexpected ast");
        }
    } else {
        panic!("unexpected root");
    }
}

#[test]
fn parse_for_loop_with_array_variable() {
    let src = "fn test() -> Integer { let numbers = [10, 20, 30]; let total = 0; for value in numbers { total = total + value; } return total; }";
    let ast = parse_ccl_source(src).expect("parse");
    // Basic parsing validation - detailed AST structure validation omitted for brevity
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_nested_for_loops() {
    let src = "fn test() -> Integer { let total = 0; for outer in [1, 2, 3] { for inner in [10, 20] { total = total + outer + inner; } } return total; }";
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_for_loop_with_conditional_logic() {
    let src = r#"
        fn test() -> Integer {
            let processed = 0;
            for score in [85, 90, 75, 95, 80] {
                if score >= 80 {
                    if score >= 90 {
                        processed = processed + 2;
                    } else {
                        processed = processed + 1;
                    }
                }
            }
            return processed;
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

// Test that for loops fail compilation (expected since WASM backend not implemented)
#[test]
fn for_loop_compilation_not_yet_supported() {
    let src = r#"
        fn run() -> Integer {
            let sum = 0;
            for num in [1, 2, 3, 4, 5] {
                sum = sum + num;
            }
            return sum;
        }
    "#;
    let result = compile_ccl_source_to_wasm(src);
    match result {
        Err(CclError::WasmGenerationError(msg)) => {
            assert!(msg.contains("For loops not yet supported"));
        }
        _ => panic!("Expected WasmGenerationError for for loops"),
    }
}

#[test]
fn parse_for_loop_in_governance_context() {
    let src = r#"
        fn calculate_voting_power(reputation_scores: [Integer; 5]) -> Integer {
            let total_power = 0;
            for reputation in reputation_scores {
                let base_power = 10;
                let bonus_power = 0;
                
                if reputation >= 75 {
                    bonus_power = 20;
                } else if reputation >= 50 {
                    bonus_power = 10;
                }
                
                total_power = total_power + base_power + bonus_power;
            }
            return total_power;
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_for_loop_with_mana_distribution() {
    let src = r#"
        fn distribute_mana_equally(member_ids: [Integer; 4], total_mana: Mana) -> Mana {
            let distributed = 0;
            let per_member = total_mana / 4;
            
            for member_id in member_ids {
                let allocation = per_member;
                
                // Special allocation for founding members (id < 10)
                if member_id < 10 {
                    allocation = allocation + (allocation / 20);
                }
                
                distributed = distributed + allocation;
            }
            
            return distributed;
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_for_loop_boolean_array() {
    let src = r#"
        fn count_approvals(votes: [Bool; 3]) -> Integer {
            let approvals = 0;
            for vote in votes {
                if vote {
                    approvals = approvals + 1;
                }
            }
            return approvals;
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[tokio::test(flavor = "multi_thread")]
async fn for_loop_sum_three_elements() {
    let src = r#"
        fn run() -> Integer {
            let nums = [1, 2, 3];
            let total = 0;
            for n in nums { total = total + n; }
            return total;
        }
    "#;
    let (wasm, _) = compile_ccl_source_to_wasm(src).expect("compile");
    let ctx = ctx_with_temp_store("did:key:zFor3", 10);
    let ts = 0u64;
    let author = icn_common::Did::new("key", "tester");
    let sig_opt = None;
    let cid = icn_common::compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
    let block = DagBlock {
        cid: cid.clone(),
        data: wasm.clone(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
        scope: None,
    };
    {
        let mut store = ctx.dag_store.store.lock().await;
        store.put(&block).await.unwrap();
    }
    let cid = block.cid.clone();
    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"for3")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let signer = Arc::new(StubSigner::new_with_keys(sk, vk));
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
    let job_clone = job.clone();
    let handle = std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &6i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}

#[tokio::test(flavor = "multi_thread")]
async fn for_loop_sum_five_elements() {
    let src = r#"
        fn run() -> Integer {
            let nums = [1, 2, 3, 4, 5];
            let total = 0;
            for n in nums { total = total + n; }
            return total;
        }
    "#;
    let (wasm, _) = compile_ccl_source_to_wasm(src).expect("compile");
    let ctx = ctx_with_temp_store("did:key:zFor5", 10);
    let ts = 0u64;
    let author = icn_common::Did::new("key", "tester");
    let sig_opt = None;
    let cid = icn_common::compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
    let block = DagBlock {
        cid: cid.clone(),
        data: wasm.clone(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
        scope: None,
    };
    {
        let mut store = ctx.dag_store.store.lock().await;
        store.put(&block).await.unwrap();
    }
    let cid = block.cid.clone();
    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"for5")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let signer = Arc::new(StubSigner::new_with_keys(sk, vk));
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
    let job_clone = job.clone();
    let handle = std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &15i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}
