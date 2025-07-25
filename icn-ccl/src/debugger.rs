// icn-ccl/src/debugger.rs
//! CCL WASM Debugger
//!
//! This module provides debugging capabilities for CCL contracts compiled to WASM:
//! - Breakpoint management
//! - Step-through execution
//! - Variable inspection
//! - Call stack analysis
//! - Memory inspection

use crate::{CclError, ContractMetadata};
use icn_common::{Did, NodeScope};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use wasmtime::{
    AsContext, AsContextMut, Caller, Engine, Extern, Func, Instance, Linker, Memory, Module,
    Store, Trap, Val, ValType, WasmBacktrace,
};

/// Debugger state and control
#[derive(Debug, Clone)]
pub struct DebuggerState {
    pub running: bool,
    pub paused: bool,
    pub step_mode: StepMode,
    pub breakpoints: HashSet<u32>,
    pub current_instruction: Option<u32>,
    pub call_stack: Vec<StackFrame>,
    pub variables: HashMap<String, DebugValue>,
}

/// Step execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    Continue,
    StepInto,
    StepOver,
    StepOut,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub instruction_offset: u32,
    pub local_variables: HashMap<String, DebugValue>,
}

/// Debug value representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Bool(bool),
    Array(Vec<DebugValue>),
    Object(HashMap<String, DebugValue>),
    Null,
}

impl From<Val> for DebugValue {
    fn from(val: Val) -> Self {
        match val {
            Val::I32(v) => DebugValue::I32(v),
            Val::I64(v) => DebugValue::I64(v),
            Val::F32(v) => DebugValue::F32(f32::from_bits(v)),
            Val::F64(v) => DebugValue::F64(f64::from_bits(v)),
            _ => DebugValue::Null,
        }
    }
}

/// Debug event types
#[derive(Debug, Clone)]
pub enum DebugEvent {
    Breakpoint { instruction: u32 },
    Step { instruction: u32 },
    FunctionCall { name: String },
    FunctionReturn { name: String },
    Exception { error: String },
    Finished,
}

/// Callback trait for debug events
pub trait DebugEventHandler: Send + Sync {
    fn on_debug_event(&self, event: DebugEvent, state: &DebuggerState);
}

/// CCL WASM Debugger
pub struct CclDebugger {
    engine: Engine,
    state: Arc<Mutex<DebuggerState>>,
    event_handlers: Vec<Box<dyn DebugEventHandler>>,
    source_map: HashMap<u32, SourceLocation>,
}

/// Source location mapping
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub function: Option<String>,
}

impl CclDebugger {
    /// Create a new CCL debugger
    pub fn new() -> Result<Self, CclError> {
        let engine = Engine::default();

        Ok(Self {
            engine,
            state: Arc::new(Mutex::new(DebuggerState {
                running: false,
                paused: false,
                step_mode: StepMode::Continue,
                breakpoints: HashSet::new(),
                current_instruction: None,
                call_stack: Vec::new(),
                variables: HashMap::new(),
            })),
            event_handlers: Vec::new(),
            source_map: HashMap::new(),
        })
    }

    /// Add a debug event handler
    pub fn add_event_handler(&mut self, handler: Box<dyn DebugEventHandler>) {
        self.event_handlers.push(handler);
    }

    /// Set a breakpoint at the specified instruction offset
    pub fn set_breakpoint(&self, instruction: u32) {
        let mut state = self.state.lock().unwrap();
        state.breakpoints.insert(instruction);
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&self, instruction: u32) {
        let mut state = self.state.lock().unwrap();
        state.breakpoints.remove(&instruction);
    }

    /// Clear all breakpoints
    pub fn clear_breakpoints(&self) {
        let mut state = self.state.lock().unwrap();
        state.breakpoints.clear();
    }

    /// Set step mode
    pub fn set_step_mode(&self, mode: StepMode) {
        let mut state = self.state.lock().unwrap();
        state.step_mode = mode;
    }

    /// Continue execution
    pub fn continue_execution(&self) {
        let mut state = self.state.lock().unwrap();
        state.paused = false;
        state.step_mode = StepMode::Continue;
    }

