# Cclab Aurora

## Brief

Cclab Aurora is the planned Rust library surface for diagram and specification
generation.

The current checkout contains only the crate manifest and this README. No
`src/lib.rs`, `src/main.rs`, `[lib]`, or `[[bin]]` target is present yet, so the
capability map records the manifest-level product intent as blocked and does not
claim an implemented API.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Diagram And Specification Generation Library | - | planned | failing | smoke | not_ready | manifest describes the library promise, but the crate has no Rust target yet |

### Diagram And Specification Generation Library

ID: diagram-and-specification-generation-library
Type: DeveloperTool
Surfaces: Rust API: `cclab_aurora` crate - planned diagram/specification generation library API
EC Dimensions: behavior: `cargo test --manifest-path crates/cclab-aurora/Cargo.toml` - currently blocked because the manifest has no `src/lib.rs`, `src/main.rs`, `[lib]`, or `[[bin]]` target
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cclab Aurora should provide a Rust library for generating diagram and specification artifacts from structured inputs.
Gate Inventory: crates/cclab-aurora/Cargo.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Library target restoration | epic | - | planned | failing | smoke | crates/cclab-aurora/Cargo.toml |
