#![allow(clippy::uninlined_format_args)] // Development utility with format strings

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use icn_common::{parse_cid_from_string, Did};
use icn_dag::TokioFileDagStore;
use icn_identity::{generate_ed25519_keypair, ExecutionReceipt};
use icn_mesh::{ActualMeshJob, JobId};
use icn_runtime::context::mana::SimpleManaLedger;
use icn_runtime::context::{DagStoreMutexType, RuntimeContext, StubSigner};
use icn_runtime::executor::{JobExecutor, SimpleExecutor};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Parser)]
#[command(author, version, about = "ICN Job Audit Utility")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Audit a job by JobId
    Audit {
        /// JobId CID string
        job_id: String,
        /// Path to DAG directory
        #[clap(long, default_value = "./dag")]
        dag: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Audit { job_id, dag } => audit_job(&job_id, dag).await?,
    }
    Ok(())
}

async fn audit_job(job_id_str: &str, dag_path: PathBuf) -> Result<()> {
    let cid = parse_cid_from_string(job_id_str)?;
    let job_id = JobId(cid.clone());

    let dag_store = TokioFileDagStore::new(dag_path)?;
    let dag_store: Arc<DagStoreMutexType<_>> = Arc::new(DagStoreMutexType::new(dag_store));

    let tmp = tempfile::NamedTempFile::new()?;
    let ledger = SimpleManaLedger::new(tmp.path().to_path_buf());

    let signer = Arc::new(StubSigner::new());
    let ctx = RuntimeContext::new_development(
        Did::from_str("did:key:zAudit")?,
        signer.clone(),
        ledger,
        None,
        Some(dag_store.clone() as Arc<DagStoreMutexType<_>>),
    )?;

    let lifecycle = ctx
        .get_job_status(&job_id)
        .await?
        .ok_or_else(|| anyhow!("Job not found"))?;
    let receipt = lifecycle
        .receipt
        .ok_or_else(|| anyhow!("No receipt for job"))?;

    let exec_receipt = ExecutionReceipt {
        job_id: receipt.job_id.0.clone(),
        executor_did: receipt.executor_did.clone(),
        result_cid: receipt.result_cid.clone(),
        cpu_ms: receipt.cpu_ms,
        success: receipt.success,
        sig: receipt.signature.clone(),
    };

    exec_receipt.verify_with_resolver(&icn_identity::KeyDidResolver)?;

    let spec = lifecycle.job.decode_spec()?;
    let actual_job = ActualMeshJob {
        id: lifecycle.job.id.clone(),
        manifest_cid: lifecycle.job.manifest_cid.clone(),
        spec,
        creator_did: lifecycle.job.submitter_did.clone(),
        cost_mana: lifecycle.job.cost_mana,
        max_execution_wait_ms: None,
        signature: icn_identity::SignatureBytes(vec![]),
    };

    let (sk, _) = generate_ed25519_keypair();
    let executor = SimpleExecutor::with_context(receipt.executor_did.clone(), sk, ctx.clone());
    let replay = executor.execute_job(&actual_job).await?;

    let outputs_match = replay.result_cid == exec_receipt.result_cid;
    let cpu_diff = replay.cpu_ms.abs_diff(exec_receipt.cpu_ms);
    println!("Job Audit Report");
    println!("Job: {job_id_str}");
    println!("Signature valid: true");
    println!("Output match: {outputs_match}");
    println!(
        "CPU ms recorded: {} | replayed: {} (diff {})",
        exec_receipt.cpu_ms, replay.cpu_ms, cpu_diff
    );
    println!("Success flag: {}", exec_receipt.success);

    Ok(())
}
