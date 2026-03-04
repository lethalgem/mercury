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
/// - `Uuid` → `UUID` (from Data.Uuid)
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
        RustType::Uuid => PureScriptType::Custom("UUID".to_string()),
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
            PureScriptType::Maybe(inner) => {
                if needs_parens(inner) {
                    write!(f, "Maybe ({})", inner)
                } else {
                    write!(f, "Maybe {}", inner)
                }
            }
            PureScriptType::Array(inner) => {
                if needs_parens(inner) {
                    write!(f, "Array ({})", inner)
                } else {
                    write!(f, "Array {}", inner)
                }
            }
            PureScriptType::Json => write!(f, "Json"),
            PureScriptType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Check if a type needs parentheses when nested inside another type constructor
fn needs_parens(ty: &PureScriptType) -> bool {
    matches!(
        ty,
        PureScriptType::Maybe(_) | PureScriptType::Array(_)
    )
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

    #[test]
    fn test_simple_maybe_no_parens() {
        let ty = PureScriptType::Maybe(Box::new(PureScriptType::String));
        assert_eq!(format!("{}", ty), "Maybe String");
    }

    #[test]
    fn test_simple_array_no_parens() {
        let ty = PureScriptType::Array(Box::new(PureScriptType::Int));
        assert_eq!(format!("{}", ty), "Array Int");
    }

    #[test]
    fn test_maybe_array_with_parens() {
        // Option<Vec<T>> should generate Maybe (Array T)
        let ty = PureScriptType::Maybe(Box::new(PureScriptType::Array(Box::new(
            PureScriptType::String,
        ))));
        assert_eq!(format!("{}", ty), "Maybe (Array String)");
    }

    #[test]
    fn test_array_maybe_with_parens() {
        // Vec<Option<T>> should generate Array (Maybe T)
        let ty = PureScriptType::Array(Box::new(PureScriptType::Maybe(Box::new(
            PureScriptType::Int,
        ))));
        assert_eq!(format!("{}", ty), "Array (Maybe Int)");
    }

    #[test]
    fn test_deeply_nested_types() {
        // Option<Vec<Option<T>>> should generate Maybe (Array (Maybe T))
        let ty = PureScriptType::Maybe(Box::new(PureScriptType::Array(Box::new(
            PureScriptType::Maybe(Box::new(PureScriptType::Custom("User".to_string()))),
        ))));
        assert_eq!(format!("{}", ty), "Maybe (Array (Maybe User))");
    }

    #[test]
    fn test_maybe_custom_no_parens() {
        let ty = PureScriptType::Maybe(Box::new(PureScriptType::Custom("User".to_string())));
        assert_eq!(format!("{}", ty), "Maybe User");
    }

    #[test]
    fn test_array_custom_no_parens() {
        let ty = PureScriptType::Array(Box::new(PureScriptType::Custom("Product".to_string())));
        assert_eq!(format!("{}", ty), "Array Product");
    }

    #[test]
    fn test_map_nested_option_vec() {
        // Test that map_type correctly handles Option<Vec<T>>
        let rust_type = RustType::Option(Box::new(RustType::Vec(Box::new(RustType::String))));
        let ps_type = map_type(&rust_type);
        assert_eq!(format!("{}", ps_type), "Maybe (Array String)");
    }

    #[test]
    fn test_map_nested_vec_option() {
        // Test that map_type correctly handles Vec<Option<T>>
        let rust_type = RustType::Vec(Box::new(RustType::Option(Box::new(RustType::Int))));
        let ps_type = map_type(&rust_type);
        assert_eq!(format!("{}", ps_type), "Array (Maybe Int)");
    }
}
