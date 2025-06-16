//! Defines the Host ABI function indices for the ICN runtime.

// TODO: Finalize all ABI indices once the full specification is stable.
// These are initial assignments based on current understanding.

/// ABI Index for `host_account_get_mana`.
/// Retrieves the mana balance for the current account/identity.
pub const ABI_HOST_ACCOUNT_GET_MANA: u32 = 10;

/// ABI Index for `host_account_spend_mana`.
/// Attempts to spend mana from the current account/identity.
pub const ABI_HOST_ACCOUNT_SPEND_MANA: u32 = 11; // Example, TBD

/// ABI Index for `host_submit_mesh_job`.
/// Submits a job to the ICN mesh network.
pub const ABI_HOST_SUBMIT_MESH_JOB: u32 = 16;

/// ABI Index for `host_get_pending_mesh_jobs`.
/// Retrieves pending mesh jobs from the runtime.
pub const ABI_HOST_GET_PENDING_MESH_JOBS: u32 = 22; // New

/// ABI Index for `host_anchor_receipt`.
/// Anchors an execution receipt to the DAG and updates reputation.
pub const ABI_HOST_ANCHOR_RECEIPT: u32 = 23; // New

// --- Governance ABI Functions (RFC 0010) ---
// TODO: Finalize these indices and add others as needed.

/// ABI Index for `host_create_governance_proposal`.
/// Creates a new governance proposal.
pub const ABI_HOST_CREATE_GOVERNANCE_PROPOSAL: u32 = 17; // Was 30

/// ABI Index for `host_open_governance_voting`.
/// Opens a governance proposal for voting.
pub const ABI_HOST_OPEN_GOVERNANCE_VOTING: u32 = 18; // New

/// ABI Index for `host_cast_governance_vote`.
/// Casts a vote on an open governance proposal.
pub const ABI_HOST_CAST_GOVERNANCE_VOTE: u32 = 19; // Was 31

/// ABI Index for `host_close_voting_and_verify`.
/// Closes voting on a proposal, triggers tallying/verification.
pub const ABI_HOST_CLOSE_VOTING_AND_VERIFY: u32 = 20; // Was 32, renamed from ABI_HOST_CLOSE_GOVERNANCE_PROPOSAL_VOTING

/// ABI Index for `host_execute_governance_proposal`.
/// Triggers the execution of an accepted governance proposal.
pub const ABI_HOST_EXECUTE_GOVERNANCE_PROPOSAL: u32 = 21; // Was 33

// TODO: Consider ABI functions for opening voting, distributing for deliberation if they are direct host calls.
// Some lifecycle steps might be managed internally by the GovernanceModule based on time or other triggers.

// TODO: Add constants for other ABI functions (0-15, and 17+ as they are defined).
// Examples:
// pub const ABI_HOST_LOG_MESSAGE: u32 = 1;
// pub const ABI_HOST_GET_BLOCK_INFO: u32 = 2;
// pub const ABI_HOST_PUT_DAG_BLOCK: u32 = 3;
// ...
