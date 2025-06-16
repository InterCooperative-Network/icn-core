use icn_common::{Cid, CommonError, DagBlock, Transaction};
use serde::{Deserialize, Serialize};

/// Request to submit a transaction to a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub transaction: Transaction,
}

/// Response returned after a transaction submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionResponse {
    pub tx_id: String,
    pub accepted: bool,
}

/// Request to query data by CID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQueryRequest {
    pub cid: Cid,
}

/// Response containing a DAG block, if found.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQueryResponse {
    pub block: Option<DagBlock>,
}

/// Trait describing transaction-related API operations.
pub trait TransactionApi {
    fn submit_transaction(
        &self,
        req: SubmitTransactionRequest,
    ) -> Result<SubmitTransactionResponse, CommonError>;

    fn query_data(&self, req: DataQueryRequest) -> Result<DataQueryResponse, CommonError>;
}
