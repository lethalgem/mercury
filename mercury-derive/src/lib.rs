//! Mercury Derive Macro
//!
//! Provides the `#[mercury]` attribute macro for marking Rust types to be
//! generated as PureScript types with Argonaut JSON codecs.
//!
//! This is a no-op macro at compile time - it simply passes through the
//! annotated item unchanged. The actual code generation is performed by
//! the `cargo mercury generate` CLI tool, which scans source files for
//! the `#[mercury]` attribute.
//!
//! # Example
//!
//! ```rust,ignore
//! use mercury_derive::mercury;
//! use serde::{Deserialize, Serialize};
//!
//! #[mercury]
//! #[derive(Debug, Serialize, Deserialize)]
//! pub struct User {
//!     pub id: i32,
//!     pub name: String,
//! }
//! ```

use proc_macro::TokenStream;

/// Marker attribute for types to be generated as PureScript.
///
/// This attribute does nothing at compile time - it's only used by the
/// `cargo mercury generate` command to identify which types should have
/// PureScript definitions and JSON codecs generated.
///
/// # Usage
///
/// Apply this attribute to structs and enums that are sent between your
/// Rust backend and PureScript frontend:
///
/// ```rust,ignore
/// use mercury_derive::mercury;
/// use serde::{Deserialize, Serialize};
///
/// #[mercury]
/// #[derive(Serialize, Deserialize)]
/// pub struct AuthResponse {
///     pub token: String,
///     pub user_id: i32,
/// }
///
/// #[mercury]
/// #[derive(Serialize, Deserialize)]
/// pub enum Role {
///     Admin,
///     User,
/// }
/// ```
///
/// After annotating your types, run:
///
/// ```bash
/// cargo mercury generate
/// ```
///
/// This will generate PureScript type definitions and Argonaut codecs in
/// your frontend's `Generated/` directory.
#[proc_macro_attribute]
pub fn mercury(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This is a no-op macro - it just passes through the original item unchanged.
    // The actual code generation happens in the mercury CLI tool by scanning
    // source files for the #[mercury] attribute.
    item
}
