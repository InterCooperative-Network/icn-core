// icn-ccl/src/debugger/wasm_debugger.rs
//! WASM debugger integration for CCL contracts

use super::source_map::{SourceLocation, SourceMap, WasmLocation};
use std::collections::HashMap;

/// Breakpoint in a CCL contract
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: u32,
    pub ccl_location: SourceLocation,
    pub wasm_location: Option<WasmLocation>,
    pub condition: Option<String>, // Optional condition for conditional breakpoints
    pub enabled: bool,
}

/// Debugger state
#[derive(Debug, Clone)]
pub enum DebuggerState {
    Stopped,
    Running,
    Paused { location: WasmLocation },
    Stepped { location: WasmLocation },
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_index: u32,
    pub function_name: Option<String>,
    pub ccl_location: Option<SourceLocation>,
    pub locals: HashMap<String, String>, // Variable name -> value
}

/// WASM debugger for CCL contracts
pub struct WasmDebugger {
    source_map: SourceMap,
    breakpoints: HashMap<u32, Breakpoint>,
    next_breakpoint_id: u32,
    state: DebuggerState,
    call_stack: Vec<StackFrame>,
}

impl WasmDebugger {
    /// Create a new WASM debugger with source map
    pub fn new(source_map: SourceMap) -> Self {
        Self {
            source_map,
            breakpoints: HashMap::new(),
            next_breakpoint_id: 1,
            state: DebuggerState::Stopped,
            call_stack: Vec::new(),
        }
    }

    /// Add a breakpoint at a CCL source location
    pub fn add_breakpoint(
        &mut self,
        ccl_location: SourceLocation,
        condition: Option<String>,
    ) -> Result<u32, String> {
        // Find corresponding WASM location
        let wasm_location = self.source_map.find_wasm_location(&ccl_location).cloned();

        if wasm_location.is_none() {
            return Err("No corresponding WASM location found for CCL location".to_string());
        }

        let breakpoint_id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;

        let breakpoint = Breakpoint {
            id: breakpoint_id,
            ccl_location,
            wasm_location,
            condition,
            enabled: true,
        };

        self.breakpoints.insert(breakpoint_id, breakpoint);
        Ok(breakpoint_id)
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, breakpoint_id: u32) -> bool {
        self.breakpoints.remove(&breakpoint_id).is_some()
    }

