//! Rust AST parser using syn

use crate::error::{MercuryError, Result};
use crate::serde_attrs::parse_serde_attrs;
use crate::types::{EnumType, EnumVariant, Field, RustType, StructType, TypeDefinition, TypeKind};
use std::path::Path;
use syn::{File, Item, ItemEnum, ItemStruct, Type};

/// Parse a Rust source file and extract mercury-annotated types
///
/// # Arguments
///
/// * `file_path` - Path to the Rust source file
/// * `contents` - Contents of the source file
///
/// # Returns
///
/// Returns a vector of `TypeDefinition` structs for each `#[mercury]` annotated type.
pub fn parse_file(file_path: &Path, contents: &str) -> Result<Vec<TypeDefinition>> {
    let syntax_tree: File = syn::parse_str(contents).map_err(|e| MercuryError::ParseError {
        file: file_path.to_path_buf(),
        message: e.to_string(),
    })?;

    let mut type_defs = Vec::new();

    for item in syntax_tree.items {
        match item {
            Item::Struct(item_struct) => {
                if has_mercury_attribute(&item_struct.attrs) {
                    type_defs.push(parse_struct(file_path, item_struct)?);
                }
            }
            Item::Enum(item_enum) => {
                if has_mercury_attribute(&item_enum.attrs) {
                    type_defs.push(parse_enum(file_path, item_enum)?);
                }
            }
            _ => {}
        }
    }

    Ok(type_defs)
}

/// Check if attributes contain #[mercury]
fn has_mercury_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .iter()
            .any(|seg| seg.ident == "mercury")
    })
}

/// Parse a struct item
fn parse_struct(file_path: &Path, item: ItemStruct) -> Result<TypeDefinition> {
    let name = item.ident.to_string();
    let line = item.ident.span().start().line;

    // Parse serde attributes on the struct
    let serde_attrs = parse_serde_attrs(&item.attrs);
    let rename_all = serde_attrs.rename_all;

    let fields = match item.fields {
        syn::Fields::Named(fields_named) => {
            let mut parsed_fields = Vec::new();
            for field in fields_named.named {
                let field_name = field.ident.as_ref().unwrap().to_string();
                let field_type = parse_type(&field.ty);

                // Parse field-level serde attributes
                let field_serde_attrs = parse_serde_attrs(&field.attrs);

                // Skip fields marked with skip/skip_serializing/skip_deserializing
                if field_serde_attrs.should_skip() {
                    continue;
                }

                // Get the JSON field name (applying rename rules)
                let json_name = field_serde_attrs.get_json_name(&field_name, rename_all);

                parsed_fields.push(Field {
                    rust_name: field_name,
                    json_name,
                    field_type,
                });
            }
            parsed_fields
        }
        syn::Fields::Unnamed(_) => {
            // Tuple structs not supported yet
            vec![]
        }
        syn::Fields::Unit => vec![],
    };

    Ok(TypeDefinition {
        name,
        source_file: file_path.to_path_buf(),
        line,
        kind: TypeKind::Struct(StructType { fields, rename_all }),
        serde_attrs,
    })
}

/// Parse an enum item
fn parse_enum(file_path: &Path, item: ItemEnum) -> Result<TypeDefinition> {
    let name = item.ident.to_string();
    let line = item.ident.span().start().line;

    // Parse serde attributes on the enum
    let serde_attrs = parse_serde_attrs(&item.attrs);
    let rename_all = serde_attrs.rename_all;

    let variants = item
        .variants
        .into_iter()
        .map(|variant| {
            let rust_name = variant.ident.to_string();

            // Parse variant-level serde attributes
            let variant_serde_attrs = parse_serde_attrs(&variant.attrs);

            // Get the JSON variant name (applying rename rules)
            let json_name = variant_serde_attrs.get_json_name(&rust_name, rename_all);

            EnumVariant {
                rust_name,
                json_name,
            }
        })
        .collect();

    Ok(TypeDefinition {
        name,
        source_file: file_path.to_path_buf(),
        line,
        kind: TypeKind::Enum(EnumType {
            variants,
            rename_all,
        }),
        serde_attrs,
    })
}

