//! CCL Error types and handling
//!
//! Comprehensive error reporting for all stages of CCL compilation and execution.

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CclError {
    #[error("Parsing error: {0}")]
    ParsingError(String), // Typically from Pest

    #[error("Semantic error: {0}")]
    SemanticError(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Optimization error: {0}")]
    OptimizationError(String),

    #[error("WASM generation error: {0}")]
    WasmGenerationError(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("CLI argument error: {0}")]
    CliArgumentError(String),

    #[error("Internal compiler error: {0}")]
    InternalCompilerError(String),

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    // Advanced error types for better debugging
    #[error("Undefined symbol: {symbol} at line {line}, column {column}")]
    UndefinedSymbol {
        symbol: String,
        line: usize,
        column: usize,
    },

    #[error("Type mismatch: expected {expected}, found {found} at line {line}")]
    TypeMismatch {
        expected: String,
        found: String,
        line: usize,
    },

    #[error("Function signature mismatch: {function} expects {expected_params} parameters, got {actual_params}")]
    FunctionSignatureMismatch {
        function: String,
        expected_params: usize,
        actual_params: usize,
    },

    #[error("Resource limit exceeded: {resource} limit of {limit} exceeded with value {actual}")]
    ResourceLimitExceeded {
        resource: String,
        limit: u64,
        actual: u64,
    },

    #[error("Mana limit exceeded: used {used} mana, limit is {limit}")]
    ManaLimitExceeded { used: u64, limit: u64 },

    #[error("Memory allocation failed: requested {requested} bytes, available {available}")]
    MemoryAllocationFailed { requested: usize, available: usize },

    #[error("Stack overflow: maximum call depth {max_depth} exceeded")]
    StackOverflow { max_depth: usize },

    #[error("Division by zero in expression at line {line}")]
    DivisionByZero { line: usize },

    #[error("Array index out of bounds: index {index}, array length {length} at line {line}")]
    ArrayIndexOutOfBounds {
        index: i64,
        length: usize,
        line: usize,
    },

    #[error("Integer overflow in operation at line {line}")]
    IntegerOverflow { line: usize },

    #[error("Governance constraint violation: {constraint} at line {line}")]
    GovernanceConstraintViolation { constraint: String, line: usize },

    #[error("Macro expansion error: {macro_name} failed to expand: {reason}")]
    MacroExpansionError { macro_name: String, reason: String },

    #[error("Module import error: {module} could not be imported: {reason}")]
    ModuleImportError { module: String, reason: String },

    #[error("Pattern matching error: non-exhaustive patterns for {type_name} at line {line}")]
    NonExhaustivePatterns { type_name: String, line: usize },

    #[error("Unreachable code detected at line {line}")]
    UnreachableCode { line: usize },

    #[error("Circular dependency detected: {cycle}")]
    CircularDependency { cycle: String },

    #[error("Contract validation failed: {validator} failed with message: {message}")]
    ContractValidationFailed { validator: String, message: String },

    // Additional semantic analyzer error types
    #[error("Type mismatch: expected {expected:?}, found {found:?}")]
    TypeMismatchError {
        expected: crate::ast::TypeAnnotationNode,
        found: crate::ast::TypeAnnotationNode,
    },

    #[error("Undefined variable: {variable}")]
    UndefinedVariableError { variable: String },

    #[error("Undefined function: {function}")]
    UndefinedFunctionError { function: String },

    #[error("Argument count mismatch: {function} expects {expected} arguments, got {found}")]
    ArgumentCountMismatchError {
        function: String,
        expected: usize,
        found: usize,
    },

    #[error("Duplicate field: {field_name} in struct {struct_name}")]
    DuplicateFieldError {
        struct_name: String,
        field_name: String,
    },

    #[error("Cannot assign to immutable variable: {variable}")]
    ImmutableAssignmentError { variable: String },

    #[error("Invalid binary operation: {left_type:?} {operator:?} {right_type:?}")]
    InvalidBinaryOperationError {
        left_type: crate::ast::TypeAnnotationNode,
        operator: crate::ast::BinaryOperator,
        right_type: crate::ast::TypeAnnotationNode,
    },

    #[error("Invalid unary operation: {operator:?} {operand_type:?}")]
    InvalidUnaryOperationError {
        operator: crate::ast::UnaryOperator,
        operand_type: crate::ast::TypeAnnotationNode,
    },

    #[error("Code generation error: {0}")]
    CodeGenError(String),
}

impl From<std::io::Error> for CclError {
    fn from(err: std::io::Error) -> Self {
        CclError::IoError(err.to_string())
    }
}

impl CclError {
    /// Create a parsing error with position information
    pub fn parsing_error_at(message: &str, line: usize, column: usize) -> Self {
        CclError::ParsingError(format!("{} at line {}, column {}", message, line, column))
    }

    /// Create a type error with detailed context
    pub fn type_error_with_context(
        expected: &str,
        found: &str,
        line: usize,
        context: &str,
    ) -> Self {
        CclError::TypeMismatch {
            expected: format!("{} (in {})", expected, context),
            found: found.to_string(),
            line,
        }
    }

    /// Create a resource limit error
    pub fn resource_limit(resource: &str, limit: u64, actual: u64) -> Self {
        CclError::ResourceLimitExceeded {
            resource: resource.to_string(),
            limit,
            actual,
        }
    }

    /// Create a mana limit error
    pub fn mana_limit_exceeded(used: u64, limit: u64) -> Self {
        CclError::ManaLimitExceeded { used, limit }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            CclError::ParsingError(_)
            | CclError::SemanticError(_)
            | CclError::TypeError(_)
            | CclError::UndefinedSymbol { .. }
            | CclError::TypeMismatch { .. }
            | CclError::FunctionSignatureMismatch { .. } => true,

            CclError::StackOverflow { .. }
            | CclError::ManaLimitExceeded { .. }
            | CclError::MemoryAllocationFailed { .. } => false,

            _ => true, // Default to recoverable for new error types
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            CclError::ParsingError(_)
            | CclError::SemanticError(_)
            | CclError::TypeError(_)
            | CclError::UndefinedSymbol { .. }
            | CclError::TypeMismatch { .. } => ErrorSeverity::Error,

            CclError::UnreachableCode { .. } | CclError::NonExhaustivePatterns { .. } => {
                ErrorSeverity::Warning
            }

            CclError::StackOverflow { .. }
            | CclError::ManaLimitExceeded { .. }
            | CclError::DivisionByZero { .. } => ErrorSeverity::Fatal,

            _ => ErrorSeverity::Error,
        }
    }

    /// Get suggestions for fixing this error
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            CclError::UndefinedSymbol { symbol, .. } => {
                vec![
                    format!("Check if '{}' is spelled correctly", symbol),
                    "Make sure the symbol is defined before use".to_string(),
                    "Check if you need to import a module".to_string(),
                ]
            }
            CclError::TypeMismatch {
                expected, found, ..
            } => {
                vec![
                    format!("Convert {} to {} using explicit casting", found, expected),
                    "Check the function signature".to_string(),
                    "Verify variable assignments".to_string(),
                ]
            }
            CclError::ManaLimitExceeded { .. } => {
                vec![
                    "Optimize your contract to use fewer operations".to_string(),
                    "Break large operations into smaller chunks".to_string(),
                    "Consider using more efficient algorithms".to_string(),
                ]
            }
            _ => vec!["Check the CCL documentation for guidance".to_string()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Fatal,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Warning => "warning",
            ErrorSeverity::Error => "error",
            ErrorSeverity::Fatal => "fatal",
        }
    }
}

// If using pest::error::Error directly:
// impl From<pest::error::Error<crate::parser::Rule>> for CclError {
//     fn from(err: pest::error::Error<crate::parser::Rule>) -> Self {
//         CclError::ParsingError(err.to_string())
//     }
// }
