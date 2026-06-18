# Cclab Aurora

## Brief

Cclab Aurora is the Rust library surface for diagram and specification
generation.

The current checkout provides a small structured-input API for Mermaid flowchart
rendering and Markdown specification rendering. OpenAPI/AsyncAPI generation,
template catalogs, and richer diagram families remain outside the current smoke
contract, so this is not yet production-ready documentation generation.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Diagram And Specification Generation Library | - | implemented | verified | smoke | not_ready | basic Mermaid flowchart and Markdown spec rendering library with behavior smoke proof |

### Diagram And Specification Generation Library

ID: diagram-and-specification-generation-library
Type: DeveloperTool
Surfaces: Rust API: `cclab_aurora::{DiagramSpec, DiagramNode, DiagramEdge, SpecificationDoc, render_mermaid_flowchart, render_markdown_spec}`
EC Dimensions: behavior: `cargo test --manifest-path crates/cclab-aurora/Cargo.toml` - Mermaid flowchart, Markdown spec rendering, and validation smoke
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Cclab Aurora provides a Rust library for generating basic Mermaid diagram and Markdown specification artifacts from structured inputs.
Gate Inventory: `cargo test --manifest-path crates/cclab-aurora/Cargo.toml`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Library target behavior smoke | epic | - | implemented | verified | smoke | `cargo test --manifest-path crates/cclab-aurora/Cargo.toml`; crates/cclab-aurora/src/lib.rs |
