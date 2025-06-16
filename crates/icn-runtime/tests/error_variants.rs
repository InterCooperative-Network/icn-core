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
