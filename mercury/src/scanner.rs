//! File scanner for finding #[mercury] annotations

use crate::error::{MercuryError, Result};
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A file containing #[mercury] annotated types
#[derive(Debug, Clone)]
pub struct AnnotatedFile {
    /// Path to the source file
    pub path: PathBuf,
    /// Number of #[mercury] annotations found
    pub annotation_count: usize,
}

/// Scan a Cargo workspace for files containing #[mercury] annotations
///
/// This function walks the workspace directory looking for `.rs` files,
/// then scans each file for the `#[mercury]` attribute marker.
///
/// # Arguments
///
/// * `workspace_root` - Path to the Cargo workspace root directory
///
/// # Returns
///
/// Returns a vector of `AnnotatedFile` structs, one for each file containing
/// at least one `#[mercury]` annotation.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace directory cannot be read
/// - File contents cannot be read
pub fn scan_workspace<P: AsRef<Path>>(workspace_root: P) -> Result<Vec<AnnotatedFile>> {
    let workspace_root = workspace_root.as_ref();

    // Regex to match #[mercury] attribute (handles whitespace variations)
    let mercury_pattern = Regex::new(r"#\s*\[\s*mercury\s*\]")
        .map_err(|e| MercuryError::ScanError(format!("Invalid regex: {}", e)))?;

    let mut annotated_files = Vec::new();

    // Walk the workspace directory
    for entry in WalkDir::new(workspace_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_excluded(e.path()))
    {
        let entry = entry.map_err(|e| {
            MercuryError::ScanError(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Only process .rs files
        if !path.is_file() || path.extension().map(|e| e != "rs").unwrap_or(true) {
            continue;
        }

        // Read file contents
        let contents =
            std::fs::read_to_string(path).map_err(|source| MercuryError::FileReadError {
                path: path.to_path_buf(),
                source,
            })?;

        // Count #[mercury] annotations
        let annotation_count = mercury_pattern.find_iter(&contents).count();

        if annotation_count > 0 {
            annotated_files.push(AnnotatedFile {
                path: path.to_path_buf(),
                annotation_count,
            });
        }
    }

    Ok(annotated_files)
}

/// Check if a path should be excluded from scanning
///
/// Excludes common directories that shouldn't contain source code:
/// - target/
/// - .git/
/// - node_modules/
/// - frontend/ (we're generating for this, not scanning it)
fn is_excluded(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        matches!(
            name.as_ref(),
            "target" | ".git" | "node_modules" | "frontend" | "dist" | "build"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_scan_finds_mercury_annotation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(
            file,
            r#"
            #[mercury]
            pub struct Foo {{
                pub id: i32,
            }}
        "#
        )
        .unwrap();

        let result = scan_workspace(temp_dir.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, file_path);
        assert_eq!(result[0].annotation_count, 1);
    }

    #[test]
    fn test_scan_counts_multiple_annotations() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(
            file,
            r#"
            #[mercury]
            pub struct Foo {{ pub id: i32 }}

            #[mercury]
            pub enum Bar {{ A, B }}
        "#
        )
        .unwrap();

        let result = scan_workspace(temp_dir.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].annotation_count, 2);
    }

    #[test]
    fn test_scan_ignores_files_without_annotation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(
            file,
            r#"
            pub struct Foo {{ pub id: i32 }}
        "#
        )
        .unwrap();

        let result = scan_workspace(temp_dir.path()).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_is_excluded() {
        assert!(is_excluded(Path::new("target/debug/foo.rs")));
        assert!(is_excluded(Path::new(".git/config")));
        assert!(is_excluded(Path::new("node_modules/pkg/index.js")));
        assert!(is_excluded(Path::new("frontend/src/Main.purs")));

        assert!(!is_excluded(Path::new("src/main.rs")));
        assert!(!is_excluded(Path::new("lib/mercury/src/lib.rs")));
    }
}
