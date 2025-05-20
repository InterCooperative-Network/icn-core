"""//! Defines the Host ABI function indices for the ICN runtime.

// TODO: Finalize all ABI indices once the full specification is stable.
// These are initial assignments based on current understanding.

/// ABI Index for `host_account_get_mana`.
/// Retrieves the mana balance for the current account/identity.
pub const ABI_HOST_ACCOUNT_GET_MANA: u32 = 10;

/// ABI Index for `host_account_spend_mana`.
/// Attempts to spend mana from the current account/identity.
/// TODO: Define this function and its parameters.
pub const ABI_HOST_ACCOUNT_SPEND_MANA: u32 = 11; // Example, TBD

/// ABI Index for `host_submit_mesh_job`.
/// Submits a job to the ICN mesh network.
pub const ABI_HOST_SUBMIT_MESH_JOB: u32 = 16;

// TODO: Add constants for other ABI functions (0-15, and 17+ as they are defined).
// Examples:
// pub const ABI_HOST_LOG_MESSAGE: u32 = 1;
// pub const ABI_HOST_GET_BLOCK_INFO: u32 = 2;
// pub const ABI_HOST_PUT_DAG_BLOCK: u32 = 3;
// ...
"" 