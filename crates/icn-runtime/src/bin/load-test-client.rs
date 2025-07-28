//! Load test client for ICN Core production validation
//!
//! This binary provides comprehensive load testing capabilities for ICN Core,
//! including job submission stress testing, burst traffic simulation, and
//! performance regression detection.

use clap::{Parser, Subcommand};
use icn_common::{Cid, Did};
use icn_mesh::{JobKind, JobSpec};
use log::{debug, error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::{interval, sleep};

#[derive(Parser)]
#[command(name = "load-test-client")]
#[command(about = "ICN Core Load Test Client for production validation")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Test type to run
    #[arg(long, default_value = "job-submission")]
    test_type: String,

    /// Request rate per second
    #[arg(long, default_value_t = 10.0)]
    rate: f64,

    /// Test duration in seconds
    #[arg(long, default_value_t = 60)]
    duration: u64,

    /// Job size in MB for large job tests
    #[arg(long, default_value_t = 1)]
    job_size_mb: usize,

    /// Output file for results
    #[arg(long, default_value = "load_test_results.json")]
    output: String,

    /// Test name for identification
    #[arg(long, default_value = "load_test")]
    test_name: String,

    /// ICN node endpoint
    #[arg(long, default_value = "http://localhost:7845")]
    endpoint: String,

    /// Number of concurrent connections
    #[arg(long, default_value_t = 10)]
    concurrent: usize,

    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run job submission load test
    JobSubmission,
    /// Run burst traffic test
    Burst,
    /// Run large jobs test
    LargeJobs,
    /// Run governance load test
    Governance,
    /// Run custom test scenario
    Custom,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoadTestResults {
    test_name: String,
    start_time: u64,
    end_time: u64,
    duration_seconds: u64,
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    success_rate: f64,
    avg_response_time_ms: f64,
    p95_response_time_ms: f64,
    p99_response_time_ms: f64,
    max_response_time_ms: f64,
    min_response_time_ms: f64,
    requests_per_second: f64,
    error_types: std::collections::HashMap<String, u64>,
    performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceMetrics {
    memory_usage_mb: Vec<f64>,
    cpu_usage_percent: Vec<f64>,
    network_io_bytes: Vec<u64>,
    error_rate_per_second: Vec<f64>,
}

#[derive(Debug)]
struct RequestResult {
    success: bool,
    response_time_ms: f64,
    error_type: Option<String>,
    timestamp: Instant,
}

struct LoadTestClient {
    client: Client,
    endpoint: String,
    test_node_did: String,
}

impl LoadTestClient {
    fn new(endpoint: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        // Generate a test DID for load testing
        let test_node_did = "did:key:z6MkLoadTestClient123456789".to_string();

        Self {
            client,
            endpoint,
            test_node_did,
        }
    }

    async fn submit_test_job(&self, job_size_bytes: usize) -> RequestResult {
        let start = Instant::now();

        // Create a test job with specified size
        let payload = "x".repeat(job_size_bytes);
        let job_spec = JobSpec {
            kind: JobKind::Echo { payload },
            ..Default::default()
        };

        let job_payload = json!({
            "manifest_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
            "spec": job_spec,
            "cost_mana": 10,
            "submitter_did": self.test_node_did
        });

        match self
            .client
            .post(&format!("{}/api/v1/jobs/submit", self.endpoint))
            .json(&job_payload)
            .send()
            .await
        {
            Ok(response) => {
                let response_time = start.elapsed().as_millis() as f64;

                if response.status().is_success() {
                    RequestResult {
                        success: true,
                        response_time_ms: response_time,
                        error_type: None,
                        timestamp: start,
                    }
                } else {
                    RequestResult {
                        success: false,
                        response_time_ms: response_time,
                        error_type: Some(format!("HTTP_{}", response.status().as_u16())),
                        timestamp: start,
                    }
                }
            }
            Err(e) => {
                let response_time = start.elapsed().as_millis() as f64;
                RequestResult {
                    success: false,
                    response_time_ms: response_time,
                    error_type: Some(format!("Network: {}", e)),
                    timestamp: start,
                }
            }
        }
    }

    async fn submit_governance_proposal(&self) -> RequestResult {
        let start = Instant::now();

        let proposal_payload = json!({
            "title": format!("Load Test Proposal {}", start.elapsed().as_nanos()),
            "description": "Test proposal for load testing",
            "proposal_type": "parameter_change",
            "changes": {
                "max_job_size": 1024000
            },
            "proposer_did": self.test_node_did
        });

        match self
            .client
            .post(&format!("{}/api/v1/governance/proposals", self.endpoint))
            .json(&proposal_payload)
            .send()
            .await
        {
            Ok(response) => {
                let response_time = start.elapsed().as_millis() as f64;
                RequestResult {
                    success: response.status().is_success(),
                    response_time_ms: response_time,
                    error_type: if response.status().is_success() {
                        None
                    } else {
                        Some(format!("HTTP_{}", response.status().as_u16()))
                    },
                    timestamp: start,
                }
            }
            Err(e) => {
                let response_time = start.elapsed().as_millis() as f64;
                RequestResult {
                    success: false,
                    response_time_ms: response_time,
                    error_type: Some(format!("Network: {}", e)),
                    timestamp: start,
                }
            }
        }
    }

    async fn get_node_status(&self) -> RequestResult {
        let start = Instant::now();

        match self
            .client
            .get(&format!("{}/api/v1/status", self.endpoint))
            .send()
            .await
        {
            Ok(response) => {
                let response_time = start.elapsed().as_millis() as f64;
                RequestResult {
                    success: response.status().is_success(),
                    response_time_ms: response_time,
                    error_type: if response.status().is_success() {
                        None
                    } else {
                        Some(format!("HTTP_{}", response.status().as_u16()))
                    },
                    timestamp: start,
                }
            }
            Err(e) => {
                let response_time = start.elapsed().as_millis() as f64;
                RequestResult {
                    success: false,
                    response_time_ms: response_time,
                    error_type: Some(format!("Network: {}", e)),
                    timestamp: start,
                }
            }
        }
    }
}

struct LoadTestRunner {
    client: LoadTestClient,
    results: Arc<tokio::sync::Mutex<Vec<RequestResult>>>,
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
}

impl LoadTestRunner {
    fn new(endpoint: String) -> Self {
        Self {
            client: LoadTestClient::new(endpoint),
            results: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
        }
    }

    async fn run_job_submission_test(
        &self,
        rate: f64,
        duration: Duration,
        job_size_bytes: usize,
        concurrent_workers: usize,
    ) -> LoadTestResults {
        info!(
            "Starting job submission load test: rate={}/s, duration={:?}, job_size={}B, workers={}",
            rate, duration, job_size_bytes, concurrent_workers
        );

        let start_time = Instant::now();
        let start_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Calculate interval between requests
        let request_interval = Duration::from_secs_f64(concurrent_workers as f64 / rate);

        // Spawn worker tasks
        let mut handles = Vec::new();

        for worker_id in 0..concurrent_workers {
            let client = LoadTestClient::new(self.client.endpoint.clone());
            let results = self.results.clone();
            let total_requests = self.total_requests.clone();
            let successful_requests = self.successful_requests.clone();
            let failed_requests = self.failed_requests.clone();

            let handle = tokio::spawn(async move {
                let mut interval_timer = interval(request_interval);
                let worker_start = Instant::now();

                while worker_start.elapsed() < duration {
                    interval_timer.tick().await;

                    let result = client.submit_test_job(job_size_bytes).await;

                    total_requests.fetch_add(1, Ordering::Relaxed);
                    if result.success {
                        successful_requests.fetch_add(1, Ordering::Relaxed);
                    } else {
                        failed_requests.fetch_add(1, Ordering::Relaxed);
                    }

                    results.lock().await.push(result);

                    // Log progress periodically
                    if total_requests.load(Ordering::Relaxed) % 100 == 0 {
                        debug!(
                            "Worker {}: {} requests completed",
                            worker_id,
                            total_requests.load(Ordering::Relaxed)
                        );
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all workers to complete
        for handle in handles {
            handle.await.expect("Worker task panicked");
        }

        let end_time = Instant::now();
        let end_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.generate_results(
            "job_submission",
            start_time,
            end_time,
            start_timestamp,
            end_timestamp,
        )
        .await
    }

    async fn run_burst_test(
        &self,
        burst_rate: f64,
        burst_duration: Duration,
        job_size_bytes: usize,
    ) -> LoadTestResults {
        info!(
            "Starting burst test: rate={}/s, duration={:?}",
            burst_rate, burst_duration
        );

        let start_time = Instant::now();
        let start_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Calculate requests to send in burst
        let total_requests_in_burst = (burst_rate * burst_duration.as_secs_f64()) as usize;
        let request_interval = burst_duration / total_requests_in_burst as u32;

        let mut handles = Vec::new();

        for _ in 0..total_requests_in_burst {
            let client = LoadTestClient::new(self.client.endpoint.clone());
            let results = self.results.clone();
            let total_requests = self.total_requests.clone();
            let successful_requests = self.successful_requests.clone();
            let failed_requests = self.failed_requests.clone();

            let handle = tokio::spawn(async move {
                let result = client.submit_test_job(job_size_bytes).await;

                total_requests.fetch_add(1, Ordering::Relaxed);
                if result.success {
                    successful_requests.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed_requests.fetch_add(1, Ordering::Relaxed);
                }

                results.lock().await.push(result);
            });

            handles.push(handle);

            // Small delay to spread requests
            sleep(request_interval).await;
        }

        // Wait for all requests to complete
        for handle in handles {
            handle.await.expect("Burst request task panicked");
        }

        let end_time = Instant::now();
        let end_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.generate_results(
            "burst",
            start_time,
            end_time,
            start_timestamp,
            end_timestamp,
        )
        .await
    }

    async fn run_governance_test(&self, rate: f64, duration: Duration) -> LoadTestResults {
        info!(
            "Starting governance load test: rate={}/s, duration={:?}",
            rate, duration
        );

        let start_time = Instant::now();
        let start_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let request_interval = Duration::from_secs_f64(1.0 / rate);
        let mut interval_timer = interval(request_interval);

        while start_time.elapsed() < duration {
            interval_timer.tick().await;

            let client = LoadTestClient::new(self.client.endpoint.clone());
            let results = self.results.clone();
            let total_requests = self.total_requests.clone();
            let successful_requests = self.successful_requests.clone();
            let failed_requests = self.failed_requests.clone();

            tokio::spawn(async move {
                let result = client.submit_governance_proposal().await;

                total_requests.fetch_add(1, Ordering::Relaxed);
                if result.success {
                    successful_requests.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed_requests.fetch_add(1, Ordering::Relaxed);
                }

                results.lock().await.push(result);
            });
        }

        // Wait a bit for final requests to complete
        sleep(Duration::from_secs(5)).await;

        let end_time = Instant::now();
        let end_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.generate_results(
            "governance",
            start_time,
            end_time,
            start_timestamp,
            end_timestamp,
        )
        .await
    }

    async fn generate_results(
        &self,
        test_name: &str,
        start_time: Instant,
        end_time: Instant,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> LoadTestResults {
        let results = self.results.lock().await;
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);

        if results.is_empty() {
            warn!("No results collected for test {}", test_name);
            return LoadTestResults {
                test_name: test_name.to_string(),
                start_time: start_timestamp,
                end_time: end_timestamp,
                duration_seconds: end_time.duration_since(start_time).as_secs(),
                total_requests: total,
                successful_requests: successful,
                failed_requests: failed,
                success_rate: 0.0,
                avg_response_time_ms: 0.0,
                p95_response_time_ms: 0.0,
                p99_response_time_ms: 0.0,
                max_response_time_ms: 0.0,
                min_response_time_ms: 0.0,
                requests_per_second: 0.0,
                error_types: std::collections::HashMap::new(),
                performance_metrics: PerformanceMetrics {
                    memory_usage_mb: vec![],
                    cpu_usage_percent: vec![],
                    network_io_bytes: vec![],
                    error_rate_per_second: vec![],
                },
            };
        }

        // Calculate response time statistics
        let mut response_times: Vec<f64> = results.iter().map(|r| r.response_time_ms).collect();
        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg_response_time = response_times.iter().sum::<f64>() / response_times.len() as f64;
        let p95_index = (response_times.len() as f64 * 0.95) as usize;
        let p99_index = (response_times.len() as f64 * 0.99) as usize;

        let p95_response_time = response_times.get(p95_index).copied().unwrap_or(0.0);
        let p99_response_time = response_times.get(p99_index).copied().unwrap_or(0.0);
        let max_response_time = response_times.last().copied().unwrap_or(0.0);
        let min_response_time = response_times.first().copied().unwrap_or(0.0);

        // Count error types
        let mut error_types = std::collections::HashMap::new();
        for result in results.iter() {
            if let Some(error_type) = &result.error_type {
                *error_types.entry(error_type.clone()).or_insert(0) += 1;
            }
        }

        let duration_seconds = end_time.duration_since(start_time).as_secs();
        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let requests_per_second = if duration_seconds > 0 {
            total as f64 / duration_seconds as f64
        } else {
            0.0
        };

        LoadTestResults {
            test_name: test_name.to_string(),
            start_time: start_timestamp,
            end_time: end_timestamp,
            duration_seconds,
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            success_rate,
            avg_response_time_ms: avg_response_time,
            p95_response_time_ms: p95_response_time,
            p99_response_time_ms: p99_response_time,
            max_response_time_ms: max_response_time,
            min_response_time_ms: min_response_time,
            requests_per_second,
            error_types,
            performance_metrics: PerformanceMetrics {
                memory_usage_mb: vec![], // Would be collected by external monitoring
                cpu_usage_percent: vec![],
                network_io_bytes: vec![],
                error_rate_per_second: vec![],
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    info!("Starting ICN Core load test client");
    info!(
        "Test configuration: type={}, rate={}/s, duration={}s",
        cli.test_type, cli.rate, cli.duration
    );

    let runner = LoadTestRunner::new(cli.endpoint.clone());
    let test_duration = Duration::from_secs(cli.duration);

    let results = match cli.test_type.as_str() {
        "job-submission" => {
            let job_size_bytes = cli.job_size_mb * 1024 * 1024;
            runner
                .run_job_submission_test(cli.rate, test_duration, job_size_bytes, cli.concurrent)
                .await
        }
        "burst" => {
            let job_size_bytes = cli.job_size_mb * 1024 * 1024;
            runner
                .run_burst_test(cli.rate, test_duration, job_size_bytes)
                .await
        }
        "large-jobs" => {
            let job_size_bytes = cli.job_size_mb * 1024 * 1024;
            runner
                .run_job_submission_test(cli.rate, test_duration, job_size_bytes, cli.concurrent)
                .await
        }
        "governance" => runner.run_governance_test(cli.rate, test_duration).await,
        _ => {
            error!("Unknown test type: {}", cli.test_type);
            std::process::exit(1);
        }
    };

    // Save results to file
    let results_json = serde_json::to_string_pretty(&results)?;
    std::fs::write(&cli.output, results_json)?;

    // Print summary
    info!("Load test completed!");
    info!("Total requests: {}", results.total_requests);
    info!("Successful requests: {}", results.successful_requests);
    info!("Failed requests: {}", results.failed_requests);
    info!("Success rate: {:.2}%", results.success_rate);
    info!(
        "Average response time: {:.2}ms",
        results.avg_response_time_ms
    );
    info!(
        "95th percentile response time: {:.2}ms",
        results.p95_response_time_ms
    );
    info!("Requests per second: {:.2}", results.requests_per_second);
    info!("Results saved to: {}", cli.output);

    if !results.error_types.is_empty() {
        warn!("Error breakdown:");
        for (error_type, count) in results.error_types {
            warn!("  {}: {}", error_type, count);
        }
    }

    Ok(())
}
