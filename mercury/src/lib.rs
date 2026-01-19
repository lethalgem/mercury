//! Mercury - Rust to PureScript Type Generator
//!
//! Mercury automatically generates PureScript type definitions and Argonaut JSON
//! codecs from Rust types annotated with `#[mercury]`.
//!
//! # Overview
//!
//! This library provides the core functionality for:
//! - Scanning Rust source files for `#[mercury]` annotations
//! - Parsing Rust type definitions using the `syn` crate
//! - Analyzing and mapping Rust types to PureScript equivalents
//! - Generating PureScript type definitions and JSON codecs
//! - Writing organized multi-module output
//!
//! # Example
//!
//! ```rust,ignore
//! use mercury::generate;
//!
//! // Generate PureScript from annotated Rust types
//! let result = generate(".")?;
//! println!("Generated {} types in {} modules", result.type_count, result.module_count);
//! ```

pub mod analyzer;
pub mod codec_gen;
pub mod codegen;
pub mod error;
pub mod parser;
pub mod scanner;
pub mod serde_attrs;
pub mod types;
pub mod writer;

pub use error::{MercuryError, Result};

use std::path::Path;

/// Result of a successful code generation run
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Number of types generated
    pub type_count: usize,
    /// Number of modules written
    pub module_count: usize,
    /// List of generated file paths
    pub generated_files: Vec<String>,
}

/// Main entry point for code generation
///
/// Scans the workspace for `#[mercury]` annotated types and generates
/// PureScript type definitions and JSON codecs.
///
/// # Arguments
///
/// * `workspace_root` - Path to the Cargo workspace root
///
/// # Returns
///
/// Returns a `GenerationResult` containing statistics about the generation.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace cannot be scanned
/// - Rust source files cannot be parsed
/// - PureScript code generation fails
/// - Output files cannot be written
pub fn generate<P: AsRef<Path>>(workspace_root: P) -> Result<GenerationResult> {
    let workspace_root = workspace_root.as_ref();

    // Step 1: Scan for annotated files
    let annotated_files = scanner::scan_workspace(workspace_root)?;

    if annotated_files.is_empty() {
        return Ok(GenerationResult {
            type_count: 0,
            module_count: 0,
            generated_files: vec![],
        });
    }

    // Step 2: Parse each file to extract type definitions
    let mut all_type_defs = Vec::new();
    for annotated_file in &annotated_files {
        let contents = std::fs::read_to_string(&annotated_file.path)?;

        // Make path relative to workspace root to avoid exposing absolute paths
        let relative_path = annotated_file
            .path
            .strip_prefix(workspace_root)
            .unwrap_or(&annotated_file.path);

        let type_defs = parser::parse_file(relative_path, &contents)?;
        all_type_defs.extend(type_defs);
    }

    // Step 3: Group types by source file
    let modules = group_types_by_module(&all_type_defs, workspace_root);

    // Step 3.5: Build a map of type name -> module name for cross-module imports
    let type_to_module = build_type_to_module_map(&modules);

    // Step 4: Generate and write each module
    let output_dir = workspace_root.join("frontend/src/Generated");
    std::fs::create_dir_all(&output_dir)?;

    let mut generated_files = Vec::new();
    for (module_name, type_defs) in &modules {
        let module_code = codegen::generate_module(module_name, type_defs, &type_to_module);
        let file_name = format!("{}.purs", module_name.replace('.', "/"));
        let output_path = output_dir.join(&file_name);

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        writer::write_file(&output_path, &module_code)?;
        generated_files.push(output_path.display().to_string());
    }

    Ok(GenerationResult {
        type_count: all_type_defs.len(),
        module_count: modules.len(),
        generated_files,
    })
}

/// Group type definitions by their source module
fn group_types_by_module(
    type_defs: &[types::TypeDefinition],
    workspace_root: &Path,
) -> std::collections::BTreeMap<String, Vec<types::TypeDefinition>> {
    use std::collections::BTreeMap;

    let mut modules: BTreeMap<String, Vec<types::TypeDefinition>> = BTreeMap::new();

    for type_def in type_defs {
        let module_name = source_path_to_module_name(&type_def.source_file, workspace_root);
        modules
            .entry(module_name)
            .or_default()
            .push(type_def.clone());
    }

    modules
}

/// Build a map from type name to module name for resolving cross-module imports
fn build_type_to_module_map(
    modules: &std::collections::BTreeMap<String, Vec<types::TypeDefinition>>,
) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;

    let mut map = HashMap::new();

    for (module_name, type_defs) in modules {
        for type_def in type_defs {
            map.insert(type_def.name.clone(), module_name.clone());
        }
    }

    map
}

/// Convert a Rust source file path to a PureScript module name
///
/// Examples:
/// - `app/backend/src/models.rs` -> `Generated.Models`
/// - `lib/constitution/src/models/merchant.rs` -> `Generated.Merchant`
/// - `test-mercury/test.rs` -> `Generated.Test`
fn source_path_to_module_name(source_path: &Path, workspace_root: &Path) -> String {
    // Get relative path from workspace root
    let relative = source_path
        .strip_prefix(workspace_root)
        .unwrap_or(source_path);

    // Extract the meaningful part of the path
    let _path_str = relative.to_string_lossy();

    // Simple heuristic: use the file name without extension as the module name
    // This can be enhanced later with more sophisticated mapping
    let file_name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    // Convert to PascalCase
    let module_suffix = to_pascal_case(file_name);

    format!("Generated.{}", module_suffix)
}

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
