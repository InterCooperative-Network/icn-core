#![doc = include_str!("../README.md")]

//! # ICN Network Crate
//! This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN),
//! likely using libp2p. It covers P2P communication, transport protocols, peer discovery,
//! message routing, and federation synchronization.

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
