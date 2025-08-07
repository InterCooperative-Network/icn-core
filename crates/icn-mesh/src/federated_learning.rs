//! Federated Learning implementation for privacy-preserving distributed machine learning.
//!
//! This module provides coordination primitives for federated learning, including
//! model distribution, privacy-preserving aggregation, and multi-round training workflows.

use crate::Resources;
use icn_common::{Cid, CommonError, Did};
use icn_identity::{SignatureBytes, SigningKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use log::{debug, info};

/// Specification for a federated learning model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Model architecture identifier
    pub architecture: ModelArchitecture,
    /// Model parameters (serialized weights, etc.)
    pub parameters: Vec<u8>,
    /// Model metadata
    pub metadata: ModelMetadata,
    /// Hyperparameters for training
    pub hyperparameters: TrainingHyperparameters,
    /// Privacy configuration
    pub privacy_config: PrivacyConfig,
}

/// Model architecture specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelArchitecture {
    /// Neural network with layer specifications
    NeuralNetwork {
        layers: Vec<LayerSpec>,
        activation: ActivationFunction,
        loss_function: LossFunction,
    },
    /// Linear model (regression, classification)
    Linear {
        features: u32,
        output_classes: u32,
        regularization: RegularizationType,
    },
    /// Tree-based model
    TreeBased {
        max_depth: u32,
        num_trees: u32,
        boosting: bool,
    },
    /// Custom model with user-defined architecture
    Custom {
        model_type: String,
        parameters: HashMap<String, String>,
    },
}

/// Neural network layer specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerSpec {
    /// Layer type
    pub layer_type: LayerType,
    /// Number of units/neurons
    pub units: u32,
    /// Layer-specific parameters
    pub parameters: HashMap<String, f32>,
}

/// Neural network layer types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerType {
    Dense,
    Convolutional,
    LSTM,
    Attention,
    Dropout,
    BatchNorm,
}

/// Activation functions for neural networks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationFunction {
    ReLU,
    Sigmoid,
    Tanh,
    Softmax,
    Swish,
    GELU,
}

/// Loss functions for training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LossFunction {
    MeanSquaredError,
    CrossEntropy,
    BinaryCrossEntropy,
    Huber,
    Custom(String),
}

/// Regularization types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegularizationType {
    None,
    L1(f32),
    L2(f32),
    ElasticNet { l1: f32, l2: f32 },
}

/// Model metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model name/identifier
    pub name: String,
    /// Model version
    pub version: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Model accuracy metrics
    pub metrics: HashMap<String, f64>,
    /// Dataset information the model was trained on
    pub dataset_info: DatasetInfo,
}

/// Information about training dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    /// Dataset size (number of samples)
    pub size: u64,
    /// Feature dimensions
    pub feature_dimensions: Vec<u32>,
    /// Label/target information
    pub target_info: TargetInfo,
    /// Data distribution characteristics
    pub distribution: DataDistribution,
}

/// Target/label information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetInfo {
    /// Classification with class names
    Classification { classes: Vec<String> },
    /// Regression with value range
    Regression { min_value: f64, max_value: f64 },
    /// Multi-label classification
    MultiLabel { labels: Vec<String> },
}

/// Data distribution characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDistribution {
    /// Whether data is IID (independent and identically distributed)
    pub is_iid: bool,
    /// Skewness measure (0 = uniform, higher = more skewed)
    pub skewness: f64,
    /// Regional/demographic distribution info
    pub demographics: HashMap<String, f64>,
}

/// Training hyperparameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingHyperparameters {
    /// Learning rate
    pub learning_rate: f64,
    /// Batch size for local training
    pub batch_size: u32,
    /// Number of local epochs per round
    pub local_epochs: u32,
    /// Optimizer type
    pub optimizer: OptimizerType,
    /// Maximum rounds of federated training
    pub max_rounds: u32,
    /// Convergence criteria
    pub convergence_criteria: ConvergenceCriteria,
}

/// Optimizer types for training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizerType {
    SGD { momentum: f64 },
    Adam { beta1: f64, beta2: f64 },
    AdaGrad,
    RMSprop { decay: f64 },
    Custom(String),
}

