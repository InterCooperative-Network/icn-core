//! Docker sandboxing implementation for secure job execution.
//!
//! This module provides Docker container management with security policies,
//! resource limits, and isolation for mesh job execution.

use crate::{JobId, JobKind, JobSpec, Resources};
use icn_common::{CommonError, Did};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Docker container execution environment with security policies.
#[derive(Debug, Clone)]
pub struct DockerSandbox {
    /// Resource limits for the container
    pub resource_limits: DockerResourceLimits,
    /// Security configuration
    pub security_config: DockerSecurityConfig,
    /// Network configuration
    pub network_config: DockerNetworkConfig,
    /// Container image and execution parameters
    pub execution_config: DockerExecutionConfig,
}

/// Resource limits for Docker containers.
#[derive(Debug, Clone)]
pub struct DockerResourceLimits {
    /// CPU shares (relative weight)
    pub cpu_shares: u32,
    /// Memory limit in bytes
    pub memory_limit: u64,
    /// CPU cores limit (can be fractional, e.g., 1.5)
    pub cpu_limit: f64,
    /// Disk I/O read limit in bytes per second
    pub disk_read_bps: Option<u64>,
    /// Disk I/O write limit in bytes per second
    pub disk_write_bps: Option<u64>,
    /// Network bandwidth limit in bytes per second
    pub network_bps: Option<u64>,
    /// Maximum PIDs (process limit)
    pub pids_limit: u32,
    /// Execution timeout
    pub timeout: Duration,
}

/// Security configuration for Docker containers.
#[derive(Debug, Clone)]
pub struct DockerSecurityConfig {
    /// Run container with read-only root filesystem
    pub readonly_rootfs: bool,
    /// Disable privilege escalation
    pub no_new_privileges: bool,
    /// Linux capabilities to drop
    pub drop_capabilities: Vec<String>,
    /// Linux capabilities to add (if any)
    pub add_capabilities: Vec<String>,
    /// User to run as (format: "uid:gid" or "user:group")
    pub user: String,
    /// Security options (e.g., seccomp, apparmor)
    pub security_opts: Vec<String>,
    /// Disable inter-container communication
    pub icc: bool,
}

/// Network configuration for Docker containers.
#[derive(Debug, Clone)]
pub struct DockerNetworkConfig {
    /// Network mode (none, bridge, host, custom)
    pub mode: NetworkMode,
    /// DNS servers to use
    pub dns_servers: Vec<String>,
    /// Exposed ports (internal_port -> external_port)
    pub port_mappings: HashMap<u16, u16>,
    /// Allow outbound internet access
    pub allow_internet: bool,
}

/// Docker execution configuration.
#[derive(Debug, Clone)]
pub struct DockerExecutionConfig {
    /// Docker image to use
    pub image: String,
    /// Image tag/version
    pub tag: String,
    /// Command to execute
    pub command: Vec<String>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Working directory
    pub workdir: Option<String>,
    /// Volume mounts (host_path -> container_path)
    pub volumes: HashMap<String, String>,
}

/// Network mode for Docker containers.
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkMode {
    /// No network access
    None,
    /// Bridge network (default)
    Bridge,
    /// Host network (dangerous - avoid in production)
    Host,
    /// Custom network
    Custom(String),
}

/// Result of Docker container execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerExecutionResult {
    /// Exit code of the container
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution time
    pub execution_time: Duration,
    /// Resource usage statistics
    pub resource_usage: DockerResourceUsage,
    /// Container ID for reference
    pub container_id: String,
}

/// Resource usage statistics from Docker container execution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DockerResourceUsage {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Total CPU time used
    pub cpu_time_ms: u64,
    /// Network bytes received
    pub network_rx_bytes: u64,
    /// Network bytes transmitted
    pub network_tx_bytes: u64,
    /// Disk bytes read
    pub disk_read_bytes: u64,
    /// Disk bytes written
    pub disk_write_bytes: u64,
}

