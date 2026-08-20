# build-stamp

## Brief

`build-stamp` is the shared build-script helper for service and CLI crates. It
emits git SHA, build timestamp, and target-triple environment variables through
Cargo `cargo:rustc-env` directives so runtime binaries can expose consistent
version metadata without copying build-script logic.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Build Script Version Stamp | - | emits `<PREFIX>_GIT_SHA`, `<PREFIX>_BUILT_AT`, and `<PREFIX>_TARGET` |

### Build Script Version Stamp

Build scripts can call `stamp(prefix)` to emit consistent best-effort build
metadata without failing source-tarball, non-git, or missing-target builds.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `build_stamp::stamp(prefix)`; Cargo build-script stdout
  directives.
- Gate — behavior: `cargo test -p build-stamp` - unit coverage for SHA
  decoding, timestamp formatting, target fallback, and rerun hints
- Gate: `cargo test -p build-stamp`
- Source: `libs/build-stamp/src/lib.rs`
- Evidence: `cargo test -p build-stamp`; libs/build-stamp/src/lib.rs
