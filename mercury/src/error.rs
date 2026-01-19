//! Error types for Mercury code generation

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using MercuryError
pub type Result<T> = std::result::Result<T, MercuryError>;

/// Errors that can occur during Mercury code generation
#[derive(Debug, Error)]
pub enum MercuryError {
    /// Error scanning workspace for annotated files
    #[error("Failed to scan workspace: {0}")]
    ScanError(String),

    /// Error reading a file
    #[error("Failed to read file {path}: {source}")]
    FileReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Error parsing Rust source code
    #[error("Failed to parse Rust source in {file}: {message}")]
    ParseError { file: PathBuf, message: String },

    /// Unsupported type encountered
    #[error("Unsupported type '{rust_type}' in {file}:{line}\n  → Suggestion: {suggestion}")]
    UnsupportedType {
        rust_type: String,
        file: PathBuf,
        line: usize,
        suggestion: String,
    },

    /// Conflicting serde attributes
    #[error(
        "Conflicting serde attributes on {type_name}::{field_name} in {file}:{line}\n  → {details}"
    )]
    SerdeAttributeConflict {
        type_name: String,
        field_name: String,
        file: PathBuf,
        line: usize,
        details: String,
    },

    /// Error generating PureScript code
    #[error("Failed to generate PureScript code for {type_name}: {reason}")]
    CodegenError { type_name: String, reason: String },

    /// Error writing output file
    #[error("Failed to write output file {path}: {source}")]
    FileWriteError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Generic IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}
