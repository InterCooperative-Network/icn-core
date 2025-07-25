// icn-ccl/src/debugger/source_map.rs
//! Source mapping between CCL and WASM for debugging

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Represents a location in CCL source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Represents a location in WASM bytecode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLocation {
    pub function_index: u32,
    pub instruction_offset: u32,
}

/// Mapping between CCL source and WASM bytecode locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapEntry {
    pub ccl_location: SourceLocation,
    pub wasm_location: WasmLocation,
    pub ccl_symbol: Option<String>,  // Variable or function name
}

/// Complete source map for a CCL contract
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceMap {
    pub contract_name: String,
    pub ccl_source_file: String,
    pub wasm_module: String,
    pub mappings: Vec<SourceMapEntry>,
    pub function_map: HashMap<String, u32>,  // CCL function name -> WASM function index
    pub variable_map: HashMap<String, u32>,  // CCL variable name -> WASM local index
}

impl SourceMap {
    /// Create a new source map
    pub fn new(contract_name: String, ccl_source_file: String, wasm_module: String) -> Self {
        Self {
            contract_name,
            ccl_source_file,
            wasm_module,
            mappings: Vec::new(),
            function_map: HashMap::new(),
            variable_map: HashMap::new(),
        }
    }

    /// Add a mapping between CCL and WASM locations
    pub fn add_mapping(&mut self, ccl_location: SourceLocation, wasm_location: WasmLocation, symbol: Option<String>) {
        self.mappings.push(SourceMapEntry {
            ccl_location,
            wasm_location,
            ccl_symbol: symbol,
        });
    }

    /// Find CCL location for a WASM instruction
    pub fn find_ccl_location(&self, wasm_location: &WasmLocation) -> Option<&SourceLocation> {
        self.mappings
            .iter()
            .find(|entry| {
                entry.wasm_location.function_index == wasm_location.function_index
                    && entry.wasm_location.instruction_offset == wasm_location.instruction_offset
            })
            .map(|entry| &entry.ccl_location)
    }

    /// Find WASM location for a CCL source position
    pub fn find_wasm_location(&self, ccl_location: &SourceLocation) -> Option<&WasmLocation> {
        self.mappings
            .iter()
            .find(|entry| {
                entry.ccl_location.file == ccl_location.file
                    && entry.ccl_location.line == ccl_location.line
                    && entry.ccl_location.column == ccl_location.column
            })
            .map(|entry| &entry.wasm_location)
    }

    /// Get all breakpoint locations for a CCL file
    pub fn get_breakpoint_locations(&self, ccl_file: &str) -> Vec<SourceLocation> {
        self.mappings
            .iter()
            .filter(|entry| entry.ccl_location.file == ccl_file)
            .map(|entry| entry.ccl_location.clone())
            .collect()
    }

    /// Add a function mapping
    pub fn add_function_mapping(&mut self, ccl_name: String, wasm_index: u32) {
        self.function_map.insert(ccl_name, wasm_index);
    }

    /// Add a variable mapping  
    pub fn add_variable_mapping(&mut self, ccl_name: String, wasm_local: u32) {
        self.variable_map.insert(ccl_name, wasm_local);
    }

    /// Serialize source map to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize source map from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}