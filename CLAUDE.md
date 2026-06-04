# Mercury — Project Notes

## Current published version

**0.5.2** (published to crates.io)

All three crates share the workspace version defined in the root `Cargo.toml`
under `[workspace.package].version`. Keep them in lockstep.

## Crates

This is a Cargo workspace with three published crates:

| Directory       | Crate name (crates.io)  | Depends on        |
|-----------------|-------------------------|-------------------|
| `mercury-derive`| `mercury-derive`        | (none internal)   |
| `mercury`       | `cargo-mercury`         | (none internal)   |
| `mercury-cli`   | `cargo-mercury-cli`     | `cargo-mercury`   |

Note: directory names do not match crate names. `mercury/` publishes as
`cargo-mercury`, `mercury-cli/` publishes as `cargo-mercury-cli`.

## Publishing a new version

1. Pull latest main:
   ```
   git pull --ff-only origin main
   ```
2. Bump the version in the root `Cargo.toml`:
   - `[workspace.package].version`
   - `[workspace.dependencies]` entries for `cargo-mercury` and
     `mercury-derive` (these carry an explicit `version = "..."` that must
     match).
3. Authenticate to crates.io if needed (token rejected = expired):
   ```
   cargo login   # paste a token with publish-new + publish-update scopes
   ```
   Token is stored in `~/.cargo/credentials.toml`, not a `.env`.
4. Publish in dependency order (CLI last, since it depends on `cargo-mercury`):
   ```
   cargo publish -p mercury-derive
   cargo publish -p cargo-mercury
   cargo publish -p cargo-mercury-cli
   ```
   `cargo publish` waits for index propagation between crates automatically.
   Use `--dry-run` first to validate packaging.
