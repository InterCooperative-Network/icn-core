use icn_network::error::MeshNetworkError;
use icn_runtime::context::HostAbiError;
use icn_runtime::error::MeshJobError;

#[test]
fn host_abi_network_error_maps_to_connection_failed() {
    let err = HostAbiError::NetworkError("disconnected".to_string());
    let mesh_err: MeshJobError = err.into();
    match mesh_err {
        MeshJobError::Network(MeshNetworkError::ConnectionFailed {
            peer_id: None,
            cause,
        }) => {
            assert_eq!(cause, "disconnected");
        }
        other => panic!("Unexpected variant: {other:?}"),
    }
}

#[test]
fn host_abi_invalid_params_maps_to_invalid_spec() {
    let err = HostAbiError::InvalidParameters("bad".to_string());
    let mesh_err: MeshJobError = err.into();
    match mesh_err {
        MeshJobError::InvalidSpec {
            job_id: None,
            reason,
        } => {
            assert_eq!(reason, "bad");
        }
        _ => panic!("Unexpected mapping: {mesh_err:?}"),
    }
}

#[test]
fn host_abi_signature_error_maps_to_signature_error() {
    let err = HostAbiError::SignatureError("bad sig".to_string());
    let mesh_err: MeshJobError = err.into();
    match mesh_err {
        MeshJobError::SignatureError {
            job_id: None,
            reason,
        } => {
            assert_eq!(reason, "bad sig");
        }
        _ => panic!("Unexpected mapping: {mesh_err:?}"),
    }
}

#[test]
fn host_abi_dag_error_maps_to_dag_operation_failed() {
    let err = HostAbiError::DagOperationFailed("write failed".to_string());
    let mesh_err: MeshJobError = err.into();
    match mesh_err {
        MeshJobError::DagOperationFailed {
            job_id: None,
            reason,
        } => {
            assert_eq!(reason, "write failed");
        }
        _ => panic!("Unexpected mapping: {mesh_err:?}"),
    }
}

#[test]
fn host_abi_permission_denied_maps() {
    let err = HostAbiError::PermissionDenied("nope".to_string());
    let mesh_err: MeshJobError = err.into();
    match mesh_err {
        MeshJobError::PermissionDenied { reason, .. } => {
            assert_eq!(reason, "nope");
        }
        _ => panic!("Unexpected mapping: {mesh_err:?}"),
    }
}
