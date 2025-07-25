use crate::{breakpoints::BreakpointManager, inspector::ContractInspector};
use icn_ccl::{compile_ccl_source_to_wasm, ContractMetadata};
use icn_runtime::context::RuntimeContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmtime::{Engine, Module, Store};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub id: String,
    pub contract_name: String,
    pub source_path: String,
    pub wasm_bytes: Vec<u8>,
    pub metadata: ContractMetadata,
    pub state: DebugState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugState {
    NotStarted,
    Running,
    Paused { line: u32, column: u32 },
    Terminated,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub line: u32,
    pub column: u32,
    pub locals: HashMap<String, DebugValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugValue {
    Integer(i64),
    String(String),
    Boolean(bool),
    Array(Vec<DebugValue>),
    Object(HashMap<String, DebugValue>),
    Null,
}

pub struct CclDebugger {
    engine: Engine,
    sessions: Arc<Mutex<HashMap<String, DebugSession>>>,
    breakpoint_manager: BreakpointManager,
    inspector: ContractInspector,
}

impl CclDebugger {
    pub fn new() -> Self {
        let engine = Engine::default();
        
        Self {
            engine,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            breakpoint_manager: BreakpointManager::new(),
            inspector: ContractInspector::new(),
        }
    }
    
    /// Create a new debug session for a CCL contract
    pub fn create_session(
        &mut self,
        session_id: String,
        contract_name: String,
        source_path: String,
        source_code: &str,
    ) -> Result<String, String> {
        // Compile CCL to WASM
        let (wasm_bytes, metadata) = compile_ccl_source_to_wasm(source_code)
            .map_err(|e| format!("Failed to compile CCL: {:?}", e))?;
        
        let session = DebugSession {
            id: session_id.clone(),
            contract_name,
            source_path,
            wasm_bytes,
            metadata,
            state: DebugState::NotStarted,
        };
        
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(session_id.clone(), session);
        }
        
        Ok(session_id)
    }
    
    /// Set a breakpoint at a specific line
    pub fn set_breakpoint(&mut self, session_id: &str, line: u32) -> Result<(), String> {
        self.breakpoint_manager.add_breakpoint(session_id, line);
        Ok(())
    }
    
    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, session_id: &str, line: u32) -> Result<(), String> {
        self.breakpoint_manager.remove_breakpoint(session_id, line);
        Ok(())
    }
    
    /// Start execution of a debug session
    pub fn start_execution(&mut self, session_id: &str) -> Result<DebugState, String> {
        let mut sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;
        
        // Create WASM module
        let module = Module::from_binary(&self.engine, &session.wasm_bytes)
            .map_err(|e| format!("Failed to create WASM module: {}", e))?;
        
        // Start execution in paused state for step-by-step debugging
        session.state = DebugState::Paused { line: 1, column: 1 };
        
        Ok(session.state.clone())
    }
    
    /// Step to the next line of execution
    pub fn step_next(&mut self, session_id: &str) -> Result<DebugState, String> {
        let mut sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;
        
        match &session.state {
            DebugState::Paused { line, column: _ } => {
                let next_line = line + 1;
                
                // Check if we hit a breakpoint
                if self.breakpoint_manager.has_breakpoint(session_id, next_line) {
                    session.state = DebugState::Paused { line: next_line, column: 1 };
                } else {
                    // Continue execution or pause at next line for step debugging
                    session.state = DebugState::Paused { line: next_line, column: 1 };
                }
                
                Ok(session.state.clone())
            }
            _ => Err("Cannot step: session not in paused state".to_string()),
        }
    }
    
    /// Continue execution until breakpoint or completion
    pub fn continue_execution(&mut self, session_id: &str) -> Result<DebugState, String> {
        let mut sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;
        
        match &session.state {
            DebugState::Paused { line: _, column: _ } => {
                // Simulate execution until next breakpoint
                session.state = DebugState::Running;
                
                // In a real implementation, this would execute the WASM
                // and pause at the next breakpoint
                
                // For now, simulate completion
                session.state = DebugState::Terminated;
                
                Ok(session.state.clone())
            }
            _ => Err("Cannot continue: session not in paused state".to_string()),
        }
    }
    
    /// Get the current call stack
    pub fn get_call_stack(&self, session_id: &str) -> Result<Vec<StackFrame>, String> {
        let sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        let _session = sessions.get(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;
        
        // In a real implementation, this would inspect the WASM execution state
        let stack = vec![
            StackFrame {
                function_name: "main".to_string(),
                line: 1,
                column: 1,
                locals: HashMap::new(),
            }
        ];
        
        Ok(stack)
    }
    
    /// Inspect a variable value
    pub fn inspect_variable(&self, session_id: &str, variable_name: &str) -> Result<DebugValue, String> {
        let sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        let _session = sessions.get(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;
        
        // In a real implementation, this would inspect the WASM memory
        // For now, return a placeholder
        match variable_name {
            "test_var" => Ok(DebugValue::Integer(42)),
            _ => Err(format!("Variable '{}' not found", variable_name)),
        }
    }
    
    /// Get all active debug sessions
    pub fn list_sessions(&self) -> Result<Vec<String>, String> {
        let sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(sessions.keys().cloned().collect())
    }
    
    /// Terminate a debug session
    pub fn terminate_session(&mut self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(mut session) = sessions.get_mut(session_id) {
            session.state = DebugState::Terminated;
        }
        Ok(())
    }
}