/// Convergence criteria for stopping training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceCriteria {
    /// Minimum improvement required to continue training
    pub min_improvement: f64,
    /// Maximum rounds without improvement before stopping
    pub patience: u32,
    /// Target accuracy to reach
    pub target_accuracy: Option<f64>,
    /// Maximum training time
    pub max_training_time: Option<Duration>,
}

/// Privacy configuration for federated learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Differential privacy settings
    pub differential_privacy: Option<DifferentialPrivacyConfig>,
    /// Secure aggregation settings
    pub secure_aggregation: bool,
    /// Homomorphic encryption settings
    pub homomorphic_encryption: Option<HomomorphicEncryptionConfig>,
    /// Minimum number of participants required
    pub min_participants: u32,
    /// Maximum data sharing allowed
    pub max_data_sharing: DataSharingLevel,
}

/// Differential privacy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialPrivacyConfig {
    /// Privacy budget (epsilon)
    pub epsilon: f64,
    /// Delta parameter for (ε,δ)-differential privacy
    pub delta: f64,
    /// Noise mechanism
    pub noise_mechanism: NoiseMechanism,
    /// Clipping threshold for gradients
    pub clipping_threshold: f64,
}

/// Noise mechanisms for differential privacy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoiseMechanism {
    Gaussian,
    Laplacian,
    Custom(String),
}

/// Homomorphic encryption configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomomorphicEncryptionConfig {
    /// Encryption scheme
    pub scheme: EncryptionScheme,
    /// Key size
    pub key_size: u32,
    /// Noise budget for operations
    pub noise_budget: u32,
}

/// Homomorphic encryption schemes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionScheme {
    CKKS,
    BFV,
    BGV,
    Custom(String),
}

/// Data sharing levels for privacy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSharingLevel {
    /// No raw data sharing, only model updates
    ModelUpdatesOnly,
    /// Allow sharing of aggregated statistics
    AggregatedStatistics,
    /// Allow sharing of synthetic data
    SyntheticData,
    /// Full data sharing (not recommended for privacy)
    FullData,
}

/// A federated learning training round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedTrainingRound {
    /// Round number (0-based)
    pub round_number: u32,
    /// Global model state at start of round
    pub global_model_cid: Cid,
    /// Participants in this round
    pub participants: Vec<FederatedParticipant>,
    /// Training parameters for this round
    pub round_config: RoundConfig,
    /// Deadline for this round
    pub deadline: SystemTime,
    /// Round status
    pub status: RoundStatus,
}

/// Information about a federated learning participant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedParticipant {
    /// Participant's DID
    pub did: Did,
    /// Dataset size the participant has
    pub dataset_size: u64,
    /// Computational capacity
    pub compute_capacity: Resources,
    /// Reputation score
    pub reputation: f64,
    /// Geographic/network region
    pub region: Option<String>,
    /// Participation history
    pub participation_history: ParticipationHistory,
}

/// History of participant involvement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationHistory {
    /// Total rounds participated in
    pub total_rounds: u32,
    /// Successful completions
    pub successful_rounds: u32,
    /// Average contribution quality
    pub avg_contribution_quality: f64,
    /// Reliability score (0-1)
    pub reliability: f64,
}

/// Configuration for a training round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundConfig {
    /// Target number of participants
    pub target_participants: u32,
    /// Minimum participants required to proceed
    pub min_participants: u32,
    /// Local training epochs
    pub local_epochs: u32,
    /// Aggregation strategy
    pub aggregation_strategy: AggregationStrategy,
    /// Quality thresholds for contributions
    pub quality_thresholds: QualityThresholds,
}

/// Aggregation strategies for combining model updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Federated averaging (FedAvg)
    FederatedAveraging,
    /// Weighted average by dataset size
    WeightedByDataSize,
    /// Weighted average by contribution quality
    WeightedByQuality,
    /// Median aggregation (robust to outliers)
    Median,
    /// Secure aggregation with cryptographic protection
    SecureAggregation,
    /// Custom aggregation function
    Custom(String),
}