    /// Pause execution
    pub fn pause_execution(&self) {
        let mut state = self.state.lock().unwrap();
        state.paused = true;
    }

    /// Load and prepare a CCL contract for debugging
    pub async fn load_contract(
        &mut self,
        wasm_bytes: &[u8],
        metadata: &ContractMetadata,
    ) -> Result<DebugSession, CclError> {
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| CclError::CompilationError(format!("Failed to load WASM: {}", e)))?;

        // Build source map from metadata
        self.build_source_map(metadata);

        Ok(DebugSession {
            debugger: self,
            module,
            metadata: metadata.clone(),
        })
    }

    /// Build source mapping from contract metadata
    fn build_source_map(&mut self, metadata: &ContractMetadata) {
        // For now, create basic source mapping
        // In a full implementation, this would parse debug info from metadata
        for (i, function) in metadata.exports.iter().enumerate() {
            self.source_map.insert(
                i as u32 * 100, // Simple offset calculation
                SourceLocation {
                    file: metadata.source_hash.clone(),
                    line: i as u32 + 1,
                    column: 0,
                    function: Some(function.clone()),
                },
            );
        }
    }

    /// Fire debug event to all handlers
    fn fire_event(&self, event: DebugEvent) {
        let state = self.state.lock().unwrap();
        for handler in &self.event_handlers {
            handler.on_debug_event(event.clone(), &state);
        }
    }

    /// Get current debugger state (thread-safe)
    pub fn get_state(&self) -> DebuggerState {
        self.state.lock().unwrap().clone()
    }
}

impl Default for CclDebugger {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Active debug session for a loaded contract
pub struct DebugSession<'a> {
    debugger: &'a CclDebugger,
    module: Module,
    metadata: ContractMetadata,
}

impl<'a> DebugSession<'a> {
    /// Execute the contract with debugging enabled
    pub async fn execute_with_debug(
        &self,
        function_name: &str,
        args: &[Val],
    ) -> Result<Vec<Val>, CclError> {
        let mut store = Store::new(&self.debugger.engine, ());
        let mut linker = Linker::new(&self.debugger.engine);

        // Add debug hooks
        self.add_debug_hooks(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| CclError::CompilationError(format!("Instantiation failed: {}", e)))?;

        let func = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, function_name)
            .map_err(|e| {
                CclError::CompilationError(format!("Function '{}' not found: {}", function_name, e))
            })?;

        // Set initial debug state
        {
            let mut state = self.debugger.state.lock().unwrap();
            state.running = true;
            state.paused = false;
            state.current_instruction = Some(0);
        }

        // Execute with debugging
        let result = match args.len() {
            0 => {
                let typed_func = instance
                    .get_typed_func::<(), i32>(&mut store, function_name)
                    .map_err(|e| {
                        CclError::CompilationError(format!("Function signature mismatch: {}", e))
                    })?;
                vec![Val::I32(typed_func.call(&mut store, ()).map_err(|e| {
                    CclError::CompilationError(format!("Execution failed: {}", e))
                })?)]
            }
            1 => {
                let arg0 = match &args[0] {
                    Val::I32(v) => *v,
                    _ => {
                        return Err(CclError::CompilationError(
                            "Unsupported argument type".to_string(),
                        ))
                    }
                };
                let typed_func = instance
                    .get_typed_func::<i32, i32>(&mut store, function_name)
                    .map_err(|e| {
                        CclError::CompilationError(format!("Function signature mismatch: {}", e))
                    })?;
                vec![Val::I32(
                    typed_func.call(&mut store, arg0).map_err(|e| {
                        CclError::CompilationError(format!("Execution failed: {}", e))
                    })?,
                )]
            }
            2 => {
                let arg0 = match &args[0] {
                    Val::I32(v) => *v,
                    _ => {
                        return Err(CclError::CompilationError(
                            "Unsupported argument type".to_string(),
                        ))
                    }
                };
                let arg1 = match &args[1] {
                    Val::I32(v) => *v,
                    _ => {
                        return Err(CclError::CompilationError(
                            "Unsupported argument type".to_string(),
                        ))
                    }
                };
                vec![Val::I32(func.call(&mut store, (arg0, arg1)).map_err(
                    |e| CclError::CompilationError(format!("Execution failed: {}", e)),
                )?)]
            }
            _ => {
                return Err(CclError::CompilationError(
                    "Too many arguments".to_string(),
                ))
            }
        };

