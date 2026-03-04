# Mercury

**Automatic PureScript type generation from Rust**

Mercury is a code generator that automatically creates PureScript type definitions and Argonaut JSON codecs from annotated Rust types. It eliminates manual synchronization between your Rust backend and PureScript frontend, preventing type mismatches and reducing boilerplate.

## Features

- **Automatic type generation** - Mark Rust types with `#[mercury]` and get PureScript types
- **Full serde support** - Handles `rename_all`, `rename`, `skip`, and other serde attributes
- **Type-safe JSON codecs** - Generates Argonaut `EncodeJson` and `DecodeJson` instances
- **Cross-module imports** - Automatically generates import statements for type dependencies
- **Advanced type mapping** - Supports `Option<T>`, `Vec<T>`, `DateTime`, `Uuid`, and nested types
- **Multi-module organization** - Outputs organized by source file structure
- **Deterministic output** - Byte-for-byte identical on repeated runs (perfect for git)
- **Production-ready** - Thoroughly tested with 48 tests (45 unit + 3 integration)

## Quick Start

### 1. Add Mercury to your workspace

```toml
# Cargo.toml
[workspace]
members = ["lib/mercury-derive", "lib/mercury"]

[dependencies]
mercury-derive = { path = "lib/mercury-derive" }
```

### 2. Annotate your Rust types

```rust
use mercury_derive::mercury;
use serde::{Deserialize, Serialize};

#[mercury]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    pub product_name: String,
    pub price: i32,
    pub is_active: bool,
    pub tags: Vec<String>,
}

#[mercury]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProductStatus {
    Active,
    Archived,
}
```

### 3. Generate PureScript types

```bash
cargo run --bin mercury -- generate
```

### 4. Use the generated types in PureScript

```purescript
-- frontend/src/Generated/Generated/Models.purs
module Generated.Models where

import Prelude
import Data.Argonaut.Decode.Class (class DecodeJson)
import Data.Argonaut.Encode.Class (class EncodeJson)
import Data.Maybe (Maybe(..))

newtype CreateProductRequest = CreateProductRequest
  { productName :: String
  , price :: Int
  , isActive :: Boolean
  , tags :: Array String
  }

-- Codecs automatically generated!
instance decodeCreateProductRequest :: DecodeJson CreateProductRequest
instance encodeCreateProductRequest :: EncodeJson CreateProductRequest

data ProductStatus = Active | Archived

instance decodeProductStatus :: DecodeJson ProductStatus
instance encodeProductStatus :: EncodeJson ProductStatus
```

## Type Mapping

Mercury maps Rust types to their PureScript equivalents:

| Rust Type               | PureScript Type | Notes                |
| ----------------------- | --------------- | -------------------- |
| `i32`, `i64`            | `Int`           |                      |
| `f32`, `f64`            | `Number`        |                      |
| `bool`                  | `Boolean`       |                      |
| `String`                | `String`        |                      |
| `Option<T>`             | `Maybe T`       | Nullable fields      |
| `Vec<T>`                | `Array T`       |                      |
| `chrono::DateTime<Utc>` | `String`        | ISO 8601 format      |
| `uuid::Uuid`            | `UUID`          | From Data.UUID       |
| `rust_decimal::Decimal` | `Number`        | Serialized as number |
| `serde_json::Value`     | `Json`          | Arbitrary JSON       |
| Custom types            | Same name       | Enums and structs    |

### Nested Types

Mercury handles arbitrarily nested types:

```rust
#[mercury]
pub struct UserList {
    pub users: Vec<Option<User>>,
    pub admin: Option<Admin>,
}
```

Generates:

```purescript
newtype UserList = UserList
  { users :: Array (Maybe User)
  , admin :: Maybe Admin
  }
```

## Serde Attributes

Mercury respects your serde configuration:

### `rename_all`

```rust
#[mercury]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    pub user_id: i32,        // → userId in JSON
    pub created_at: String,  // → createdAt in JSON
}
```

Supported rename rules:

- `camelCase` - `user_name` → `userName`
- `PascalCase` - `user_name` → `UserName`
- `snake_case` - `UserName` → `user_name`
- `SCREAMING_SNAKE_CASE` - `user_name` → `USER_NAME`
- `kebab-case` - `user_name` → `user-name`
- `lowercase` - `UserName` → `username`
- `UPPERCASE` - `user_name` → `USER_NAME`