/// Quality thresholds for accepting contributions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum accuracy improvement required
    pub min_accuracy_improvement: f64,
    /// Maximum loss increase allowed
    pub max_loss_increase: f64,
    /// Minimum gradient norm
    pub min_gradient_norm: f64,
    /// Maximum gradient norm (for anomaly detection)
    pub max_gradient_norm: f64,
}

/// Status of a training round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoundStatus {
    /// Round is being set up
    Initializing,
    /// Waiting for participants to join
    WaitingForParticipants,
    /// Local training in progress
    Training,
    /// Collecting model updates
    CollectingUpdates,
    /// Aggregating updates
    Aggregating,
    /// Round completed successfully
    Completed,
    /// Round failed
    Failed { reason: String },
    /// Round was cancelled
    Cancelled,
}

/// Update from a participant after local training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUpdate {
    /// Participant who created this update
    pub participant_did: Did,
    /// Round this update belongs to
    pub round_number: u32,
    /// Model parameters after local training
    pub parameters: Vec<u8>,
    /// Training metadata
    pub training_metadata: TrainingMetadata,
    /// Proof of training (if required)
    pub proof_of_training: Option<ProofOfTraining>,
    /// Signature from participant
    pub signature: SignatureBytes,
}

/// Metadata about local training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetadata {
    /// Number of local samples used
    pub local_samples: u64,
    /// Local training loss
    pub local_loss: f64,
    /// Local accuracy (if applicable)
    pub local_accuracy: Option<f64>,
    /// Training time taken
    pub training_time_ms: u64,
    /// Compute resources used
    pub resources_used: Resources,
    /// Gradient norms for quality assessment
    pub gradient_norms: Vec<f64>,
}

/// Cryptographic proof that training was performed correctly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfTraining {
    /// Type of proof
    pub proof_type: ProofType,
    /// Proof data
    pub proof_data: Vec<u8>,
    /// Verification key
    pub verification_key: Vec<u8>,
}

/// Types of training proofs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    /// Zero-knowledge proof of computation
    ZKProof,
    /// Trusted execution environment attestation
    TEEAttestation,
    /// Verifiable computation proof
    VerifiableComputation,
    /// Custom proof mechanism
    Custom(String),
}

/// Result of federated training aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    /// New global model after aggregation
    pub global_model_cid: Cid,
    /// Participants included in aggregation
    pub included_participants: Vec<Did>,
    /// Participants excluded and reasons
    pub excluded_participants: Vec<(Did, String)>,
    /// Aggregation quality metrics
    pub quality_metrics: AggregationQualityMetrics,
    /// Convergence status
    pub convergence_status: ConvergenceStatus,
}

/// Quality metrics for aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationQualityMetrics {
    /// Global model accuracy after aggregation
    pub global_accuracy: f64,
    /// Global model loss after aggregation
    pub global_loss: f64,
    /// Variance in participant contributions
    pub contribution_variance: f64,
    /// Number of anomalous contributions detected
    pub anomalous_contributions: u32,
    /// Consensus score among participants
    pub consensus_score: f64,
}

/// Convergence status of federated training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConvergenceStatus {
    /// Training should continue
    Continue,
    /// Converged based on accuracy improvement
    ConvergedAccuracy,
    /// Converged based on loss stabilization
    ConvergedLoss,
    /// Reached maximum rounds
    MaxRoundsReached,
    /// Timed out
    TimedOut,
    /// Diverged (quality decreased)
    Diverged,
}

/// Federated learning coordinator that manages the training process.
pub struct FederatedLearningCoordinator {
    /// Coordinator's DID
    coordinator_did: Did,
    /// Signing key for authentication
    signing_key: Arc<SigningKey>,
    /// Current training sessions
    active_sessions: std::sync::RwLock<HashMap<String, FederatedSession>>,
    /// Configuration
    config: FederatedConfig,
}

/// Configuration for federated learning coordinator.
#[derive(Debug, Clone)]
pub struct FederatedConfig {
    /// Maximum concurrent sessions
    pub max_concurrent_sessions: u32,
    /// Default timeout for rounds
    pub default_round_timeout: Duration,
    /// Minimum participants for any session
    pub min_global_participants: u32,
    /// Maximum rounds per session
    pub max_rounds_per_session: u32,
}

