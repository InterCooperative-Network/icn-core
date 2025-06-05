# ICN Mesh Job Lifecycle

This document outlines the expected flow of a job through the ICN mesh networking system,
from submission by a host to final completion and receipt anchoring.

## Key Components

*   **Host Runtime (`icn-runtime`)**: The environment where user code (hosts, executors) runs. Provides `RuntimeContext`.
*   **`RuntimeContext`**: Holds state for a node, including identity, mana, job states, and access to services.
*   **`JobManager` (within `RuntimeContext`)**: A background task responsible for managing the lifecycle of mesh jobs initiated or processed by its `RuntimeContext`.
*   **`MeshNetworkService` (trait in `icn-runtime`)**: An abstraction for mesh-specific network operations (announce, bid, assign, receipt).
    *   **`DefaultMeshNetworkService` (impl in `icn-runtime`)**: Concrete implementation using `Libp2pNetworkService`.
*   **`NetworkService` (trait in `icn-network`)**: Lower-level P2P networking abstraction.
    *   **`Libp2pNetworkService` (impl in `icn-network`)**: Concrete libp2p-based implementation.
*   **`NetworkMessage` (enum in `icn-network`)**: Defines the types of messages exchanged over the network.

## Job Lifecycle Stages

1.  **Job Submission (by Host)**
    *   A host calls `host_submit_mesh_job` (from `icn-runtime`) with job details (manifest, cost).
    *   This function internally interacts with the host's `RuntimeContext`.
    *   Mana is deducted from the host.
    *   The job (`ActualMeshJob`) is added to the `pending_mesh_jobs` queue within the `RuntimeContext`.
    *   The job's state is set to `JobState::Pending` in `job_states`.

