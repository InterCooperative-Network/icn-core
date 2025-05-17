#![doc = include_str!("../README.md")]

//! # ICN Runtime Crate
//! This crate provides the execution environment for InterCooperative Network (ICN) logic, 
//! possibly including WebAssembly (WASM) runtimes and host interaction capabilities.
//! It focuses on a secure, performant, and modular execution environment with well-defined host functions.

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
