//! Internal representation types for Mercury code generation

use crate::serde_attrs::{RenameRule, SerdeAttrs};
use std::path::PathBuf;

/// Represents a Rust type to be generated as PureScript
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Name of the type
    pub name: String,
    /// Source file path
    pub source_file: PathBuf,
    /// Line number in source file
    pub line: usize,
    /// The kind of type (struct or enum)
    pub kind: TypeKind,
    /// Serde attributes on the type
    pub serde_attrs: SerdeAttrs,
}

/// Kind of Rust type
#[derive(Debug, Clone)]
pub enum TypeKind {
    /// A struct with named fields
    Struct(StructType),
    /// An enum with variants
    Enum(EnumType),
}

/// A struct type definition
#[derive(Debug, Clone)]
pub struct StructType {
    /// Fields in the struct
    pub fields: Vec<Field>,
    /// Serde rename_all rule for this struct
    pub rename_all: Option<RenameRule>,
}

/// An enum type definition
#[derive(Debug, Clone)]
pub struct EnumType {
    /// Variants in the enum
    pub variants: Vec<EnumVariant>,
    /// Serde rename_all rule for this enum
    pub rename_all: Option<RenameRule>,
}

/// A field in a struct
#[derive(Debug, Clone)]
pub struct Field {
    /// Rust field name (snake_case)
    pub rust_name: String,
    /// JSON field name (after serde renaming, e.g. camelCase)
    pub json_name: String,
    /// Field type
    pub field_type: RustType,
}

/// A variant in an enum
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// Rust variant name
    pub rust_name: String,
    /// JSON variant name (after serde renaming)
    pub json_name: String,
}

/// Rust type representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustType {
    /// i32, i64
    Int,
    /// f32, f64
    Float,
    /// bool
    Bool,
    /// String
    String,
    /// chrono::DateTime<Utc> (serialized as ISO 8601 string)
    DateTime,
    /// uuid::Uuid (mapped to MerchantFacingId in PureScript)
    Uuid,
    /// rust_decimal::Decimal (serialized as number)
    Decimal,
    /// serde_json::Value (arbitrary JSON)
    JsonValue,
    /// Option<T>
    Option(Box<RustType>),
    /// Vec<T>
    Vec(Box<RustType>),
    /// Custom type (struct or enum name)
    Custom(String),
    /// Unsupported type
    Unsupported(String),
}

impl RustType {
    /// Collect all custom type names referenced by this type
    pub fn collect_custom_types(&self) -> Vec<String> {
        let mut types = Vec::new();
        match self {
            RustType::Custom(name) => {
                types.push(name.clone());
            }
            RustType::Option(inner) | RustType::Vec(inner) => {
                types.extend(inner.collect_custom_types());
            }
            _ => {}
        }
        types
    }
}

impl TypeDefinition {
    /// Collect all custom type names referenced by this type definition
    pub fn collect_dependencies(&self) -> Vec<String> {
        let mut deps = Vec::new();

        match &self.kind {
            TypeKind::Struct(struct_type) => {
                for field in &struct_type.fields {
                    deps.extend(field.field_type.collect_custom_types());
                }
            }
            TypeKind::Enum(_) => {
                // Enums don't reference other types
            }
        }

        deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_custom_types_primitive() {
        let rust_type = RustType::Int;
        let types = rust_type.collect_custom_types();
        assert!(
            types.is_empty(),
            "Primitive types should have no dependencies"
        );
    }

    #[test]
    fn test_collect_custom_types_custom() {
        let rust_type = RustType::Custom("Role".to_string());
        let types = rust_type.collect_custom_types();
        assert_eq!(types, vec!["Role"]);
    }

    #[test]
    fn test_collect_custom_types_option() {
        let rust_type = RustType::Option(Box::new(RustType::Custom("User".to_string())));
        let types = rust_type.collect_custom_types();
        assert_eq!(types, vec!["User"]);
    }

    #[test]
    fn test_collect_custom_types_vec() {
        let rust_type = RustType::Vec(Box::new(RustType::Custom("Product".to_string())));
        let types = rust_type.collect_custom_types();
        assert_eq!(types, vec!["Product"]);
    }

    #[test]
    fn test_collect_custom_types_nested() {
        let rust_type = RustType::Vec(Box::new(RustType::Option(Box::new(RustType::Custom(
            "Item".to_string(),
        )))));
        let types = rust_type.collect_custom_types();
        assert_eq!(types, vec!["Item"]);
    }

    #[test]
    fn test_collect_dependencies_struct() {
        let type_def = TypeDefinition {
            name: "User".to_string(),
            source_file: PathBuf::from("test.rs"),
            line: 0,
            kind: TypeKind::Struct(StructType {
                fields: vec![
                    Field {
                        rust_name: "role".to_string(),
                        json_name: "role".to_string(),
                        field_type: RustType::Custom("Role".to_string()),
                    },
                    Field {
                        rust_name: "status".to_string(),
                        json_name: "status".to_string(),
                        field_type: RustType::Custom("Status".to_string()),
                    },
                ],
                rename_all: None,
            }),
            serde_attrs: SerdeAttrs::default(),
        };

        let deps = type_def.collect_dependencies();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"Role".to_string()));
        assert!(deps.contains(&"Status".to_string()));
    }

    #[test]
    fn test_collect_dependencies_enum() {
        let type_def = TypeDefinition {
            name: "Status".to_string(),
            source_file: PathBuf::from("test.rs"),
            line: 0,
            kind: TypeKind::Enum(EnumType {
                variants: vec![],
                rename_all: None,
            }),
            serde_attrs: SerdeAttrs::default(),
        };

        let deps = type_def.collect_dependencies();
        assert!(deps.is_empty(), "Enums should have no dependencies");
    }
}
