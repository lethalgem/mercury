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

/// Two annotated files share a basename (`models.rs`), so they collapse into the
/// same generated module (`Generated.Models`) and their types are concatenated.
/// `generate` sorts scanned files by path before parsing, so the emitted order
/// must follow the source path, not the filesystem scan order or the type name.
///
/// We place `Zebra` under `aaa_crate/` and `Apple` under `zzz_crate/`: path order
/// (aaa < zzz) yields Zebra-before-Apple, while type-name order would be the
/// reverse. Asserting Zebra precedes Apple proves ordering is path-driven.
#[test]
fn test_output_ordered_by_source_path() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    let write_struct = |dir: &str, name: &str| {
        let crate_src = root.join(dir).join("src");
        fs::create_dir_all(&crate_src).unwrap();
        fs::write(
            crate_src.join("models.rs"),
            format!("#[mercury]\npub struct {name} {{\n    pub id: i32,\n}}\n"),
        )
        .unwrap();
    };

    write_struct("aaa_crate", "Zebra");
    write_struct("zzz_crate", "Apple");

    let result = cargo_mercury::generate(root).unwrap();
    assert_eq!(result.type_count, 2);
    assert_eq!(result.module_count, 1, "same basename collapses to one module");

    let module = &result.generated_files[0];
    let output = fs::read_to_string(module).unwrap();

    let zebra = output.find("Zebra").expect("Zebra type missing from output");
    let apple = output.find("Apple").expect("Apple type missing from output");
    assert!(
        zebra < apple,
        "types must be ordered by source path (aaa_crate before zzz_crate), \
         not by type name; got:\n{output}"
    );
}

// Note: Cross-module import tests are application-specific
// and tested in the consuming project
