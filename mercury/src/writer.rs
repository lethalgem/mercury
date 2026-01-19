//! File writer for generated PureScript code

use crate::error::{MercuryError, Result};
use std::fs;
use std::path::Path;

/// Write generated PureScript code to a file
///
/// Creates parent directories if they don't exist.
pub fn write_file(path: &Path, contents: &str) -> Result<()> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| MercuryError::FileWriteError {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    // Write file
    fs::write(path, contents).map_err(|source| MercuryError::FileWriteError {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(())
}