impl DockerSandbox {
    /// Create a new Docker sandbox from job specifications.
    pub fn from_job_spec(job_spec: &JobSpec, _executor_did: &Did) -> Result<Self, CommonError> {
        let docker_spec = match &job_spec.kind {
            JobKind::GenericPlaceholder => {
                return Err(CommonError::InvalidParameters(
                    "Cannot create Docker sandbox for placeholder job".to_string(),
                ));
            }
            // For now, we'll support a default configuration
            // In practice, job specs would contain Docker-specific configuration
            _ => DockerExecutionConfig {
                image: "ubuntu".to_string(),
                tag: "22.04".to_string(),
                command: vec!["echo".to_string(), "Hello from Docker".to_string()],
                environment: HashMap::new(),
                workdir: Some("/workspace".to_string()),
                volumes: HashMap::new(),
            },
        };

        let resource_limits =
            DockerResourceLimits::from_mesh_resources(&job_spec.required_resources);
        let security_config = DockerSecurityConfig::secure_defaults();
        let network_config = DockerNetworkConfig::restricted_defaults();

        Ok(Self {
            resource_limits,
            security_config,
            network_config,
            execution_config: docker_spec,
        })
    }

    /// Execute a job in the Docker sandbox.
    pub async fn execute_job(
        &self,
        job_id: &JobId,
        _input_data: &[u8],
    ) -> Result<DockerExecutionResult, CommonError> {
        info!("[DockerSandbox] Starting job execution for {}", job_id);

        let start_time = Instant::now();
        let container_name = format!("icn-job-{}", job_id.to_string().replace(':', "-"));

        // Build Docker run command
        let mut docker_cmd = Command::new("docker");
        docker_cmd.arg("run");

        // Add basic options
        docker_cmd.args(["--rm", "--name", &container_name]);

        // Add resource limits
        self.add_resource_limits(&mut docker_cmd);

        // Add security configuration
        self.add_security_config(&mut docker_cmd);

        // Add network configuration
        self.add_network_config(&mut docker_cmd);

        // Add execution configuration
        self.add_execution_config(&mut docker_cmd);

        // Add the image
        docker_cmd.arg(format!(
            "{}:{}",
            self.execution_config.image, self.execution_config.tag
        ));

        // Add the command
        docker_cmd.args(&self.execution_config.command);

        debug!("[DockerSandbox] Docker command: {:?}", docker_cmd);

        // Execute with timeout
        let output = tokio::time::timeout(
            self.resource_limits.timeout,
            tokio::task::spawn_blocking(move || {
                docker_cmd
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
            }),
        )
        .await;

        let execution_time = start_time.elapsed();

        match output {
            Ok(Ok(Ok(output))) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                info!(
                    "[DockerSandbox] Job {} completed with exit code {} in {:?}",
                    job_id, exit_code, execution_time
                );

                // Collect resource usage (simplified - in production, use docker stats)
                let resource_usage = self.collect_resource_usage(&container_name).await;

                Ok(DockerExecutionResult {
                    exit_code,
                    stdout,
                    stderr,
                    execution_time,
                    resource_usage,
                    container_id: container_name,
                })
            }
            Ok(Ok(Err(e))) => {
                error!("[DockerSandbox] Failed to execute Docker command: {}", e);
                Err(CommonError::InternalError(format!(
                    "Docker execution failed: {}",
                    e
                )))
            }
            Ok(Err(e)) => {
                error!("[DockerSandbox] Docker task panicked: {}", e);
                Err(CommonError::InternalError(format!(
                    "Docker task panicked: {}",
                    e
                )))
            }
            Err(_) => {
                error!(
                    "[DockerSandbox] Docker execution timed out for job {}",
                    job_id
                );
                // Try to kill the container
                self.cleanup_container(&container_name).await;
                Err(CommonError::InternalError(
                    "Docker execution timed out".to_string(),
                ))
            }
        }
    }

    /// Add resource limit arguments to Docker command.
    fn add_resource_limits(&self, cmd: &mut Command) {
        // Memory limit
        cmd.args(["--memory", &self.resource_limits.memory_limit.to_string()]);

        // CPU limits
        cmd.args(["--cpu-shares", &self.resource_limits.cpu_shares.to_string()]);
        cmd.args(["--cpus", &self.resource_limits.cpu_limit.to_string()]);

        // PIDs limit
        cmd.args(["--pids-limit", &self.resource_limits.pids_limit.to_string()]);

        // Disk I/O limits (if specified)
        if let Some(read_bps) = self.resource_limits.disk_read_bps {
            cmd.args(["--device-read-bps", &format!("/dev/sda:{}", read_bps)]);
        }
        if let Some(write_bps) = self.resource_limits.disk_write_bps {
            cmd.args(["--device-write-bps", &format!("/dev/sda:{}", write_bps)]);
        }
    }

    /// Add security configuration to Docker command.
    fn add_security_config(&self, cmd: &mut Command) {
        if self.security_config.readonly_rootfs {
            cmd.arg("--read-only");
        }

        if self.security_config.no_new_privileges {
            cmd.args(["--security-opt", "no-new-privileges:true"]);
        }

        // Drop capabilities
        for cap in &self.security_config.drop_capabilities {
            cmd.args(["--cap-drop", cap]);
        }

        // Add capabilities
        for cap in &self.security_config.add_capabilities {
            cmd.args(["--cap-add", cap]);
        }

        // User
        if !self.security_config.user.is_empty() {
            cmd.args(["--user", &self.security_config.user]);
        }

        // Additional security options
        for opt in &self.security_config.security_opts {
            cmd.args(["--security-opt", opt]);
        }
    }

    /// Add network configuration to Docker command.
    fn add_network_config(&self, cmd: &mut Command) {
        match &self.network_config.mode {
            NetworkMode::None => {
                cmd.args(["--network", "none"]);
            }
            NetworkMode::Bridge => {
                cmd.args(["--network", "bridge"]);
            }
            NetworkMode::Host => {
                cmd.args(["--network", "host"]);
            }
            NetworkMode::Custom(network) => {
                cmd.args(["--network", network]);
            }
        }

        // DNS servers
        for dns in &self.network_config.dns_servers {
            cmd.args(["--dns", dns]);
        }

        // Port mappings
        for (internal, external) in &self.network_config.port_mappings {
            cmd.args(["-p", &format!("{}:{}", external, internal)]);
        }
    }

    /// Add execution configuration to Docker command.
    fn add_execution_config(&self, cmd: &mut Command) {
        // Environment variables
        for (key, value) in &self.execution_config.environment {
            cmd.args(["--env", &format!("{}={}", key, value)]);
        }

        // Working directory
        if let Some(workdir) = &self.execution_config.workdir {
            cmd.args(["--workdir", workdir]);
        }

        // Volume mounts
        for (host_path, container_path) in &self.execution_config.volumes {
            cmd.args(["-v", &format!("{}:{}", host_path, container_path)]);
        }
    }

    /// Collect resource usage statistics from container.
    ///
    /// **Placeholder implementation:** This method is not yet implemented and always returns default values.
    /// In a real implementation, this would use `docker stats` or the Docker API to collect
    /// actual resource usage statistics (CPU, memory, I/O, etc.) for the specified container.
    async fn collect_resource_usage(&self, _container_name: &str) -> DockerResourceUsage {
        // In a real implementation, we would use `docker stats` or the Docker API
        // to collect actual resource usage. For now, return defaults.
        warn!("[DockerSandbox] Resource usage collection not yet implemented");
        DockerResourceUsage::default()
    }

    /// Clean up container after execution.
    async fn cleanup_container(&self, container_name: &str) {
        debug!("[DockerSandbox] Cleaning up container {}", container_name);

        let mut kill_cmd = Command::new("docker");
        kill_cmd.args(["kill", container_name]);

        if let Err(e) = kill_cmd.output() {
            warn!(
                "[DockerSandbox] Failed to kill container {}: {}",
                container_name, e
            );
        }
    }
}