2.  **Job Announcement (by JobManager on Submitter's Node)**
    *   The `JobManager` (running within the submitter's `RuntimeContext`) picks up the job from `pending_mesh_jobs`.
    *   It calls `MeshNetworkService::announce_job` with a `MeshJobAnnounce` payload.
    *   `DefaultMeshNetworkService` translates this into a `NetworkMessage::MeshJobAnnouncement(Job)` and uses its underlying `Libp2pNetworkService` (specifically `broadcast_message`) to send it over the gossipsub topic.

3.  **Job Discovery & Bid Submission (by Potential Executors)**
    *   Executor nodes are running their own `RuntimeContext` and `Libp2pNetworkService`.
    *   Their `Libp2pNetworkService` is subscribed to the gossipsub topic.
    *   Upon receiving a `NetworkMessage::MeshJobAnnouncement`, an interested executor:
        *   Evaluates the job.
        *   If willing to execute, constructs a `MeshJobBid`.
        *   Submits the bid by creating a `NetworkMessage::BidSubmission(Bid)` and broadcasting it via its `Libp2pNetworkService`.

4.  **Bid Collection & Executor Selection (by JobManager on Submitter's Node)**
    *   The `JobManager` on the submitter's node calls `MeshNetworkService::collect_bids_for_job` for a defined period.
    *   `DefaultMeshNetworkService` subscribes to its `Libp2pNetworkService` for incoming messages.
    *   It filters for `NetworkMessage::BidSubmission` messages matching the `JobId`.
    *   After the bidding window, the `JobManager` validates bids (e.g., mana checks, reputation).
    *   It uses a `SelectionPolicy` (e.g., `select_executor` from `icn-mesh`) to choose the best executor from valid bids.

5.  **Job Assignment (by JobManager on Submitter's Node)**
    *   If an executor is selected:
        *   The `JobManager` updates the job's state to `JobState::Assigned { executor_did }`.
        *   It calls `MeshNetworkService::broadcast_assignment` with a `JobAssignmentNotice`.
        *   `DefaultMeshNetworkService` translates this into `NetworkMessage::JobAssignmentNotification(JobId, ExecutorDid)` and broadcasts it.
    *   The assigned executor node receives this notification and prepares to execute the job.

6.  **Job Execution & Receipt Creation (by Assigned Executor)**
    *   The assigned executor performs the job's work.
    *   Upon completion, it creates an `ExecutionReceipt` containing the `JobId`, its own `Did`, the `result_cid`, and other metrics.
    *   The executor's `RuntimeContext` (via a host ABI call like `host_anchor_receipt` or an internal equivalent) signs this receipt. The signature is stored in `receipt.sig`.

7.  **Receipt Submission (by Assigned Executor)**
    *   The executor node sends the signed `ExecutionReceipt` back to the network.
    *   This is done by creating a `NetworkMessage::SubmitReceipt(ExecutionReceipt)` and broadcasting it via its `Libp2pNetworkService`.

8.  **Receipt Processing & Anchoring (by JobManager on Submitter's Node)**
    *   The `JobManager` on the submitter's node periodically calls `MeshNetworkService::try_receive_receipt`.
    *   `DefaultMeshNetworkService` again uses its subscription to `Libp2pNetworkService` to look for `NetworkMessage::SubmitReceipt`.
    *   When a receipt for a managed job is received:
        *   The `JobManager` calls its `RuntimeContext::anchor_receipt`.
        *   `anchor_receipt` verifies:
            *   The job exists and was assigned to the claiming executor.
            *   The signature on the receipt is valid for the claiming executor.
        *   If valid, the full receipt is stored in the DAG via `DagStore::put`.
        *   The job's state is updated to `JobState::Completed { receipt }`.
        *   Reputation updates may be triggered.

9.  **Handling Failures & Timeouts**
    *   **No Bids**: Job may be re-queued or marked as failed.
    *   **Execution Timeout**: If an assigned executor doesn't submit a receipt in time, the `JobManager` marks the job `JobState::Failed` and may refund the submitter.
    *   **Invalid Receipt**: If a receipt is invalid (wrong executor, bad signature), it's rejected, and the job might eventually time out or be marked failed.

This lifecycle relies on asynchronous P2P communication and assumes nodes are discoverable and can exchange messages via the underlying libp2p gossipsub and Kademlia DHT mechanisms.

### Logging Reference

To aid in debugging and understanding the flow of mesh operations, the ICN crates utilize structured logging. You can enable this logging by setting the `RUST_LOG` environment variable.

Example:
`RUST_LOG=icn_network=debug,icn_runtime=debug,icn_mesh=debug`

Common log prefixes and their meanings:

-   **`[libp2p_service][mesh-job]`**: Events specifically within the `libp2p_service` module in `icn-network` related to mesh job processing (e.g., gossipsub subscriptions, message broadcasting, Kademlia operations for discovery/records).
-   **`[DefaultMeshNetworkService]`**: Events from the `DefaultMeshNetworkService` in `icn-runtime`, which acts as an adapter between the runtime's mesh logic and the underlying `Libp2pNetworkService`. Covers job announcements, bid collection, assignment broadcasts, and receipt reception attempts.
-   **`[JobManager]`**: Events from the core mesh job manager task within `icn-runtime` (in `context.rs`). Tracks the lifecycle of a job from pending, to bid collection, assignment, and completion/failure, including interactions with the network service and mana ledger.
-   **`[CONTEXT]`**: General events from the `RuntimeContext` in `icn-runtime`, often related to mana operations, internal job queuing, or receipt anchoring that might be invoked by the job manager or other host functions.
-   **`[test-mesh-network]`**: Logging from integration tests in `icn-network` (e.g., `libp2p_mesh_integration.rs`) focusing on network-level interactions between peer services.
-   **`[test-mesh-runtime]`**: Logging from integration tests in `icn-runtime` (e.g., `mesh.rs`) focusing on the end-to-end job lifecycle involving the runtime context and job manager.
-   **`[libp2p]`**: General low-level messages from the libp2p framework itself (e.g., connection events, Kademlia protocol messages, ping results). These are often verbose but can be crucial for diagnosing fundamental network issues.

By filtering or observing these prefixes, developers can trace the progression of jobs and messages through the different layers of the ICN system. 