/// Active federated learning session.
#[derive(Debug, Clone)]
struct FederatedSession {
    /// Session identifier
    session_id: String,
    /// Model being trained
    model_spec: ModelSpec,
    /// Current round
    current_round: Option<FederatedTrainingRound>,
    /// Training history
    round_history: Vec<FederatedTrainingRound>,
    /// Registered participants
    registered_participants: Vec<FederatedParticipant>,
    /// Session start time
    started_at: SystemTime,
    /// Session status
    status: SessionStatus,
}

/// Status of a federated learning session.
#[derive(Debug, Clone)]
enum SessionStatus {
    Initializing,
    RecruitingParticipants,
    Training,
    Completed,
    Failed(String),
}

impl FederatedLearningCoordinator {
    /// Create a new federated learning coordinator.
    pub fn new(coordinator_did: Did, signing_key: Arc<SigningKey>, config: FederatedConfig) -> Self {
        Self {
            coordinator_did,
            signing_key,
            active_sessions: std::sync::RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Start a new federated learning session.
    pub async fn start_session(
        &self,
        session_id: String,
        model_spec: ModelSpec,
        participants: Vec<FederatedParticipant>,
    ) -> Result<(), CommonError> {
        info!("[FederatedLearning] Starting session {} with {} participants", 
              session_id, participants.len());

        if participants.len() < self.config.min_global_participants as usize {
            return Err(CommonError::InvalidParameters(
                format!("Insufficient participants: {} < {}", 
                        participants.len(), self.config.min_global_participants)
            ));
        }

        let session = FederatedSession {
            session_id: session_id.clone(),
            model_spec,
            current_round: None,
            round_history: Vec::new(),
            registered_participants: participants,
            started_at: SystemTime::now(),
            status: SessionStatus::Initializing,
        };

        {
            let mut sessions = self.active_sessions.write().unwrap();
            if sessions.len() >= self.config.max_concurrent_sessions as usize {
                return Err(CommonError::InternalError(
                    "Maximum concurrent sessions reached".to_string()
                ));
            }
            sessions.insert(session_id.clone(), session);
        }

        // Start the first training round
        self.start_round(&session_id, 0).await?;

        Ok(())
    }

    /// Start a specific training round.
    async fn start_round(&self, session_id: &str, round_number: u32) -> Result<(), CommonError> {
        debug!("[FederatedLearning] Starting round {} for session {}", round_number, session_id);

        let (model_cid, participants) = {
            let sessions = self.active_sessions.read().unwrap();
            let session = sessions.get(session_id)
                .ok_or_else(|| CommonError::InvalidParameters("Session not found".to_string()))?;

            // Get model CID from previous round or initial model
            let model_cid = if round_number == 0 {
                // For the first round, we'd store the initial model in DAG
                Cid::new_v1_sha256(0x55, format!("initial_model_{}", session_id).as_bytes())
            } else {
                // Get from previous round's result
                session.round_history.last()
                    .and_then(|r| if let RoundStatus::Completed = r.status { 
                        Some(r.global_model_cid.clone()) 
                    } else { 
                        None 
                    })
                    .ok_or_else(|| CommonError::InternalError("No previous round result".to_string()))?
            };

            (model_cid, session.registered_participants.clone())
        };

        let round = FederatedTrainingRound {
            round_number,
            global_model_cid: model_cid.clone(),
            participants,
            round_config: RoundConfig {
                target_participants: 5, // Default
                min_participants: 3,    // Default
                local_epochs: 1,        // Default
                aggregation_strategy: AggregationStrategy::FederatedAveraging,
                quality_thresholds: QualityThresholds {
                    min_accuracy_improvement: 0.001,
                    max_loss_increase: 0.1,
                    min_gradient_norm: 0.001,
                    max_gradient_norm: 10.0,
                },
            },
            deadline: SystemTime::now() + self.config.default_round_timeout,
            status: RoundStatus::WaitingForParticipants,
        };

        // Update session with new round
        {
            let mut sessions = self.active_sessions.write().unwrap();
            let session = sessions.get_mut(session_id)
                .ok_or_else(|| CommonError::InvalidParameters("Session not found".to_string()))?;
            session.current_round = Some(round);
            session.status = SessionStatus::Training;
        }

        // Distribute model to participants (simplified)
        self.distribute_model_to_participants(session_id, &model_cid).await?;

        Ok(())
    }

    /// Distribute the global model to participants for local training.
    async fn distribute_model_to_participants(
        &self,
        session_id: &str,
        model_cid: &Cid,
    ) -> Result<(), CommonError> {
        debug!("[FederatedLearning] Distributing model {} to participants", model_cid);

        // In a real implementation, this would:
        // 1. Retrieve model from DAG storage
        // 2. Send model to each participant via mesh network
        // 3. Wait for acknowledgments
        // 4. Handle failures and retries

        // For now, just simulate the distribution
        tokio::time::sleep(Duration::from_millis(100)).await;

        info!("[FederatedLearning] Model distributed to participants for session {}", session_id);
        Ok(())
    }

    /// Process a model update from a participant.
    pub async fn process_model_update(
        &self,
        session_id: &str,
        update: ModelUpdate,
    ) -> Result<(), CommonError> {
        debug!("[FederatedLearning] Processing update from {} for session {}", 
               update.participant_did, session_id);

        // Validate the update
        self.validate_model_update(&update)?;

        // Store the update (in a real implementation, this would go to DAG)
        self.store_model_update(session_id, update).await?;

        // Check if we have enough updates to proceed with aggregation
        if self.ready_for_aggregation(session_id).await? {
            self.aggregate_round(session_id).await?;
        }

        Ok(())
    }

    /// Validate a model update.
    fn validate_model_update(&self, update: &ModelUpdate) -> Result<(), CommonError> {
        // Basic validation - in production this would be much more comprehensive
        if update.parameters.is_empty() {
            return Err(CommonError::InvalidParameters("Empty model parameters".to_string()));
        }

        if update.training_metadata.local_samples == 0 {
            return Err(CommonError::InvalidParameters("No training samples reported".to_string()));
        }

        // Validate gradient norms are within reasonable bounds
        for &norm in &update.training_metadata.gradient_norms {
            if norm < 0.0 || norm > 1000.0 {
                return Err(CommonError::InvalidParameters(
                    format!("Gradient norm out of bounds: {}", norm)
                ));
            }
        }

        Ok(())
    }

    /// Store a model update.
    async fn store_model_update(&self, session_id: &str, update: ModelUpdate) -> Result<(), CommonError> {
        // In a real implementation, this would store the update in DAG storage
        debug!("[FederatedLearning] Stored update from {} for session {}", 
               update.participant_did, session_id);
        Ok(())
    }

    /// Check if enough updates have been received for aggregation.
    async fn ready_for_aggregation(&self, session_id: &str) -> Result<bool, CommonError> {
        let sessions = self.active_sessions.read().unwrap();
        let session = sessions.get(session_id)
            .ok_or_else(|| CommonError::InvalidParameters("Session not found".to_string()))?;

        if let Some(round) = &session.current_round {
            // In a real implementation, we'd check stored updates
            // For now, simulate that we have enough updates
            let min_participants = round.round_config.min_participants;
            let received_updates = 3; // Simulated
            
            Ok(received_updates >= min_participants)
        } else {
            Ok(false)
        }
    }

    /// Aggregate model updates for the current round.
    async fn aggregate_round(&self, session_id: &str) -> Result<(), CommonError> {
        info!("[FederatedLearning] Aggregating round for session {}", session_id);

        let aggregation_result = self.perform_aggregation(session_id).await?;

        // Update session with aggregation result
        {
            let mut sessions = self.active_sessions.write().unwrap();
            let session = sessions.get_mut(session_id)
                .ok_or_else(|| CommonError::InvalidParameters("Session not found".to_string()))?;

            if let Some(mut round) = session.current_round.take() {
                round.status = RoundStatus::Completed;
                round.global_model_cid = aggregation_result.global_model_cid;
                session.round_history.push(round);
            }
        }

        // Check convergence and decide whether to start next round
        match aggregation_result.convergence_status {
            ConvergenceStatus::Continue => {
                let next_round = {
                    let sessions = self.active_sessions.read().unwrap();
                    let session = sessions.get(session_id).unwrap();
                    session.round_history.len() as u32
                };
                
                if next_round < self.config.max_rounds_per_session {
                    self.start_round(session_id, next_round).await?;
                } else {
                    self.complete_session(session_id, ConvergenceStatus::MaxRoundsReached).await?;
                }
            }
            _ => {
                self.complete_session(session_id, aggregation_result.convergence_status).await?;
            }
        }

        Ok(())
    }

    /// Perform the actual model aggregation.
    async fn perform_aggregation(&self, session_id: &str) -> Result<AggregationResult, CommonError> {
        debug!("[FederatedLearning] Performing aggregation for session {}", session_id);

        // In a real implementation, this would:
        // 1. Retrieve all model updates from DAG
        // 2. Apply the configured aggregation strategy
        // 3. Validate the aggregated model
        // 4. Store the new global model in DAG
        // 5. Compute quality metrics

        // For now, simulate aggregation
        tokio::time::sleep(Duration::from_millis(50)).await;

        let aggregated_model_cid = Cid::new_v1_sha256(
            0x55, 
            format!("aggregated_model_{}_{}", session_id, SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
            ).as_bytes()
        );

        let result = AggregationResult {
            global_model_cid: aggregated_model_cid,
            included_participants: vec![], // Would be populated from actual updates
            excluded_participants: vec![], // Would include any rejected updates
            quality_metrics: AggregationQualityMetrics {
                global_accuracy: 0.85,     // Simulated
                global_loss: 0.15,         // Simulated
                contribution_variance: 0.05, // Simulated
                anomalous_contributions: 0,
                consensus_score: 0.95,     // Simulated
            },
            convergence_status: if session_id.contains("converged") {
                ConvergenceStatus::ConvergedAccuracy
            } else {
                ConvergenceStatus::Continue
            },
        };

        info!("[FederatedLearning] Aggregation completed for session {} with {} accuracy", 
              session_id, result.quality_metrics.global_accuracy);

        Ok(result)
    }

    /// Complete a federated learning session.
    async fn complete_session(
        &self,
        session_id: &str,
        convergence_status: ConvergenceStatus,
    ) -> Result<(), CommonError> {
        info!("[FederatedLearning] Completing session {} with status {:?}", 
              session_id, convergence_status);

        {
            let mut sessions = self.active_sessions.write().unwrap();
            if let Some(session) = sessions.get_mut(session_id) {
                session.status = SessionStatus::Completed;
            }
            // Keep session for result retrieval, or remove after some time
        }

        Ok(())
    }

    /// Get session status and results.
    pub fn get_session_status(&self, session_id: &str) -> Option<SessionStatus> {
        let sessions = self.active_sessions.read().unwrap();
        sessions.get(session_id).map(|s| s.status.clone())
    }

    /// Cancel a federated learning session.
    pub async fn cancel_session(&self, session_id: &str) -> Result<(), CommonError> {
        info!("[FederatedLearning] Cancelling session {}", session_id);

        let mut sessions = self.active_sessions.write().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SessionStatus::Failed("Cancelled by coordinator".to_string());
        }

        Ok(())
    }

    /// Get coordinator capacity.
    pub fn get_capacity(&self) -> (usize, u32) {
        let sessions = self.active_sessions.read().unwrap();
        (sessions.len(), self.config.max_concurrent_sessions)
    }
}

/// Utility functions for federated learning.
impl ModelSpec {
    /// Create a simple neural network model specification.
    pub fn simple_neural_network(input_size: u32, output_size: u32) -> Self {
        Self {
            architecture: ModelArchitecture::NeuralNetwork {
                layers: vec![
                    LayerSpec {
                        layer_type: LayerType::Dense,
                        units: input_size,
                        parameters: HashMap::new(),
                    },
                    LayerSpec {
                        layer_type: LayerType::Dense,
                        units: 64,
                        parameters: HashMap::new(),
                    },
                    LayerSpec {
                        layer_type: LayerType::Dense,
                        units: output_size,
                        parameters: HashMap::new(),
                    },
                ],
                activation: ActivationFunction::ReLU,
                loss_function: LossFunction::CrossEntropy,
            },
            parameters: vec![0u8; 1024], // Placeholder parameters
            metadata: ModelMetadata {
                name: "SimpleNN".to_string(),
                version: "1.0".to_string(),
                created_at: SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                updated_at: SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                metrics: HashMap::new(),
                dataset_info: DatasetInfo {
                    size: 10000,
                    feature_dimensions: vec![input_size],
                    target_info: TargetInfo::Classification {
                        classes: vec!["class1".to_string(), "class2".to_string()],
                    },
                    distribution: DataDistribution {
                        is_iid: true,
                        skewness: 0.1,
                        demographics: HashMap::new(),
                    },
                },
            },
            hyperparameters: TrainingHyperparameters {
                learning_rate: 0.001,
                batch_size: 32,
                local_epochs: 1,
                optimizer: OptimizerType::Adam { beta1: 0.9, beta2: 0.999 },
                max_rounds: 100,
                convergence_criteria: ConvergenceCriteria {
                    min_improvement: 0.001,
                    patience: 10,
                    target_accuracy: Some(0.95),
                    max_training_time: Some(Duration::from_secs(3600)), // 1 hour
                },
            },
            privacy_config: PrivacyConfig {
                differential_privacy: Some(DifferentialPrivacyConfig {
                    epsilon: 1.0,
                    delta: 1e-5,
                    noise_mechanism: NoiseMechanism::Gaussian,
                    clipping_threshold: 1.0,
                }),
                secure_aggregation: true,
                homomorphic_encryption: None,
                min_participants: 3,
                max_data_sharing: DataSharingLevel::ModelUpdatesOnly,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Did;
    use std::str::FromStr;

    #[test]
    fn test_model_spec_creation() {
        let model = ModelSpec::simple_neural_network(784, 10);
        
        match &model.architecture {
            ModelArchitecture::NeuralNetwork { layers, .. } => {
                assert_eq!(layers.len(), 3);
                assert_eq!(layers[0].units, 784);
                assert_eq!(layers[2].units, 10);
            }
            _ => panic!("Expected neural network architecture"),
        }

        assert_eq!(model.hyperparameters.learning_rate, 0.001);
        assert!(model.privacy_config.differential_privacy.is_some());
    }

    #[test]
    fn test_federated_participant_creation() {
        let participant = FederatedParticipant {
            did: Did::from_str("did:key:participant1").unwrap(),
            dataset_size: 1000,
            compute_capacity: Resources {
                cpu_cores: 4,
                memory_mb: 8192,
                storage_mb: 10240,
            },
            reputation: 0.85,
            region: Some("US-West".to_string()),
            participation_history: ParticipationHistory {
                total_rounds: 10,
                successful_rounds: 9,
                avg_contribution_quality: 0.90,
                reliability: 0.90,
            },
        };

        assert_eq!(participant.dataset_size, 1000);
        assert_eq!(participant.reputation, 0.85);
        assert_eq!(participant.participation_history.reliability, 0.90);
    }

    #[tokio::test]
    async fn test_federated_learning_coordinator() {
        let coordinator_did = Did::from_str("did:key:coordinator").unwrap();
        let signing_key = Arc::new(icn_identity::generate_ed25519_keypair().0);
        let config = FederatedConfig {
            max_concurrent_sessions: 5,
            default_round_timeout: Duration::from_secs(300),
            min_global_participants: 2,
            max_rounds_per_session: 50,
        };

        let coordinator = FederatedLearningCoordinator::new(coordinator_did, signing_key, config);

        let model = ModelSpec::simple_neural_network(784, 10);
        let participants = vec![
            FederatedParticipant {
                did: Did::from_str("did:key:participant1").unwrap(),
                dataset_size: 1000,
                compute_capacity: Resources::default(),
                reputation: 0.8,
                region: None,
                participation_history: ParticipationHistory {
                    total_rounds: 5,
                    successful_rounds: 5,
                    avg_contribution_quality: 0.85,
                    reliability: 1.0,
                },
            },
            FederatedParticipant {
                did: Did::from_str("did:key:participant2").unwrap(),
                dataset_size: 1500,
                compute_capacity: Resources::default(),
                reputation: 0.9,
                region: None,
                participation_history: ParticipationHistory {
                    total_rounds: 8,
                    successful_rounds: 7,
                    avg_contribution_quality: 0.88,
                    reliability: 0.875,
                },
            },
        ];

        let result = coordinator.start_session("test_session".to_string(), model, participants).await;
        assert!(result.is_ok());

        let (active, max) = coordinator.get_capacity();
        assert_eq!(active, 1);
        assert_eq!(max, 5);
    }

    #[test]
    fn test_training_round_creation() {
        let round = FederatedTrainingRound {
            round_number: 0,
            global_model_cid: Cid::new_v1_sha256(0x55, b"test_model"),
            participants: vec![],
            round_config: RoundConfig {
                target_participants: 5,
                min_participants: 3,
                local_epochs: 1,
                aggregation_strategy: AggregationStrategy::FederatedAveraging,
                quality_thresholds: QualityThresholds {
                    min_accuracy_improvement: 0.001,
                    max_loss_increase: 0.1,
                    min_gradient_norm: 0.001,
                    max_gradient_norm: 10.0,
                },
            },
            deadline: SystemTime::now() + Duration::from_secs(300),
            status: RoundStatus::Initializing,
        };

        assert_eq!(round.round_number, 0);
        assert!(matches!(round.status, RoundStatus::Initializing));
        assert!(matches!(round.round_config.aggregation_strategy, AggregationStrategy::FederatedAveraging));
    }

    #[test]
    fn test_model_update_validation() {
        let coordinator_did = Did::from_str("did:key:coordinator").unwrap();
        let signing_key = Arc::new(icn_identity::generate_ed25519_keypair().0);
        let config = FederatedConfig {
            max_concurrent_sessions: 5,
            default_round_timeout: Duration::from_secs(300),
            min_global_participants: 2,
            max_rounds_per_session: 50,
        };

        let coordinator = FederatedLearningCoordinator::new(coordinator_did, signing_key, config);

        let valid_update = ModelUpdate {
            participant_did: Did::from_str("did:key:participant").unwrap(),
            round_number: 0,
            parameters: vec![1, 2, 3, 4], // Non-empty parameters
            training_metadata: TrainingMetadata {
                local_samples: 100,
                local_loss: 0.5,
                local_accuracy: Some(0.85),
                training_time_ms: 1000,
                resources_used: Resources::default(),
                gradient_norms: vec![0.1, 0.2, 0.15],
            },
            proof_of_training: None,
            signature: SignatureBytes(vec![]),
        };

        assert!(coordinator.validate_model_update(&valid_update).is_ok());

        let invalid_update = ModelUpdate {
            parameters: vec![], // Empty parameters
            training_metadata: TrainingMetadata {
                local_samples: 0, // No samples
                gradient_norms: vec![1001.0], // Out of bounds gradient
                ..valid_update.training_metadata.clone()
            },
            ..valid_update.clone()
        };

        assert!(coordinator.validate_model_update(&invalid_update).is_err());
    }

    #[test]
    fn test_privacy_config() {
        let config = PrivacyConfig {
            differential_privacy: Some(DifferentialPrivacyConfig {
                epsilon: 1.0,
                delta: 1e-5,
                noise_mechanism: NoiseMechanism::Gaussian,
                clipping_threshold: 1.0,
            }),
            secure_aggregation: true,
            homomorphic_encryption: None,
            min_participants: 5,
            max_data_sharing: DataSharingLevel::ModelUpdatesOnly,
        };

        assert!(config.differential_privacy.is_some());
        assert!(config.secure_aggregation);
        assert_eq!(config.min_participants, 5);
        assert!(matches!(config.max_data_sharing, DataSharingLevel::ModelUpdatesOnly));
    }
}