#![doc = include_str!("../README.md")]

//! # ICN Protocol Crate
//! This crate defines core message formats, communication protocols, and potentially helpers
//! for a domain-specific language (e.g., CCL) within the InterCooperative Network (ICN).
//! It focuses on message serialization, protocol definitions, and versioning.

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