impl DockerResourceLimits {
    /// Create Docker resource limits from mesh job resource requirements.
    pub fn from_mesh_resources(resources: &Resources) -> Self {
        Self {
            cpu_shares: resources.cpu_cores * 1024, // Docker CPU shares
            memory_limit: (resources.memory_mb as u64) * 1024 * 1024, // Convert MB to bytes
            cpu_limit: resources.cpu_cores as f64,
            disk_read_bps: None, // Could be derived from storage requirements
            disk_write_bps: None,
            network_bps: None, // Could be derived from network requirements
            pids_limit: 1024,  // Reasonable default
            timeout: Duration::from_secs(300), // 5 minute default timeout
        }
    }

    /// Create resource limits for high-security jobs.
    pub fn high_security() -> Self {
        Self {
            cpu_shares: 512,                       // Lower CPU priority
            memory_limit: 128 * 1024 * 1024,       // 128 MB limit
            cpu_limit: 0.5,                        // Half a CPU core
            disk_read_bps: Some(10 * 1024 * 1024), // 10 MB/s
            disk_write_bps: Some(5 * 1024 * 1024), // 5 MB/s
            network_bps: Some(1024 * 1024),        // 1 MB/s
            pids_limit: 64,                        // Very limited processes
            timeout: Duration::from_secs(60),      // 1 minute timeout
        }
    }
}

