//! Defines the Host ABI function indices for the ICN runtime.
//!
//! The values below follow the finalized Host ABI specification. Each constant
//! corresponds to a function exposed by the runtime to WASM modules.

/// ABI Index for `host_account_get_mana`.
/// Retrieves the mana balance for the current account/identity.
pub const ABI_HOST_ACCOUNT_GET_MANA: u32 = 10;

/// ABI Index for `host_account_spend_mana`.
/// Attempts to spend mana from the current account/identity.
pub const ABI_HOST_ACCOUNT_SPEND_MANA: u32 = 11;

/// ABI Index for `host_account_credit_mana`.
/// Credits mana to the specified account or identity.
pub const ABI_HOST_ACCOUNT_CREDIT_MANA: u32 = 12;

/// ABI Index for `host_submit_mesh_job`.
/// Submits a job to the ICN mesh network.
pub const ABI_HOST_SUBMIT_MESH_JOB: u32 = 16;

/// ABI Index for `host_get_pending_mesh_jobs`.
/// Retrieves pending mesh jobs from the runtime.
pub const ABI_HOST_GET_PENDING_MESH_JOBS: u32 = 22;

/// ABI Index for `host_anchor_receipt`.
/// Anchors an execution receipt to the DAG and updates reputation.
pub const ABI_HOST_ANCHOR_RECEIPT: u32 = 23;

/// ABI Index for `host_get_reputation`.
/// Retrieves the reputation score for the given DID.
pub const ABI_HOST_GET_REPUTATION: u32 = 24;

/// ABI Index for `host_verify_zk_proof`.
/// Verifies a [`ZkCredentialProof`](icn_common::ZkCredentialProof)
/// provided by the caller.
pub const ABI_HOST_VERIFY_ZK_PROOF: u32 = 25;

/// ABI Index for `host_generate_zk_proof`.
/// Creates a placeholder [`ZkCredentialProof`](icn_common::ZkCredentialProof)
/// using the supplied parameters.
pub const ABI_HOST_GENERATE_ZK_PROOF: u32 = 26;

// --- Governance ABI Functions (RFC 0010) ---
// Indices reserved for governance operations defined in RFC 0010.

/// ABI Index for `host_create_governance_proposal`.
/// Creates a new governance proposal.
pub const ABI_HOST_CREATE_GOVERNANCE_PROPOSAL: u32 = 17;

/// ABI Index for `host_open_governance_voting`.
/// Opens a governance proposal for voting.
pub const ABI_HOST_OPEN_GOVERNANCE_VOTING: u32 = 18;

/// ABI Index for `host_cast_governance_vote`.
/// Casts a vote on an open governance proposal.
pub const ABI_HOST_CAST_GOVERNANCE_VOTE: u32 = 19;

/// ABI Index for `host_close_voting_and_verify`.
/// Closes voting on a proposal, triggers tallying/verification.
pub const ABI_HOST_CLOSE_VOTING_AND_VERIFY: u32 = 20;

/// ABI Index for `host_execute_governance_proposal`.
/// Triggers the execution of an accepted governance proposal.
pub const ABI_HOST_EXECUTE_GOVERNANCE_PROPOSAL: u32 = 21;

// Additional governance lifecycle functions may be added in future revisions.
// Some steps might be handled internally by the `GovernanceModule` based on
// timing or other triggers.

// Additional host ABI constants for indices 0–9 and future functions will be
// introduced as they are specified. Examples include:
// pub const ABI_HOST_LOG_MESSAGE: u32 = 1;
// pub const ABI_HOST_GET_BLOCK_INFO: u32 = 2;
// pub const ABI_HOST_PUT_DAG_BLOCK: u32 = 3;
// ...
