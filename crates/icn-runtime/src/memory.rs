use std::sync::Arc;
use wasmtime::{Caller, Memory};

use crate::{context::RuntimeContext, HostAbiError};

/// Retrieves the guest memory from the caller.
fn get_memory(caller: &mut Caller<'_, Arc<RuntimeContext>>) -> Result<Memory, HostAbiError> {
    caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| HostAbiError::InvalidParameters("memory export missing".into()))
}

/// Reads a slice of bytes from guest memory.
pub fn read_bytes(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, HostAbiError> {
    let memory = get_memory(caller)?;
    let mut buf = vec![0u8; len as usize];
    memory
        .read(caller, ptr as usize, &mut buf)
        .map_err(|e| HostAbiError::InvalidParameters(format!("memory read failed: {e}")))?;
    Ok(buf)
}

/// Convenience alias for [`read_bytes`].
pub fn read_guest_bytes(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, HostAbiError> {
    read_bytes(caller, ptr, len)
}

/// Reads a UTF-8 string from guest memory.
pub fn read_string(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
) -> Result<String, HostAbiError> {
    let bytes = read_bytes(caller, ptr, len)?;
    String::from_utf8(bytes)
        .map_err(|e| HostAbiError::InvalidParameters(format!("utf8 error: {e}")))
}

/// Convenience alias for [`read_string`].
pub fn read_guest_string(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
) -> Result<String, HostAbiError> {
    read_string(caller, ptr, len)
}

/// Writes bytes into guest memory at the given pointer.
pub fn write_bytes(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    data: &[u8],
) -> Result<(), HostAbiError> {
    let memory = get_memory(caller)?;
    memory
        .write(caller, ptr as usize, data)
        .map_err(|e| HostAbiError::InvalidParameters(format!("memory write failed: {e}")))
}

/// Convenience alias for [`write_bytes`].
pub fn write_guest_bytes(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    data: &[u8],
) -> Result<(), HostAbiError> {
    write_bytes(caller, ptr, data)
}

/// Writes a UTF-8 string into guest memory at the given pointer.
pub fn write_string(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    data: &str,
) -> Result<(), HostAbiError> {
    write_bytes(caller, ptr, data.as_bytes())
}

/// Convenience alias for [`write_string`].
pub fn write_guest_string(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    data: &str,
) -> Result<(), HostAbiError> {
    write_string(caller, ptr, data)
}
