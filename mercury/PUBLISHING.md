# Publishing Mercury to Crates.io

## Prerequisites

1. **Create crates.io account**: https://crates.io/
2. **Get API token**: https://crates.io/settings/tokens
3. **Login**: `cargo login <your-token>`

## Pre-Publication Checklist

### 1. Update Cargo.toml Metadata

Add to each crate's `Cargo.toml`:

```toml
[package]
name = "mercury"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "Automatically generate PureScript types and Argonaut codecs from Rust"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/mercury"
homepage = "https://github.com/yourusername/mercury"
documentation = "https://docs.rs/mercury"
keywords = ["purescript", "codegen", "ffi", "webassembly", "typescript"]
categories = ["development-tools", "web-programming"]
readme = "README.md"
```

### 2. Add License Files

```bash
# Add both MIT and Apache 2.0 licenses (Rust standard)
touch LICENSE-MIT LICENSE-APACHE
```

### 3. Create README.md

Essential sections:
- Quick example (copy from CLAUDE.md)
- Installation instructions
- Supported features
- Limitations
- Contributing guidelines

### 4. Ensure Tests Pass

```bash
cargo test --all
cargo clippy --all -- -D warnings
cargo fmt --all -- --check
```

### 5. Build Documentation

```bash
cargo doc --no-deps --open
# Verify it looks good
```

## Publishing Order

Publish in dependency order (dependencies first):

### 1. Publish mercury-derive

```bash
cd lib/mercury-derive
cargo publish --dry-run  # Test first
cargo publish
```

### 2. Update mercury to use published mercury-derive

```toml
# lib/mercury/Cargo.toml
[dependencies]
mercury-derive = "0.1.0"  # Use published version
```

### 3. Publish mercury

```bash
cd lib/mercury
cargo publish --dry-run
cargo publish
```

### 4. Publish mercury-cli

```bash
cd lib/mercury-cli
cargo publish --dry-run
cargo publish
```

## Post-Publication

### 1. Tag Release

```bash
git tag -a v0.1.0 -m "Initial release"
git push origin v0.1.0
```

### 2. Create GitHub Release

- Go to GitHub releases
- Create release from tag
- Add changelog
- Attach binaries (optional)

### 3. Announce

- Reddit: r/rust
- Discourse: users.rust-lang.org
- Twitter/X
- This Week in Rust (submit PR)

## Version Management

Follow semantic versioning:
- `0.x.y` - Pre-1.0, breaking changes allowed in minor versions
- `1.x.y` - Stable API, breaking changes only in major versions

### Releasing Updates

```bash
# Update version in Cargo.toml
# Update CHANGELOG.md
cargo test --all
cargo publish
git tag -a v0.2.0 -m "Release 0.2.0"
git push origin v0.2.0
```

## Common Issues

### "crate already exists"
- You can't unpublish or overwrite
- Bump version and republish

### "token expired"
- Get new token from crates.io
- Run `cargo login <new-token>`

### "documentation failed to build"
- Test with `cargo doc`
- Fix any warnings
- Ensure all dependencies are on crates.io

## Yank a Version (If Needed)

If you publish a broken version:

```bash
cargo yank --vers 0.1.1
# This prevents new users from downloading it
# Existing users can still use it
```

## CI/CD Integration

Add to `.github/workflows/release.yml`:

```yaml
name: Publish to crates.io

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo publish --token ${CRATES_IO_TOKEN}
        env:
          CRATES_IO_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
```

## Resources

- Publishing Guide: https://doc.rust-lang.org/cargo/reference/publishing.html
- Crates.io Policies: https://crates.io/policies
- Package Metadata: https://doc.rust-lang.org/cargo/reference/manifest.html
