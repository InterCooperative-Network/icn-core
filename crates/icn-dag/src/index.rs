use icn_common::{Cid, DagBlock};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct DagTraversalIndex {
    adjacency: HashMap<Cid, Vec<Cid>>,
}

impl DagTraversalIndex {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    pub fn index_block(&mut self, block: &DagBlock) {
        self.adjacency.insert(
            block.cid.clone(),
            block.links.iter().map(|l| l.cid.clone()).collect(),
        );
    }

    pub fn remove_block(&mut self, cid: &Cid) {
        self.adjacency.remove(cid);
        for children in self.adjacency.values_mut() {
            children.retain(|c| c != cid);
        }
    }

    pub fn traverse(&self, start: &Cid) -> Vec<Cid> {
        let mut visited = HashSet::new();
        let mut stack = vec![start.clone()];
        let mut order = Vec::new();
        while let Some(cid) = stack.pop() {
            if visited.insert(cid.clone()) {
                order.push(cid.clone());
                if let Some(children) = self.adjacency.get(&cid) {
                    for child in children.iter().rev() {
                        stack.push(child.clone());
                    }
                }
            }
        }
        order
    }
}
