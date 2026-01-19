//! Argonaut JSON codec generator for PureScript

use crate::analyzer::map_type;
#[cfg(test)]
use crate::types::Field;
use crate::types::{EnumType, RustType, StructType, TypeDefinition, TypeKind};

/// Generate Argonaut DecodeJson and EncodeJson instances for a type
///
/// Creates type class instances that allow PureScript to serialize and deserialize
/// JSON for the given type. Handles both structs (using record syntax) and enums
/// (using string representations). Optional fields are encoded/decoded with Maybe.
///
/// # Optional Field Handling
///
/// Uses `.:?` (getFieldOptional') for `Option<T>` fields, which treats both missing
/// keys and `null` values as `Nothing`. This matches Rust's serde default behavior
/// where `Option::None` fields are serialized as `null`, or more commonly omitted
/// entirely with `#[serde(skip_serializing_if = "Option::is_none")]`.
///
/// # Arguments
///
/// * `type_def` - The type definition to generate codecs for
///
/// # Returns
///
/// Returns a String containing both DecodeJson and EncodeJson instance declarations
///
/// # Examples
///
/// For a struct:
/// ```rust,ignore
/// #[mercury]
/// pub struct User {
///     pub id: i32,
///     pub email: Option<String>,
/// }
/// ```
///
/// Generates:
/// ```purescript,ignore
/// instance decodeUser :: DecodeJson User where
///   decodeJson json = do
///     obj <- decodeJson json
///     id <- obj .: "id"
///     email <- obj .:? "email"  -- Uses .:? (treats null and missing as Nothing)
///     pure $ User { id, email }
///
/// instance encodeUser :: EncodeJson User where
///   encodeJson (User record) =
///     encodeJson
///       { "id": record.id
///       , "email": record.email
///       }
/// ```
pub fn generate_codecs(type_def: &TypeDefinition) -> String {
    let mut output = String::new();

    match &type_def.kind {
        TypeKind::Struct(struct_type) => {
            output.push_str(&generate_struct_decoder(&type_def.name, struct_type));
            output.push('\n');
            output.push_str(&generate_struct_encoder(&type_def.name, struct_type));
        }
        TypeKind::Enum(enum_type) => {
            output.push_str(&generate_enum_decoder(&type_def.name, enum_type));
            output.push('\n');
            output.push_str(&generate_enum_encoder(&type_def.name, enum_type));
        }
    }

    output
}

/// Generate DecodeJson instance for a struct
fn generate_struct_decoder(name: &str, struct_type: &StructType) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "instance decode{} :: DecodeJson {} where\n",
        name, name
    ));
    output.push_str("  decodeJson json = do\n");
    output.push_str("    obj <- decodeJson json\n");

    // Decode each field
    for field in &struct_type.fields {
        let decode_expr = generate_field_decoder(&field.json_name, &field.field_type);
        output.push_str(&format!("    {} <- {}\n", field.json_name, decode_expr));
    }

    // Return record wrapped in newtype constructor
    output.push_str(&format!("    pure $ {} \n", name));
    output.push_str("      { ");
    for (i, field) in struct_type.fields.iter().enumerate() {
        if i > 0 {
            output.push_str("\n      , ");
        }
        output.push_str(&field.json_name);
    }
    output.push_str("\n      }\n");

    output
}

/// Generate field decoder expression based on type
///
/// Uses `.:?` (getFieldOptional') for Option<T> fields, which treats both
/// missing keys and null values as Nothing. This matches Rust's serde behavior
/// when Option::None fields are omitted from JSON (the default or with
/// skip_serializing_if="Option::is_none").
fn generate_field_decoder(field_name: &str, rust_type: &RustType) -> String {
    let _ps_type = map_type(rust_type);

    match rust_type {
        RustType::Option(_) => {
            // Optional fields use .:? which treats null as missing
            // Matches Rust Option<T> with skip_serializing_if="Option::is_none"
            format!("obj .:? \"{}\"", field_name)
        }
        _ => {
            // Required fields use .:
            format!("obj .: \"{}\"", field_name)
        }
    }
}

/// Generate EncodeJson instance for a struct
fn generate_struct_encoder(name: &str, struct_type: &StructType) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "instance encode{} :: EncodeJson {} where\n",
        name, name
    ));
    output.push_str(&format!("  encodeJson ({} record) =\n", name));
    output.push_str("    encodeJson\n");
    output.push_str("      { ");

    for (i, field) in struct_type.fields.iter().enumerate() {
        if i > 0 {
            output.push_str("\n      , ");
        }
        output.push_str(&format!(
            "\"{}\": record.{}",
            field.json_name, field.json_name
        ));
    }

    output.push_str("\n      }\n");

    output
}

