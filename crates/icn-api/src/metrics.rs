use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;

/// Registers common ICN metrics (job execution, governance actions, network health)
/// to the provided registry.
pub fn register_core_metrics(registry: &mut Registry) {
    use icn_dag::metrics::{DAG_GET_CALLS, DAG_PUT_CALLS};
    use icn_governance::metrics::{CAST_VOTE_CALLS, EXECUTE_PROPOSAL_CALLS, SUBMIT_PROPOSAL_CALLS};
    use icn_network::metrics::{
        BYTES_RECEIVED_TOTAL, BYTES_SENT_TOTAL, KADEMLIA_PEERS_GAUGE, MESSAGES_RECEIVED_TOTAL,
        MESSAGES_SENT_TOTAL, PEER_COUNT_GAUGE, PING_AVG_RTT_MS, PING_LAST_RTT_MS, PING_MAX_RTT_MS,
        PING_MIN_RTT_MS,
    };

    #[cfg(feature = "runtime-metrics")]
    use icn_economics::metrics::{CREDIT_MANA_CALLS, GET_BALANCE_CALLS, SPEND_MANA_CALLS};
    #[cfg(feature = "runtime-metrics")]
    use icn_mesh::metrics::{
        JOB_PROCESS_TIME, PENDING_JOBS_GAUGE, SCHEDULE_MESH_JOB_CALLS, SELECT_EXECUTOR_CALLS,
    };

    #[cfg(feature = "runtime-metrics")]
    {
        registry.register(
            "economics_get_balance_calls",
            "Number of mana get_balance calls",
            GET_BALANCE_CALLS.clone(),
        );
        registry.register(
            "economics_spend_mana_calls",
            "Number of mana spend_mana calls",
            SPEND_MANA_CALLS.clone(),
        );
        registry.register(
            "economics_credit_mana_calls",
            "Number of mana credit_mana calls",
            CREDIT_MANA_CALLS.clone(),
        );
    }
    registry.register(
        "governance_submit_proposal_calls",
        "Number of submit_proposal calls",
        SUBMIT_PROPOSAL_CALLS.clone(),
    );
    registry.register(
        "governance_cast_vote_calls",
        "Number of cast_vote calls",
        CAST_VOTE_CALLS.clone(),
    );
    registry.register(
        "governance_execute_proposal_calls",
        "Number of execute_proposal calls",
        EXECUTE_PROPOSAL_CALLS.clone(),
    );
    registry.register(
        "dag_put_calls",
        "Number of DAG put calls",
        DAG_PUT_CALLS.clone(),
    );
    registry.register(
        "dag_get_calls",
        "Number of DAG get calls",
        DAG_GET_CALLS.clone(),
    );
    #[cfg(feature = "runtime-metrics")]
    {
        registry.register(
            "mesh_select_executor_calls",
            "Number of select_executor calls",
            SELECT_EXECUTOR_CALLS.clone(),
        );
        registry.register(
            "mesh_schedule_job_calls",
            "Number of schedule_mesh_job calls",
            SCHEDULE_MESH_JOB_CALLS.clone(),
        );
        registry.register(
            "mesh_pending_jobs",
            "Current number of pending mesh jobs",
            PENDING_JOBS_GAUGE.clone(),
        );
        registry.register(
            "mesh_job_process_time_seconds",
            "Time from job assignment to receipt",
            JOB_PROCESS_TIME.clone(),
        );
    }
    registry.register(
        "network_peer_count",
        "Connected peers",
        PEER_COUNT_GAUGE.clone(),
    );
    registry.register(
        "network_kademlia_peers",
        "Peers in Kademlia table",
        KADEMLIA_PEERS_GAUGE.clone(),
    );
    registry.register(
        "network_bytes_sent_total",
        "Bytes sent over network",
        BYTES_SENT_TOTAL.clone(),
    );
    registry.register(
        "network_bytes_received_total",
        "Bytes received over network",
        BYTES_RECEIVED_TOTAL.clone(),
    );
    registry.register(
        "network_messages_sent_total",
        "Messages sent over network",
        MESSAGES_SENT_TOTAL.clone(),
    );
    registry.register(
        "network_messages_received_total",
        "Messages received over network",
        MESSAGES_RECEIVED_TOTAL.clone(),
    );
    registry.register(
        "ping_last_rtt_ms",
        "Last ping RTT",
        PING_LAST_RTT_MS.clone(),
    );
    registry.register("ping_min_rtt_ms", "Min ping RTT", PING_MIN_RTT_MS.clone());
    registry.register("ping_max_rtt_ms", "Max ping RTT", PING_MAX_RTT_MS.clone());
    registry.register("ping_avg_rtt_ms", "Avg ping RTT", PING_AVG_RTT_MS.clone());
}

/// Returns Prometheus metrics in text format for the registered core metrics.
pub fn collect_metrics() -> String {
    let mut registry = Registry::default();
    register_core_metrics(&mut registry);
    let mut buffer = String::new();
    encode(&mut buffer, &registry).unwrap();
    buffer
}

#[cfg(feature = "axum-endpoint")]
pub async fn metrics_handler() -> impl axum::response::IntoResponse {
    let text = collect_metrics();
    (axum::http::StatusCode::OK, text)
}
