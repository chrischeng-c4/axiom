---
id: libs-compass-src-server-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/server/mod.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/server/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/server/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `auto_discover` | libs/compass/src/server/mod.rs | module | pub | 8 | pub mod auto_discover; |
| `daemon` | libs/compass/src/server/mod.rs | module | pub | 9 | pub mod daemon; |
| `disk_cache` | libs/compass/src/server/mod.rs | module | pub | 10 | pub mod disk_cache; |
| `handler` | libs/compass/src/server/mod.rs | module | pub | 11 | pub mod handler; |
| `incremental` | libs/compass/src/server/mod.rs | module | pub | 12 | pub mod incremental; |
| `protocol` | libs/compass/src/server/mod.rs | module | pub | 13 | pub mod protocol; |
| `watch_bridge` | libs/compass/src/server/mod.rs | module | pub | 14 | pub mod watch_bridge; |
| `ArgusDaemon` | libs/compass/src/server/mod.rs | re-export | pub | 19 | pub use daemon::{ArgusDaemon, DaemonClient, DaemonConfig}; |
| `DaemonClient` | libs/compass/src/server/mod.rs | re-export | pub | 19 | pub use daemon::{ArgusDaemon, DaemonClient, DaemonConfig}; |
| `DaemonConfig` | libs/compass/src/server/mod.rs | re-export | pub | 19 | pub use daemon::{ArgusDaemon, DaemonClient, DaemonConfig}; |
| `RequestHandler` | libs/compass/src/server/mod.rs | re-export | pub | 20 | pub use handler::RequestHandler; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Argus Daemon Server
//!
//! Provides a long-running daemon for code analysis with:
//! - In-memory code index
//! - File watching with incremental updates
//! - JSON-RPC over Unix socket

pub mod auto_discover;
pub mod daemon;
pub mod disk_cache;
pub mod handler;
pub mod incremental;
pub mod protocol;
pub mod watch_bridge;

#[cfg(test)]
mod tests;

pub use daemon::{ArgusDaemon, DaemonClient, DaemonConfig};
pub use handler::RequestHandler;
pub use incremental::{
    DependencyGraph, DirtyFileTracker, FileChangeKind, IncrementalUpdateManager,
};
pub use protocol::{
    CheckResult, DiagnosticInfo, IndexStatus, Request, Response, RpcError, SymbolInfo,
};
pub use watch_bridge::{
    spawn_watch_bridge, AsyncWatchBridgeBuilder, BridgeEvent, WatchBridge, WatchBridgeConfig,
    WatchBridgeHandle,
};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/server/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/server/mod.rs` captured during libs codegen standardization.
```
