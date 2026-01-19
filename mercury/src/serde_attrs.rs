//! Serde attribute parsing and handling

use quote::ToTokens;
use syn::Attribute;

/// Serde rename strategy for structs/enums
///
/// Defines the various case transformation rules supported by serde's `rename_all`
/// attribute. These rules are applied to field names and enum variants when
/// serializing to/from JSON.
///
/// # Examples
///
/// ```rust,ignore
/// #[serde(rename_all = "camelCase")]
/// pub struct User {
///     pub user_name: String,  // Becomes "userName" in JSON
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameRule {
    /// No transformation
    None,
    /// snake_case (default Rust style)
    SnakeCase,
    /// camelCase (JavaScript/JSON style)
    CamelCase,
    /// PascalCase (also called UpperCamelCase)
    PascalCase,
    /// SCREAMING_SNAKE_CASE
    ScreamingSnakeCase,
    /// kebab-case
    KebabCase,
    /// lowercase
    Lowercase,
    /// UPPERCASE
    Uppercase,
}

impl RenameRule {
    /// Parse rename_all value from serde attribute
    ///
    /// # Arguments
    ///
    /// * `s` - The rename rule string (e.g., "camelCase", "snake_case")
    ///
    /// # Returns
    ///
    /// Returns `Some(RenameRule)` if the string is a valid rule, `None` otherwise
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "snake_case" => Some(RenameRule::SnakeCase),
            "camelCase" => Some(RenameRule::CamelCase),
            "PascalCase" => Some(RenameRule::PascalCase),
            "SCREAMING_SNAKE_CASE" => Some(RenameRule::ScreamingSnakeCase),
            "kebab-case" => Some(RenameRule::KebabCase),
            "lowercase" => Some(RenameRule::Lowercase),
            "UPPERCASE" => Some(RenameRule::Uppercase),
            _ => None,
        }
    }

    /// Apply the rename rule to a string
    pub fn apply(&self, name: &str) -> String {
        match self {
            RenameRule::None => name.to_string(),
            RenameRule::SnakeCase => to_snake_case(name),
            RenameRule::CamelCase => to_camel_case(name),
            RenameRule::PascalCase => to_pascal_case(name),
            RenameRule::ScreamingSnakeCase => to_screaming_snake_case(name),
            RenameRule::KebabCase => to_kebab_case(name),
            RenameRule::Lowercase => name.to_lowercase(),
            RenameRule::Uppercase => name.to_uppercase(),
        }
    }
}

/// Serde attributes found on a type or field
///
/// Represents all serde attributes that affect JSON serialization and how Mercury
/// should generate PureScript types. This includes rename rules, field-level
/// renames, and skip directives.
///
/// # Examples
///
/// ```rust,ignore
/// // Type with rename_all
/// #[serde(rename_all = "camelCase")]
/// pub struct User { ... }
///
/// // Field with explicit rename
/// pub struct User {
///     #[serde(rename = "id")]
///     pub user_id: i32,
/// }
///
/// // Skipped field
/// pub struct User {
///     pub email: String,
///     #[serde(skip_serializing)]
///     pub password_hash: String,  // Won't appear in generated PureScript
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct SerdeAttrs {
    /// rename_all rule (for structs/enums)
    pub rename_all: Option<RenameRule>,
    /// Explicit rename for a single field/variant
    pub rename: Option<String>,
    /// Skip serialization
    pub skip_serializing: bool,
    /// Skip deserialization
    pub skip_deserializing: bool,
    /// Skip both
    pub skip: bool,
}

impl SerdeAttrs {
    /// Should this field/variant be skipped in generation?
    pub fn should_skip(&self) -> bool {
        self.skip || self.skip_serializing || self.skip_deserializing
    }

    /// Get the final JSON name for a field, applying rename rules
    pub fn get_json_name(&self, rust_name: &str, parent_rename_all: Option<RenameRule>) -> String {
        // Explicit rename takes precedence
        if let Some(ref rename) = self.rename {
            return rename.clone();
        }

        // Apply self's rename_all rule (for type-level)
        if let Some(rule) = self.rename_all {
            return rule.apply(rust_name);
        }

        // Apply parent's rename_all rule (for fields)
        if let Some(rule) = parent_rename_all {
            return rule.apply(rust_name);
        }

        // No transformation
        rust_name.to_string()
    }
}