    /// Enable or disable a breakpoint
    pub fn set_breakpoint_enabled(&mut self, breakpoint_id: u32, enabled: bool) -> bool {
        if let Some(breakpoint) = self.breakpoints.get_mut(&breakpoint_id) {
            breakpoint.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Get all breakpoints
    pub fn get_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Start debugging (this would interface with actual WASM runtime)
    pub fn start_debugging(&mut self) -> Result<(), String> {
        // TODO: Interface with actual WASM runtime for debugging
        // This would typically involve:
        // 1. Loading the WASM module with debug info
        // 2. Setting up breakpoint handlers
        // 3. Starting execution in debug mode

        self.state = DebuggerState::Running;
        Ok(())
    }

    /// Step to next instruction
    pub fn step_next(&mut self) -> Result<DebuggerState, String> {
        match &self.state {
            DebuggerState::Paused { location } | DebuggerState::Stepped { location } => {
                // Simulate stepping to the next instruction
                // In a real implementation, this would interact with the WASM runtime
                let next_location = WasmLocation {
                    function_index: location.function_index,
                    instruction_offset: location.instruction_offset + 1,
                };

                self.state = DebuggerState::Stepped {
                    location: next_location.clone(),
                };

                // Update call stack if needed
                self.update_call_stack_for_step(&next_location)?;

                Ok(self.state.clone())
            }
            DebuggerState::Stopped => Err("Cannot step when debugger is stopped".to_string()),
            DebuggerState::Running => Err("Cannot step when debugger is running".to_string()),
        }
    }

    /// Step into function calls
    pub fn step_into(&mut self) -> Result<DebuggerState, String> {
        match &self.state {
            DebuggerState::Paused { location } | DebuggerState::Stepped { location } => {
                // Check if current instruction is a function call
                if self.is_function_call_at_location(location) {
                    // Step into the function
                    let called_function_index = self.get_called_function_index(location)?;
                    let entry_location = WasmLocation {
                        function_index: called_function_index,
                        instruction_offset: 0,
                    };

                    self.state = DebuggerState::Stepped {
                        location: entry_location,
                    };

                    // Push new stack frame
                    self.push_stack_frame(called_function_index);

                    Ok(self.state.clone())
                } else {
                    // No function call, just step next
                    self.step_next()
                }
            }
            _ => Err("Cannot step into when not paused".to_string()),
        }
    }

    /// Step out of current function
    pub fn step_out(&mut self) -> Result<DebuggerState, String> {
        match &self.state {
            DebuggerState::Paused { .. } | DebuggerState::Stepped { .. } => {
                if self.call_stack.len() > 1 {
                    // Pop current stack frame
                    self.call_stack.pop();

                    // Get the return location in the calling function
                    if let Some(caller_frame) = self.call_stack.last() {
                        let return_location = WasmLocation {
                            function_index: caller_frame.function_index,
                            instruction_offset: 0, // Would be the actual return address
                        };

                        self.state = DebuggerState::Stepped {
                            location: return_location,
                        };
                        Ok(self.state.clone())
                    } else {
                        Err("No caller to step out to".to_string())
                    }
                } else {
                    Err("Already at top-level function".to_string())
                }
            }
            _ => Err("Cannot step out when not paused".to_string()),
        }
    }

    /// Continue execution until next breakpoint
    pub fn continue_execution(&mut self) -> Result<DebuggerState, String> {
        // TODO: Interface with WASM runtime to continue execution

        self.state = DebuggerState::Running;
        Ok(self.state.clone())
    }

    /// Get current debugger state
    pub fn get_state(&self) -> &DebuggerState {
        &self.state
    }

    /// Get current call stack
    pub fn get_call_stack(&self) -> &[StackFrame] {
        &self.call_stack
    }

    /// Get local variables at current location
    pub fn get_locals(&self) -> HashMap<String, String> {
        if let Some(frame) = self.call_stack.last() {
            frame.locals.clone()
        } else {
            HashMap::new()
        }
    }

    /// Evaluate an expression in the current context
    pub fn evaluate_expression(&self, expression: &str) -> Result<String, String> {
        match &self.state {
            DebuggerState::Paused { .. } | DebuggerState::Stepped { .. } => {
                // Parse and evaluate the expression
                if let Some(current_frame) = self.call_stack.last() {
                    self.evaluate_in_context(expression, current_frame)
                } else {
                    Err("No execution context available".to_string())
                }
            }
            DebuggerState::Stopped => Err("Cannot evaluate expression when stopped".to_string()),
            DebuggerState::Running => Err("Cannot evaluate expression while running".to_string()),
        }
    }

    /// Set a variable value (for debugging)
    pub fn set_variable(&mut self, variable_name: &str, value: &str) -> Result<(), String> {
        match &self.state {
            DebuggerState::Paused { .. } | DebuggerState::Stepped { .. } => {
                // Validate the value format first
                let validated_value = self.validate_and_convert_value(value)?;

                if let Some(current_frame) = self.call_stack.last_mut() {
                    // Set the variable in the current frame
                    current_frame
                        .locals
                        .insert(variable_name.to_string(), validated_value);
                    Ok(())
                } else {
                    Err("No execution context available".to_string())
                }
            }
            _ => Err("Cannot set variable when not paused".to_string()),
        }
    }

    /// Get the source map
    pub fn get_source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Update call stack when stepping
    fn update_call_stack_for_step(&mut self, location: &WasmLocation) -> Result<(), String> {
        // In a real implementation, this would check if we've entered or exited functions
        // and update the call stack accordingly

        // For now, just update the current frame's location info if available
        let function_index = location.function_index;

        if let Some(current_frame) = self.call_stack.last_mut() {
            current_frame.function_index = function_index;
        }

        // Update locals separately to avoid borrow checker issues
        self.refresh_locals_at_location(location)?;

        Ok(())
    }

    /// Refresh locals at a specific location
    fn refresh_locals_at_location(&mut self, _location: &WasmLocation) -> Result<(), String> {
        if let Some(current_frame) = self.call_stack.last_mut() {
            // In a real implementation, this would inspect the WASM runtime state
            // to get current local variable values

            // For demonstration, add some mock locals
            current_frame
                .locals
                .insert("local_var".to_string(), "42".to_string());
            current_frame
                .locals
                .insert("param1".to_string(), "hello".to_string());
        }

        Ok(())
    }

    /// Check if the instruction at a location is a function call
    fn is_function_call_at_location(&self, _location: &WasmLocation) -> bool {
        // In a real implementation, this would examine the WASM bytecode
        // to determine if the current instruction is a function call

        // For demonstration, assume 20% of instructions are function calls
        #[allow(clippy::manual_is_multiple_of)]
        {
            _location.instruction_offset % 5 == 0
        }
    }

    /// Get the index of the function being called at a location
    fn get_called_function_index(&self, location: &WasmLocation) -> Result<u32, String> {
        // In a real implementation, this would decode the function call instruction
        // to extract the target function index

        // For demonstration, return a mock function index
        Ok((location.function_index + 1) % 10)
    }

    /// Push a new stack frame for entering a function
    fn push_stack_frame(&mut self, function_index: u32) {
        let function_name = self.get_function_name(function_index);
        let ccl_location = self
            .source_map
            .find_ccl_location(&WasmLocation {
                function_index,
                instruction_offset: 0,
            })
            .cloned();

        let frame = StackFrame {
            function_index,
            function_name,
            ccl_location,
            locals: HashMap::new(),
        };

        self.call_stack.push(frame);
    }

    /// Get function name by index
    fn get_function_name(&self, function_index: u32) -> Option<String> {
        // In a real implementation, this would look up function names
        // from debug information or a symbol table

        Some(format!("function_{}", function_index))
    }

    /// Evaluate an expression in a given stack frame context
    fn evaluate_in_context(&self, expression: &str, frame: &StackFrame) -> Result<String, String> {
        let trimmed_expr = expression.trim();

        // Handle simple variable lookups
        if let Some(value) = frame.locals.get(trimmed_expr) {
            return Ok(value.clone());
        }

        // Handle simple arithmetic expressions
        if let Ok(result) = self.evaluate_arithmetic_expression(trimmed_expr, frame) {
            return Ok(result);
        }

        // Handle function calls (simplified)
        if trimmed_expr.contains('(') && trimmed_expr.contains(')') {
            return self.evaluate_function_call(trimmed_expr, frame);
        }

        // Handle literals
        if trimmed_expr.parse::<i64>().is_ok() {
            return Ok(trimmed_expr.to_string());
        }

        if trimmed_expr.starts_with('"') && trimmed_expr.ends_with('"') {
            return Ok(trimmed_expr.to_string());
        }

        Err(format!("Cannot evaluate expression: {}", expression))
    }

    /// Evaluate simple arithmetic expressions
    fn evaluate_arithmetic_expression(
        &self,
        expression: &str,
        frame: &StackFrame,
    ) -> Result<String, String> {
        // Handle simple binary operations
        for op in &["+", "-", "*", "/", "==", "!=", "<", ">"] {
            if let Some(pos) = expression.find(op) {
                let left = expression[..pos].trim();
                let right = expression[pos + op.len()..].trim();

                let left_val = self.evaluate_operand(left, frame)?;
                let right_val = self.evaluate_operand(right, frame)?;

                return self.apply_operator(&left_val, op, &right_val);
            }
        }

        Err("Not an arithmetic expression".to_string())
    }

    /// Evaluate an operand (variable or literal)
    fn evaluate_operand(&self, operand: &str, frame: &StackFrame) -> Result<String, String> {
        // Check if it's a variable
        if let Some(value) = frame.locals.get(operand) {
            return Ok(value.clone());
        }

        // Check if it's a number
        if operand.parse::<i64>().is_ok() {
            return Ok(operand.to_string());
        }

        Err(format!("Unknown operand: {}", operand))
    }

    /// Apply an operator to two operands
    fn apply_operator(&self, left: &str, op: &str, right: &str) -> Result<String, String> {
        let left_num = left
            .parse::<i64>()
            .map_err(|_| "Left operand is not a number")?;
        let right_num = right
            .parse::<i64>()
            .map_err(|_| "Right operand is not a number")?;

        let result = match op {
            "+" => left_num + right_num,
            "-" => left_num - right_num,
            "*" => left_num * right_num,
            "/" => {
                if right_num == 0 {
                    return Err("Division by zero".to_string());
                }
                left_num / right_num
            }
            _ => return Err(format!("Unsupported operator: {}", op)),
        };

        Ok(result.to_string())
    }

    /// Evaluate function calls
    fn evaluate_function_call(
        &self,
        expression: &str,
        _frame: &StackFrame,
    ) -> Result<String, String> {
        // Parse function name and arguments
        let paren_pos = expression.find('(').unwrap();
        let func_name = &expression[..paren_pos].trim();

        // Handle built-in functions
        match *func_name {
            "len" => Ok("5".to_string()),   // Mock implementation
            "max" => Ok("100".to_string()), // Mock implementation
            "min" => Ok("0".to_string()),   // Mock implementation
            _ => Err(format!("Unknown function: {}", func_name)),
        }
    }

    /// Validate and convert a value string for variable setting
    fn validate_and_convert_value(&self, value: &str) -> Result<String, String> {
        let trimmed = value.trim();

        // Check if it's a number
        if trimmed.parse::<i64>().is_ok() {
            return Ok(trimmed.to_string());
        }

        // Check if it's a float
        if trimmed.parse::<f64>().is_ok() {
            return Ok(trimmed.to_string());
        }

        // Check if it's a string literal
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return Ok(trimmed.to_string());
        }

        // Check if it's a boolean
        if trimmed == "true" || trimmed == "false" {
            return Ok(trimmed.to_string());
        }

        Err(format!("Invalid value format: {}", value))
    }
}