/// Generate DecodeJson instance for an enum
fn generate_enum_decoder(name: &str, enum_type: &EnumType) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "instance decode{} :: DecodeJson {} where\n",
        name, name
    ));
    output.push_str("  decodeJson json = do\n");
    output.push_str("    str <- decodeJson json\n");
    output.push_str("    case str of\n");

    // Add a case for each variant
    for variant in &enum_type.variants {
        output.push_str(&format!(
            "      \"{}\" -> Right {}\n",
            variant.json_name, variant.rust_name
        ));
    }

    // Default case for unknown variants
    output.push_str(&format!(
        "      _ -> Left $ TypeMismatch \"Invalid {}\"\n",
        name
    ));

    output
}

/// Generate EncodeJson instance for an enum
fn generate_enum_encoder(name: &str, enum_type: &EnumType) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "instance encode{} :: EncodeJson {} where\n",
        name, name
    ));
    output.push_str("  encodeJson value =\n");
    output.push_str("    let\n");
    output.push_str("      str = case value of\n");

    // Add a case for each variant
    for variant in &enum_type.variants {
        output.push_str(&format!(
            "        {} -> \"{}\"\n",
            variant.rust_name, variant.json_name
        ));
    }

    output.push_str("    in\n");
    output.push_str("      encodeJson str\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_struct_decoder() {
        let struct_type = StructType {
            fields: vec![
                Field {
                    rust_name: "id".to_string(),
                    json_name: "id".to_string(),
                    field_type: RustType::Int,
                },
                Field {
                    rust_name: "name".to_string(),
                    json_name: "name".to_string(),
                    field_type: RustType::String,
                },
            ],
            rename_all: None,
        };

        let output = generate_struct_decoder("User", &struct_type);

        assert!(output.contains("instance decodeUser :: DecodeJson User where"));
        assert!(output.contains("obj <- decodeJson json"));
        assert!(output.contains("id <- obj .: \"id\""));
        assert!(output.contains("name <- obj .: \"name\""));
        assert!(output.contains("pure $ User"));
    }

    #[test]
    fn test_generate_struct_encoder() {
        let struct_type = StructType {
            fields: vec![Field {
                rust_name: "id".to_string(),
                json_name: "id".to_string(),
                field_type: RustType::Int,
            }],
            rename_all: None,
        };

        let output = generate_struct_encoder("User", &struct_type);

        assert!(output.contains("instance encodeUser :: EncodeJson User where"));
        assert!(output.contains("encodeJson (User record)"));
        assert!(output.contains("\"id\": record.id"));
    }

    #[test]
    fn test_generate_enum_decoder() {
        let enum_type = EnumType {
            variants: vec![
                crate::types::EnumVariant {
                    rust_name: "Active".to_string(),
                    json_name: "Active".to_string(),
                },
                crate::types::EnumVariant {
                    rust_name: "Inactive".to_string(),
                    json_name: "Inactive".to_string(),
                },
            ],
            rename_all: None,
        };

        let output = generate_enum_decoder("Status", &enum_type);

        assert!(output.contains("instance decodeStatus :: DecodeJson Status where"));
        assert!(output.contains("str <- decodeJson json"));
        assert!(output.contains("\"Active\" -> Right Active"));
        assert!(output.contains("\"Inactive\" -> Right Inactive"));
        assert!(output.contains("TypeMismatch"));
    }

    #[test]
    fn test_generate_enum_encoder() {
        let enum_type = EnumType {
            variants: vec![crate::types::EnumVariant {
                rust_name: "Draft".to_string(),
                json_name: "Draft".to_string(),
            }],
            rename_all: None,
        };

        let output = generate_enum_encoder("Status", &enum_type);

        assert!(output.contains("instance encodeStatus :: EncodeJson Status where"));
        assert!(output.contains("Draft -> \"Draft\""));
        assert!(output.contains("encodeJson str"));
    }

    #[test]
    fn test_generate_optional_field() {
        let struct_type = StructType {
            fields: vec![Field {
                rust_name: "email".to_string(),
                json_name: "email".to_string(),
                field_type: RustType::Option(Box::new(RustType::String)),
            }],
            rename_all: None,
        };

        let output = generate_struct_decoder("User", &struct_type);

        // Optional fields should use .:? instead of .:
        assert!(output.contains("email <- obj .:? \"email\""));
    }
}
