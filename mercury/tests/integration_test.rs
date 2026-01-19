//! Integration tests for Mercury code generation
use std::fs;
use std::path::PathBuf;

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn test_empty_workspace() {
    let temp_dir = std::env::temp_dir().join("mercury_test_empty");
    fs::create_dir_all(&temp_dir).unwrap();
    let result = cargo_mercury::generate(&temp_dir).unwrap();
    assert_eq!(result.type_count, 0);
    assert_eq!(result.module_count, 0);
    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_deterministic_output() {
    let workspace_root = get_workspace_root();
    let result1 = cargo_mercury::generate(&workspace_root).unwrap();
    let mut first_gen = std::collections::HashMap::new();
    for file_path in &result1.generated_files {
        first_gen.insert(file_path.clone(), fs::read_to_string(file_path).unwrap());
    }
    let result2 = cargo_mercury::generate(&workspace_root).unwrap();
    assert_eq!(result1.type_count, result2.type_count);
    for file_path in &result2.generated_files {
        assert_eq!(
            first_gen.get(file_path).unwrap(),
            &fs::read_to_string(file_path).unwrap()
        );
    }
}

// Note: Cross-module import tests are application-specific
// and tested in the consuming project
