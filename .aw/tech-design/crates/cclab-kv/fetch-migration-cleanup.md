# Complete cclab-http to cclab-fetch Migration

Complete migration from legacy cclab-http to cclab-fetch across the workspace. Updated dependencies, refactored imports, and removed deprecated crate.

## Codebase Paths
- crates/cclab-fetch/
- crates/cclab-agent/Cargo.toml
- crates/cclab/Cargo.toml
- crates/cclab-qc/Cargo.toml
- crates/cclab/src/lib.rs
- crates/cclab-agent/src/

## Knowledge Refs
- 40-mcp/dynamic-config.md

## Requirements
- R1: Update Workspace Dependencies (cclab-agent, cclab, cclab-qc Cargo.toml)
- R2: Refactor Source Imports (cclab_http:: -> cclab_fetch::)
- R3: API Parity Assurance (HttpClient, HttpMethod, RequestBuilder)
- R4: Crate Deletion and Cleanup (remove crates/cclab-http, update workspace members)
- R5: Documentation and Metadata Updates