### `rename`

Override individual field names:

```rust
#[mercury]
pub struct User {
    #[serde(rename = "id")]
    pub user_id: i32,
}
```

### `skip` and `skip_serializing`

Exclude fields from generated types:

```rust
#[mercury]
pub struct User {
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,  // Not included in PureScript
}
```

## Module Organization

Mercury organizes output by source file location:

```
Your Rust project:
├── app/backend/src/models.rs              → Generated.Models
├── lib/constitution/src/models/
│   ├── merchant.rs                        → Generated.Merchant
│   └── product/
│       ├── core.rs                        → Generated.Product.Core
│       └── variant.rs                     → Generated.Product.Variant

Generated PureScript:
frontend/src/Generated/Generated/
├── Models.purs
├── Merchant.purs
└── Product/
    ├── Core.purs
    └── Variant.purs
```

### Cross-Module Imports

Mercury automatically generates import statements when types reference other types:

```rust
// lib/constitution/src/models/merchant.rs
#[mercury]
pub enum MerchantRole { Merchant, Admin }

// app/backend/src/models.rs
#[mercury]
pub struct MerchantInfo {
    pub role: MerchantRole,  // References MerchantRole
}
```

Generates with automatic import:

```purescript
-- Generated.Models
module Generated.Models where

import Generated.Merchant (MerchantRole)  -- Automatically added!

newtype MerchantInfo = MerchantInfo
  { role :: MerchantRole
  }
```

## CLI Usage

### Generate types

```bash
cargo run --bin mercury -- generate
```

Output:

```
✓ Scanning workspace...
✓ Found 25 types in 8 files
✓ Generating PureScript modules...
  Generated.Models (17 types)
  Generated.Merchant (1 type)
  Generated.Product.Core (7 types)

✓ Generated 25 types in 3 modules
✓ Wrote 3 files to frontend/src/Generated/
```

### Options

- `--workspace <path>` - Path to workspace root (default: current directory)
- `--verbose` - Show detailed progress
- `--output <dir>` - Specify output directory (default: `frontend/src/Generated`)

### Check Command

Use the `check` command to verify generated code is up-to-date (useful for CI):

```bash
cargo run --bin mercury -- check
cargo run --bin mercury -- check --fail-on-diff  # Exit with error if out of sync
```

This is useful in CI pipelines to ensure generated types stay synchronized with Rust definitions.

## Generated Code Examples

### Struct with Optional Fields

**Rust:**

```rust
#[mercury]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductRequest {
    pub product_id: i32,
    pub new_name: Option<String>,
    pub new_price: Option<i32>,
}
```

**Generated PureScript:**

```purescript
newtype UpdateProductRequest = UpdateProductRequest
  { productId :: Int
  , newName :: Maybe String
  , newPrice :: Maybe Int
  }

instance decodeUpdateProductRequest :: DecodeJson UpdateProductRequest where
  decodeJson json = do
    obj <- decodeJson json
    productId <- obj .: "productId"
    newName <- obj .:? "newName"      -- Uses .:? for Maybe (treats null and missing as Nothing)
    newPrice <- obj .:? "newPrice"
    pure $ UpdateProductRequest { productId, newName, newPrice }

instance encodeUpdateProductRequest :: EncodeJson UpdateProductRequest where
  encodeJson (UpdateProductRequest record) =
    encodeJson
      { "productId": record.productId
      , "newName": record.newName
      , "newPrice": record.newPrice
      }
```

