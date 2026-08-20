# Cclab Aurora

## Brief

Cclab Aurora is the Rust library surface for diagram and specification
generation.

The current checkout provides a small structured-input API for Mermaid flowchart
rendering and Markdown specification rendering. OpenAPI/AsyncAPI generation,
template catalogs, and richer diagram families remain outside the current smoke
contract, so this is not yet production-ready documentation generation.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Diagram And Specification Generation Library | - | basic Mermaid flowchart and Markdown spec rendering library with behavior smoke proof |

### Diagram And Specification Generation Library

Cclab Aurora provides a Rust library for generating basic Mermaid diagram and
Markdown specification artifacts from structured inputs.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API:
  `cclab_aurora::{DiagramSpec, DiagramNode, DiagramEdge, SpecificationDoc, render_mermaid_flowchart, render_markdown_spec}`
- Gate — behavior: `cargo test --manifest-path crates/cclab-aurora/Cargo.toml`
  - Mermaid flowchart, Markdown spec rendering, and validation smoke
- Gate: `cargo test --manifest-path crates/cclab-aurora/Cargo.toml`
- Evidence: `cargo test --manifest-path crates/cclab-aurora/Cargo.toml`;
  crates/cclab-aurora/src/lib.rs
