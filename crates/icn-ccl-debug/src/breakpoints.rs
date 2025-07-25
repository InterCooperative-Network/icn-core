use std::collections::{HashMap, HashSet};

pub struct BreakpointManager {
    breakpoints: HashMap<String, HashSet<u32>>, // session_id -> line numbers
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
        }
    }
    
    pub fn add_breakpoint(&mut self, session_id: &str, line: u32) {
        self.breakpoints
            .entry(session_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(line);
    }
    
    pub fn remove_breakpoint(&mut self, session_id: &str, line: u32) {
        if let Some(session_breakpoints) = self.breakpoints.get_mut(session_id) {
            session_breakpoints.remove(&line);
        }
    }
    
    pub fn has_breakpoint(&self, session_id: &str, line: u32) -> bool {
        self.breakpoints
            .get(session_id)
            .map(|bp| bp.contains(&line))
            .unwrap_or(false)
    }
    
    pub fn get_breakpoints(&self, session_id: &str) -> Vec<u32> {
        self.breakpoints
            .get(session_id)
            .map(|bp| bp.iter().copied().collect())
            .unwrap_or_default()
    }
    
    pub fn clear_session_breakpoints(&mut self, session_id: &str) {
        self.breakpoints.remove(session_id);
    }
}