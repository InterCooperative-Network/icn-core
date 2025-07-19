/// Utilities for encoding and decoding simple `Result<Integer>` values
/// exchanged with WASM modules.
///
/// The value is stored in a single `i64` where the high 32 bits contain the
/// variant tag (`0` for `Ok`, `1` for `Err`) and the low 32 bits contain the
/// associated `i32` value.

pub fn encode_result_i32(res: Result<i32, i32>) -> i64 {
    match res {
        Ok(v) => ((0u64 << 32) | (v as u32 as u64)) as i64,
        Err(e) => ((1u64 << 32) | (e as u32 as u64)) as i64,
    }
}

pub fn decode_result_i32(val: i64) -> Result<i32, i32> {
    let tag = ((val >> 32) & 0xFFFF_FFFF) as u32;
    let data = (val & 0xFFFF_FFFF) as u32 as i32;
    if tag == 0 {
        Ok(data)
    } else {
        Err(data)
    }
}