impl DockerSecurityConfig {
    /// Create secure default configuration.
    pub fn secure_defaults() -> Self {
        Self {
            readonly_rootfs: true,
            no_new_privileges: true,
            drop_capabilities: vec!["ALL".to_string()],
            add_capabilities: vec![], // No additional capabilities
            user: "nobody:nogroup".to_string(),
            security_opts: vec![
                "no-new-privileges:true".to_string(),
                "seccomp=default".to_string(),
            ],
            icc: false, // Disable inter-container communication
        }
    }

    /// Create configuration for trusted executors.
    pub fn trusted_defaults() -> Self {
        Self {
            readonly_rootfs: false, // Allow some filesystem writes
            no_new_privileges: true,
            drop_capabilities: vec![
                "SYS_ADMIN".to_string(),
                "SYS_MODULE".to_string(),
                "SYS_TIME".to_string(),
            ],
            add_capabilities: vec![],
            user: "icn:icn".to_string(),
            security_opts: vec!["seccomp=default".to_string()],
            icc: false,
        }
    }
}

impl DockerNetworkConfig {
    /// Create restricted network configuration (no external access).
    pub fn restricted_defaults() -> Self {
        Self {
            mode: NetworkMode::None,
            dns_servers: vec![],
            port_mappings: HashMap::new(),
            allow_internet: false,
        }
    }

    /// Create network configuration with limited internet access.
    pub fn limited_internet() -> Self {
        Self {
            mode: NetworkMode::Bridge,
            dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            port_mappings: HashMap::new(),
            allow_internet: true,
        }
    }
}

/// Manages multiple Docker sandboxes and execution.
pub struct DockerSandboxManager {
    /// Default security configuration
    default_security_config: DockerSecurityConfig,
    /// Maximum concurrent executions
    max_concurrent: usize,
    /// Currently running containers
    running_containers: std::sync::RwLock<HashMap<String, Instant>>,
}