        // Update final state
        {
            let mut state = self.debugger.state.lock().unwrap();
            state.running = false;
            state.paused = false;
        }

        self.debugger.fire_event(DebugEvent::Finished);

        Ok(result)
    }

    /// Add debug hooks to the linker
    fn add_debug_hooks(&self, linker: &mut Linker<()>) -> Result<(), CclError> {
        // Add a debug hook function that WASM can call
        let state = self.debugger.state.clone();
        linker
            .func_wrap("env", "debug_hook", move |_caller: Caller<'_, ()>, offset: i32| {
                let mut debug_state = state.lock().unwrap();
                debug_state.current_instruction = Some(offset as u32);

                // Check for breakpoints
                if debug_state.breakpoints.contains(&(offset as u32)) {
                    debug_state.paused = true;
                    // Note: In a real implementation, we would signal the debugger to pause here
                }

                // Handle step modes
                match debug_state.step_mode {
                    StepMode::StepInto | StepMode::StepOver => {
                        debug_state.paused = true;
                        debug_state.step_mode = StepMode::Continue;
                    }
                    _ => {}
                }
            })
            .map_err(|e| CclError::CompilationError(format!("Failed to add debug hook: {}", e)))?;

        Ok(())
    }

    /// Get memory contents for inspection
    pub fn inspect_memory(&self, _store: &Store<()>, _offset: u32, length: u32) -> Vec<u8> {
        // This is a simplified implementation
        // In a real debugger, we would access the WASM linear memory
        vec![0; length as usize]
    }

    /// Get contract metadata
    pub fn get_metadata(&self) -> &ContractMetadata {
        &self.metadata
    }
}

/// Console debug event handler
pub struct ConsoleDebugHandler;

impl DebugEventHandler for ConsoleDebugHandler {
    fn on_debug_event(&self, event: DebugEvent, state: &DebuggerState) {
        match event {
            DebugEvent::Breakpoint { instruction } => {
                println!("🔴 Breakpoint hit at instruction {}", instruction);
            }
            DebugEvent::Step { instruction } => {
                println!("👣 Step at instruction {}", instruction);
            }
            DebugEvent::FunctionCall { name } => {
                println!("📞 Function call: {}", name);
            }
            DebugEvent::FunctionReturn { name } => {
                println!("📤 Function return: {}", name);
            }
            DebugEvent::Exception { error } => {
                println!("💥 Exception: {}", error);
            }
            DebugEvent::Finished => {
                println!("✅ Execution finished");
            }
        }

        if state.paused {
            println!("⏸️  Execution paused");
        }
    }
}

/// Create a new CCL debugger with console output
pub fn create_console_debugger() -> Result<CclDebugger, CclError> {
    let mut debugger = CclDebugger::new()?;
    debugger.add_event_handler(Box::new(ConsoleDebugHandler));
    Ok(debugger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_creation() {
        let debugger = CclDebugger::new();
        assert!(debugger.is_ok());
    }

    #[test]
    fn test_breakpoint_management() {
        let debugger = CclDebugger::new().unwrap();

        debugger.set_breakpoint(100);
        {
            let state = debugger.state.lock().unwrap();
            assert!(state.breakpoints.contains(&100));
        }

        debugger.remove_breakpoint(100);
        {
            let state = debugger.state.lock().unwrap();
            assert!(!state.breakpoints.contains(&100));
        }
    }

    #[test]
    fn test_step_mode() {
        let debugger = CclDebugger::new().unwrap();

        debugger.set_step_mode(StepMode::StepInto);
        {
            let state = debugger.state.lock().unwrap();
            assert_eq!(state.step_mode, StepMode::StepInto);
        }
    }
}