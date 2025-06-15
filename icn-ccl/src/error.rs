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
}

impl From<std::io::Error> for CclError {
    fn from(err: std::io::Error) -> Self {
        CclError::IoError(err.to_string())
    }
}

// If using pest::error::Error directly:
// impl From<pest::error::Error<crate::parser::Rule>> for CclError {
//     fn from(err: pest::error::Error<crate::parser::Rule>) -> Self {
//         CclError::ParsingError(err.to_string())
//     }
// }