impl DockerSandboxManager {
    /// Create a new Docker sandbox manager.
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            default_security_config: DockerSecurityConfig::secure_defaults(),
            max_concurrent,
            running_containers: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Check if Docker is available on the system.
    pub async fn check_docker_availability() -> bool {
        let output = Command::new("docker").args(["--version"]).output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Execute a job in a managed Docker sandbox.
    pub async fn execute_job(
        &self,
        job_id: &JobId,
        job_spec: &JobSpec,
        _executor_did: &Did,
        input_data: &[u8],
    ) -> Result<DockerExecutionResult, CommonError> {
        // Check capacity
        {
            let running = self.running_containers.read().unwrap();
            if running.len() >= self.max_concurrent {
                return Err(CommonError::InternalError(
                    "Docker executor at capacity".to_string(),
                ));
            }
        }

        // Create sandbox for this job
        let sandbox = DockerSandbox::from_job_spec(job_spec, _executor_did)?;

        // Track execution
        let container_name = format!("icn-job-{}", job_id.to_string().replace(':', "-"));
        {
            let mut running = self.running_containers.write().unwrap();
            running.insert(container_name.clone(), Instant::now());
        }

        // Execute the job
        let result = sandbox.execute_job(job_id, input_data).await;

        // Clean up tracking
        {
            let mut running = self.running_containers.write().unwrap();
            running.remove(&container_name);
        }

        result
    }

    /// Get current capacity information.
    pub fn get_capacity(&self) -> (usize, usize) {
        let running = self.running_containers.read().unwrap();
        (running.len(), self.max_concurrent)
    }

    /// Kill all running containers (emergency cleanup).
    pub async fn emergency_cleanup(&self) {
        let container_names: Vec<String> = {
            let running = self.running_containers.read().unwrap();
            running.keys().cloned().collect()
        };

        for container_name in container_names {
            let mut kill_cmd = Command::new("docker");
            kill_cmd.args(["kill", &container_name]);

            if let Err(e) = kill_cmd.output() {
                error!(
                    "[DockerSandboxManager] Failed to kill container {}: {}",
                    container_name, e
                );
            }
        }

        // Clear tracking
        {
            let mut running = self.running_containers.write().unwrap();
            running.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JobKind, JobSpec};
    use icn_common::Did;
    use std::str::FromStr;

    #[test]
    fn test_docker_resource_limits_from_mesh_resources() {
        let resources = Resources {
            cpu_cores: 2,
            memory_mb: 1024,
            storage_mb: 2048,
        };

        let limits = DockerResourceLimits::from_mesh_resources(&resources);

        assert_eq!(limits.cpu_shares, 2048); // 2 * 1024
        assert_eq!(limits.memory_limit, 1024 * 1024 * 1024); // 1024 MB in bytes
        assert_eq!(limits.cpu_limit, 2.0);
    }

    #[test]
    fn test_docker_security_config_defaults() {
        let config = DockerSecurityConfig::secure_defaults();

        assert!(config.readonly_rootfs);
        assert!(config.no_new_privileges);
        assert!(config.drop_capabilities.contains(&"ALL".to_string()));
        assert_eq!(config.user, "nobody:nogroup");
    }

    #[test]
    fn test_docker_network_config_restricted() {
        let config = DockerNetworkConfig::restricted_defaults();

        assert_eq!(config.mode, NetworkMode::None);
        assert!(!config.allow_internet);
        assert!(config.dns_servers.is_empty());
    }

    #[tokio::test]
    async fn test_docker_sandbox_manager_capacity() {
        let manager = DockerSandboxManager::new(2);
        let (current, max) = manager.get_capacity();

        assert_eq!(current, 0);
        assert_eq!(max, 2);
    }

    #[test]
    fn test_docker_sandbox_from_job_spec() {
        let job_spec = JobSpec {
            kind: JobKind::Echo {
                payload: "test".to_string(),
            },
            required_resources: Resources {
                cpu_cores: 1,
                memory_mb: 512,
                storage_mb: 1024,
            },
            ..Default::default()
        };

        let executor_did = Did::from_str("did:key:test").unwrap();
        let sandbox = DockerSandbox::from_job_spec(&job_spec, &executor_did);

        assert!(sandbox.is_ok());
        let sandbox = sandbox.unwrap();
        assert_eq!(sandbox.resource_limits.cpu_shares, 1024);
        assert_eq!(sandbox.resource_limits.memory_limit, 512 * 1024 * 1024);
    }
}
