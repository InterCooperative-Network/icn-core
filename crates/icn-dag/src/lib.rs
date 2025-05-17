#![doc = include_str!("../README.md")]

//! # ICN DAG Crate
//! This crate implements or defines interfaces for content-addressed Directed Acyclic Graph (DAG) 
//! storage and manipulation, crucial for the InterCooperative Network (ICN) data model.
//! It handles DAG primitives, content addressing, storage abstraction, and serialization formats.

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