/// Parse serde attributes from syn::Attribute list
///
/// Extracts all relevant serde attributes from a type or field's attribute list.
/// Handles `rename_all`, `rename`, `skip`, `skip_serializing`, and `skip_deserializing`.
///
/// # Arguments
///
/// * `attrs` - The list of syn attributes from a type or field
///
/// # Returns
///
/// Returns a `SerdeAttrs` struct containing all parsed serde configuration
///
/// # Examples
///
/// ```rust,ignore
/// use syn::parse_quote;
///
/// let input: syn::ItemStruct = parse_quote! {
///     #[serde(rename_all = "camelCase")]
///     pub struct User {
///         pub user_name: String,
///     }
/// };
///
/// let attrs = parse_serde_attrs(&input.attrs);
/// assert_eq!(attrs.rename_all, Some(RenameRule::CamelCase));
/// ```
pub fn parse_serde_attrs(attrs: &[Attribute]) -> SerdeAttrs {
    let mut result = SerdeAttrs::default();

    for attr in attrs {
        // Check if this is a serde attribute
        if !attr.path().is_ident("serde") {
            continue;
        }

        // Parse the attribute - convert the whole thing to a string and parse it
        let attr_str = attr.meta.to_token_stream().to_string();

        // Parse individual key-value pairs by splitting on commas (rough but works)
        for part in attr_str.split(',') {
            let part = part.trim();

            // Parse rename_all = "..."
            if part.contains("rename_all") {
                if let Some(rule_str) = extract_string_value(part) {
                    result.rename_all = RenameRule::parse(&rule_str);
                }
            }
            // Parse rename = "..." (must check after rename_all)
            else if part.contains("rename") && !part.contains("rename_all") {
                if let Some(rename_value) = extract_string_value(part) {
                    result.rename = Some(rename_value);
                }
            }
            // Parse skip_serializing
            else if part.contains("skip_serializing") {
                result.skip_serializing = true;
            }
            // Parse skip_deserializing
            else if part.contains("skip_deserializing") {
                result.skip_deserializing = true;
            }
            // Parse skip
            else if part == "skip" {
                result.skip = true;
            }
        }
    }

    result
}

/// Extract string value from "key = \"value\"" format
fn extract_string_value(s: &str) -> Option<String> {
    // Find the = sign
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() < 2 {
        return None;
    }

    // Extract the value part and remove quotes, spaces, and parens
    let value_part = parts[1].trim();
    let value = value_part.trim_matches(|c: char| c == '"' || c == ' ' || c == ')' || c == '(');
    Some(value.to_string())
}

// Case conversion functions

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_lower = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && prev_is_lower {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_is_lower = false;
        } else {
            result.push(ch);
            prev_is_lower = ch.is_lowercase();
        }
    }

    result
}

fn to_camel_case(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    let mut result = String::new();

    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            // First part stays lowercase
            result.push_str(part);
        } else {
            // Capitalize first letter of subsequent parts
            if let Some(first_char) = part.chars().next() {
                result.push(first_char.to_uppercase().next().unwrap());
                result.push_str(&part[1..]);
            }
        }
    }

    result
}

fn to_pascal_case(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    let mut result = String::new();

    for part in parts {
        if let Some(first_char) = part.chars().next() {
            result.push(first_char.to_uppercase().next().unwrap());
            result.push_str(&part[1..]);
        }
    }

    result
}

fn to_screaming_snake_case(s: &str) -> String {
    to_snake_case(s).to_uppercase()
}

fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_rule_parse() {
        assert_eq!(RenameRule::parse("camelCase"), Some(RenameRule::CamelCase));
        assert_eq!(RenameRule::parse("lowercase"), Some(RenameRule::Lowercase));
        assert_eq!(RenameRule::parse("invalid"), None);
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("user_name"), "userName");
        assert_eq!(to_camel_case("is_active"), "isActive");
        assert_eq!(to_camel_case("id"), "id");
        assert_eq!(to_camel_case("user_id"), "userId");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("userName"), "user_name");
        assert_eq!(to_snake_case("isActive"), "is_active");
        assert_eq!(to_snake_case("UserId"), "user_id");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user_name"), "UserName");
        assert_eq!(to_pascal_case("is_active"), "IsActive");
    }

    #[test]
    fn test_to_lowercase() {
        let rule = RenameRule::Lowercase;
        assert_eq!(rule.apply("Active"), "active");
        assert_eq!(rule.apply("Pending"), "pending");
    }

    #[test]
    fn test_rename_all_camel_case() {
        let attrs = SerdeAttrs {
            rename_all: Some(RenameRule::CamelCase),
            ..Default::default()
        };

        assert_eq!(attrs.get_json_name("user_name", None), "userName");
        assert_eq!(attrs.get_json_name("is_default", None), "isDefault");
    }

    #[test]
    fn test_explicit_rename_overrides_rename_all() {
        let attrs = SerdeAttrs {
            rename: Some("customName".to_string()),
            ..Default::default()
        };

        assert_eq!(
            attrs.get_json_name("user_name", Some(RenameRule::CamelCase)),
            "customName"
        );
    }

    #[test]
    fn test_should_skip() {
        let mut attrs = SerdeAttrs::default();
        assert!(!attrs.should_skip());

        attrs.skip = true;
        assert!(attrs.should_skip());

        attrs.skip = false;
        attrs.skip_serializing = true;
        assert!(attrs.should_skip());
    }
}
