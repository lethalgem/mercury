//! Type analyzer and mapper

use crate::types::RustType;

/// PureScript type representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureScriptType {
    Int,
    Number,
    Boolean,
    String,
    Maybe(Box<PureScriptType>),
    Array(Box<PureScriptType>),
    Json,
    Custom(String),
}

/// Map a Rust type to its PureScript equivalent
///
/// Converts Rust type representations to their corresponding PureScript types,
/// handling primitives, containers (Option/Vec), and custom types. Recursively
/// maps nested types like `Vec<Option<User>>` to `Array (Maybe User)`.
///
/// # Arguments
///
/// * `rust_type` - The Rust type to map to PureScript
///
/// # Returns
///
/// Returns a `PureScriptType` enum representing the equivalent PureScript type
///
/// # Type Mappings
///
/// - `i32`, `i64` → `Int`
/// - `f32`, `f64` → `Number`
/// - `bool` → `Boolean`
/// - `String` → `String`
/// - `DateTime<Utc>` → `String` (ISO 8601)
/// - `Uuid` → `MerchantFacingId` (custom newtype)
/// - `Option<T>` → `Maybe T`
/// - `Vec<T>` → `Array T`
/// - Custom types → Same name
///
/// # Examples
///
/// ```rust,ignore
/// use crate::types::RustType;
/// use crate::analyzer::map_type;
///
/// let rust_type = RustType::Option(Box::new(RustType::String));
/// let ps_type = map_type(&rust_type);
/// // ps_type == PureScriptType::Maybe(Box::new(PureScriptType::String))
/// ```
pub fn map_type(rust_type: &RustType) -> PureScriptType {
    match rust_type {
        RustType::Int => PureScriptType::Int,
        RustType::Float => PureScriptType::Number,
        RustType::Bool => PureScriptType::Boolean,
        RustType::String => PureScriptType::String,
        RustType::DateTime => PureScriptType::String, // ISO 8601 string
        RustType::Uuid => PureScriptType::Custom("MerchantFacingId".to_string()),
        RustType::Decimal => PureScriptType::Number,
        RustType::JsonValue => PureScriptType::Json,
        RustType::Option(inner) => PureScriptType::Maybe(Box::new(map_type(inner))),
        RustType::Vec(inner) => PureScriptType::Array(Box::new(map_type(inner))),
        RustType::Custom(name) => PureScriptType::Custom(name.clone()),
        RustType::Unsupported(s) => {
            // For now, map unsupported to String with a warning
            // In a real implementation, this should error
            eprintln!("Warning: Unsupported type '{}' mapped to String", s);
            PureScriptType::String
        }
    }
}

impl std::fmt::Display for PureScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PureScriptType::Int => write!(f, "Int"),
            PureScriptType::Number => write!(f, "Number"),
            PureScriptType::Boolean => write!(f, "Boolean"),
            PureScriptType::String => write!(f, "String"),
            PureScriptType::Maybe(inner) => write!(f, "Maybe {}", inner),
            PureScriptType::Array(inner) => write!(f, "Array {}", inner),
            PureScriptType::Json => write!(f, "Json"),
            PureScriptType::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_decimal_to_number() {
        assert_eq!(map_type(&RustType::Decimal), PureScriptType::Number);
    }

    #[test]
    fn test_map_json_value_to_json() {
        assert_eq!(map_type(&RustType::JsonValue), PureScriptType::Json);
    }

    #[test]
    fn test_json_display() {
        assert_eq!(format!("{}", PureScriptType::Json), "Json");
    }
}
