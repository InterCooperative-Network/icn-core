#![doc = include_str!("../README.md")]

//! # ICN Identity Crate
//! This crate manages decentralized identities (DIDs), verifiable credentials (VCs),
//! and cryptographic operations for the InterCooperative Network (ICN).
//! It focuses on security, interoperability with DID/VC standards, and usability.

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
