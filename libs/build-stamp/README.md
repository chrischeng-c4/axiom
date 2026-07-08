# build-stamp

## Brief

`build-stamp` is the shared build-script helper for service and CLI crates. It
emits git SHA, build timestamp, and target-triple environment variables through
Cargo `cargo:rustc-env` directives so runtime binaries can expose consistent
version metadata without copying build-script logic.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Build Script Version Stamp | - | implemented | verified | smoke | ready | emits `<PREFIX>_GIT_SHA`, `<PREFIX>_BUILT_AT`, and `<PREFIX>_TARGET` |

### Build Script Version Stamp

ID: build-script-version-stamp
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `build_stamp::stamp(prefix)`; Cargo build-script stdout directives.
EC Dimensions: behavior: `cargo test -p build-stamp` - unit coverage for SHA decoding, timestamp formatting, target fallback, and rerun hints
Required Verification: smoke
Promise:
Build scripts can call `stamp(prefix)` to emit consistent best-effort build
metadata without failing source-tarball, non-git, or missing-target builds.
Gate Inventory: `cargo test -p build-stamp`; libs/build-stamp/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| build-script-version-stamp-contract | epic | - | implemented | verified | smoke | `cargo test -p build-stamp`; libs/build-stamp/src/lib.rs |