**Note on Optional Fields:** Mercury uses `.:?` (getFieldOptional') for `Option<T>` fields, which treats both missing JSON keys and `null` values as `Nothing`. This matches Rust's serde behavior when `Option::None` fields are omitted from JSON (the default, or with `#[serde(skip_serializing_if = "Option::is_none")]`).

### Enum

**Rust:**

```rust
#[mercury]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
}
```

**Generated PureScript:**

```purescript
data OrderStatus
  = Pending
  | Confirmed
  | Shipped
  | Delivered

instance decodeOrderStatus :: DecodeJson OrderStatus where
  decodeJson json = do
    str <- decodeJson json
    case str of
      "pending" -> Right Pending
      "confirmed" -> Right Confirmed
      "shipped" -> Right Shipped
      "delivered" -> Right Delivered
      _ -> Left $ TypeMismatch "Invalid OrderStatus"

instance encodeOrderStatus :: EncodeJson OrderStatus where
  encodeJson value =
    let
      str = case value of
        Pending -> "pending"
        Confirmed -> "confirmed"
        Shipped -> "shipped"
        Delivered -> "delivered"
    in
      encodeJson str
```

## Testing

Mercury includes comprehensive testing. See [TESTING.md](TESTING.md) for details.

**Test Coverage:**

- 48 total tests (45 unit + 3 integration)
- Parser tests (10) - Rust AST parsing
- Serde attribute tests (6) - Rename rules and attributes
- Codegen tests (7) - Type and module generation including enum derive instances
- Codec generation tests (5) - Encoder/decoder correctness
- Code generation tests (8) - Type definition output
- Type dependency tests (8) - Cross-module imports
- Integration tests (3) - End-to-end pipeline

Run tests:

```bash
cd lib/mercury
cargo test
```

All tests must pass with 100% success rate.

## Publishing

See [PUBLISHING.md](PUBLISHING.md) for instructions on publishing Mercury to crates.io.

## How It Works

Mercury's pipeline:

1. **Scanner** - Finds all Rust files with `#[mercury]` annotations
2. **Parser** - Uses `syn` crate to parse Rust syntax trees
3. **Analyzer** - Maps Rust types to PureScript equivalents
4. **Codegen** - Generates PureScript type definitions and codecs
5. **Writer** - Organizes output into multi-module structure

## Architecture

```
lib/mercury-derive/    - Procedural macro (#[mercury] attribute)
lib/mercury/
  ├── src/
  │   ├── lib.rs       - Public API and pipeline orchestration
  │   ├── scanner.rs   - Find #[mercury] annotations
  │   ├── parser.rs    - Parse Rust AST with syn
  │   ├── analyzer.rs  - Type mapping logic
  │   ├── codegen.rs   - Generate PureScript types
  │   ├── codec_gen.rs - Generate Argonaut codecs
  │   ├── writer.rs    - File writing and organization
  │   └── error.rs     - Error types and messages
  └── tests/           - Integration tests
```

## Requirements

- **Rust 1.70+** - For Rust development
- **PureScript 0.15+** - For frontend
- **Spago** - PureScript package manager
- **Argonaut** - PureScript JSON library

## Limitations

Mercury currently does not support:

- **Generic types** - `struct Wrapper<T>` not supported
- **Tuple structs** - `struct Point(i32, i32)` not supported
- **HashMap/BTreeMap** - Use `Vec<(K, V)>` instead
- **Enum variants with data** - Only simple enums (no `Variant(Data)`)
- **Recursive types** - Types that reference themselves

These limitations may be addressed in future versions.

## Workflow

### Daily Development

1. Write Rust API types
2. Add `#[mercury]` annotation
3. Run `cargo run --bin mercury -- generate`
4. Commit both Rust and generated PureScript together
5. Review generated code in PRs

### CI Integration

Ensure generated code stays in sync:

```yaml
# .github/workflows/ci.yml
- name: Generate PureScript types
  run: cargo run --bin mercury -- generate

- name: Check for uncommitted changes
  run: |
    if ! git diff --exit-code frontend/src/Generated/; then
      echo "Generated code is out of sync!"
      echo "Run: cargo run --bin mercury -- generate"
      exit 1
    fi
```

## Troubleshooting

### Generated code doesn't compile

1. Check that all referenced types have `#[mercury]`
2. Verify serde attributes are correct
3. Run `cargo test` in mercury crate
4. Check output files for syntax errors

### Import errors in PureScript

1. Ensure all dependencies are generated
2. Check that module names are correct
3. Verify cross-module references work

### Type not found

1. Confirm `#[mercury]` is present
2. Check that type is `pub`
3. Verify file is in workspace

## Contributing

Mercury is currently an internal tool for Merchant Tech. When open-sourced, contributions will be welcome!

## License

TBD - To be determined when open-sourced.

## Related Documentation

- [TESTING.md](TESTING.md) - Comprehensive test documentation
- [PUBLISHING.md](PUBLISHING.md) - Publishing to crates.io guide
- [Merchant Tech Project README](../../README.md) - Main project documentation

## Support

For issues or questions, contact the Merchant Tech development team.

---

**Generated with Mercury** - Keeping Rust and PureScript types in perfect sync.