/// Parse a Rust type into our internal representation
fn parse_type(ty: &Type) -> RustType {
    match ty {
        Type::Path(type_path) => {
            let segments = &type_path.path.segments;
            if segments.is_empty() {
                return RustType::Unsupported("empty path".to_string());
            }

            // Get the full path as a string for matching qualified types
            let _full_path = segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            let last_segment = &segments.last().unwrap();
            let type_name = last_segment.ident.to_string();

            // Handle chrono::DateTime<Utc> -> String
            // Accept both "chrono::DateTime" and bare "DateTime" (after import)
            if type_name == "DateTime" {
                return RustType::DateTime;
            }

            // Handle uuid::Uuid -> MerchantFacingId
            // Accept both "uuid::Uuid" and bare "Uuid" (after import)
            if type_name == "Uuid" {
                return RustType::Uuid;
            }

            // Handle primitive types
            match type_name.as_str() {
                "i32" | "i64" => RustType::Int,
                "f32" | "f64" => RustType::Float,
                "bool" => RustType::Bool,
                "String" => RustType::String,
                "Option" => {
                    // Extract inner type from Option<T>
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        return RustType::Option(Box::new(parse_type(inner_ty)));
                    }
                    RustType::Unsupported("Option without type argument".to_string())
                }
                "Vec" => {
                    // Extract inner type from Vec<T>
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        return RustType::Vec(Box::new(parse_type(inner_ty)));
                    }
                    RustType::Unsupported("Vec without type argument".to_string())
                }
                _ => RustType::Custom(type_name),
            }
        }
        _ => RustType::Unsupported(format!("{:?}", ty)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_simple_struct() {
        let source = r#"
            #[mercury]
            pub struct User {
                pub id: i32,
                pub name: String,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "User");

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            assert_eq!(struct_type.fields.len(), 2);
            assert_eq!(struct_type.fields[0].rust_name, "id");
            assert_eq!(struct_type.fields[0].field_type, RustType::Int);
            assert_eq!(struct_type.fields[1].rust_name, "name");
            assert_eq!(struct_type.fields[1].field_type, RustType::String);
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_simple_enum() {
        let source = r#"
            #[mercury]
            pub enum Status {
                Active,
                Archived,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Status");

        if let TypeKind::Enum(enum_type) = &result[0].kind {
            assert_eq!(enum_type.variants.len(), 2);
            assert_eq!(enum_type.variants[0].rust_name, "Active");
            assert_eq!(enum_type.variants[1].rust_name, "Archived");
        } else {
            panic!("Expected enum type");
        }
    }

    #[test]
    fn test_parse_ignores_non_mercury_types() {
        let source = r#"
            pub struct Foo {
                pub id: i32,
            }

            #[mercury]
            pub struct Bar {
                pub name: String,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Bar");
    }

    #[test]
    fn test_parse_option_type() {
        let source = r#"
            #[mercury]
            pub struct User {
                pub name: String,
                pub email: Option<String>,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            assert_eq!(
                struct_type.fields[1].field_type,
                RustType::Option(Box::new(RustType::String))
            );
        }
    }

    #[test]
    fn test_parse_vec_type() {
        let source = r#"
            #[mercury]
            pub struct Product {
                pub tags: Vec<String>,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            assert_eq!(
                struct_type.fields[0].field_type,
                RustType::Vec(Box::new(RustType::String))
            );
        }
    }

    #[test]
    fn test_parse_serde_rename_all_camel_case() {
        let source = r#"
            #[mercury]
            #[serde(rename_all = "camelCase")]
            pub struct User {
                pub user_name: String,
                pub is_active: bool,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            assert_eq!(struct_type.fields[0].json_name, "userName");
            assert_eq!(struct_type.fields[1].json_name, "isActive");
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_serde_skip_field() {
        let source = r#"
            #[mercury]
            pub struct User {
                pub name: String,
                #[serde(skip_serializing)]
                pub password: String,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            // Only one field should be present (password was skipped)
            assert_eq!(struct_type.fields.len(), 1);
            assert_eq!(struct_type.fields[0].rust_name, "name");
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_enum_with_lowercase() {
        let source = r#"
            #[mercury]
            #[serde(rename_all = "lowercase")]
            pub enum EmailStatus {
                Sent,
                Failed,
                Pending,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Enum(enum_type) = &result[0].kind {
            assert_eq!(enum_type.variants[0].json_name, "sent");
            assert_eq!(enum_type.variants[1].json_name, "failed");
            assert_eq!(enum_type.variants[2].json_name, "pending");
        } else {
            panic!("Expected enum type");
        }
    }

    #[test]
    fn test_parse_field_rename_override() {
        let source = r#"
            #[mercury]
            #[serde(rename_all = "camelCase")]
            pub struct User {
                pub user_name: String,
                #[serde(rename = "custom")]
                pub is_active: bool,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            assert_eq!(struct_type.fields[0].json_name, "userName");
            assert_eq!(struct_type.fields[1].json_name, "custom");
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_datetime_type() {
        let source = r#"
            use chrono::{DateTime, Utc};

            #[mercury]
            pub struct Event {
                pub created_at: DateTime<Utc>,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            assert_eq!(struct_type.fields[0].field_type, RustType::DateTime);
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_uuid_type() {
        let source = r#"
            use uuid::Uuid;

            #[mercury]
            pub struct Record {
                pub id: Uuid,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            assert_eq!(struct_type.fields[0].field_type, RustType::Uuid);
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_nested_datetime() {
        let source = r#"
            use chrono::{DateTime, Utc};

            #[mercury]
            pub struct Event {
                pub timestamps: Vec<DateTime<Utc>>,
                pub deleted_at: Option<DateTime<Utc>>,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            // timestamps: Vec<DateTime>
            assert_eq!(
                struct_type.fields[0].field_type,
                RustType::Vec(Box::new(RustType::DateTime))
            );
            // deleted_at: Option<DateTime>
            assert_eq!(
                struct_type.fields[1].field_type,
                RustType::Option(Box::new(RustType::DateTime))
            );
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_nested_uuid() {
        let source = r#"
            use uuid::Uuid;

            #[mercury]
            pub struct Record {
                pub ids: Vec<Uuid>,
                pub parent_id: Option<Uuid>,
            }
        "#;

        let result = parse_file(&PathBuf::from("test.rs"), source).unwrap();

        if let TypeKind::Struct(struct_type) = &result[0].kind {
            // ids: Vec<Uuid>
            assert_eq!(
                struct_type.fields[0].field_type,
                RustType::Vec(Box::new(RustType::Uuid))
            );
            // parent_id: Option<Uuid>
            assert_eq!(
                struct_type.fields[1].field_type,
                RustType::Option(Box::new(RustType::Uuid))
            );
        } else {
            panic!("Expected struct type");
        }
    }
}
