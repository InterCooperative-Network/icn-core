#![doc = include_str!("../README.md")]

//! # ICN Common Crate
//! This crate provides common data structures, types, utilities, and error definitions
//! shared across multiple InterCooperative Network (ICN) core crates. It aims to
//! reduce code duplication, ensure consistency, and simplify dependencies.

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
