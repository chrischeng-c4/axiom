---
id: libs-compass-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lib.rs`.
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

# Standardized libs/compass/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lib.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check_pipeline` | libs/compass/src/lib.rs | module | pub | 21 | pub mod check_pipeline; |
| `checker` | libs/compass/src/lib.rs | module | pub | 22 | pub mod checker; |
| `core` | libs/compass/src/lib.rs | module | pub | 23 | pub mod core; |
| `diagnostic` | libs/compass/src/lib.rs | module | pub | 25 | pub mod diagnostic; |
| `format` | libs/compass/src/lib.rs | module | pub | 26 | pub mod format; |
| `gen` | libs/compass/src/lib.rs | module | pub | 27 | pub mod gen; |
| `graph` | libs/compass/src/lib.rs | module | pub | 28 | pub mod graph; |
| `lens_error` | libs/compass/src/lib.rs | module | pub | 29 | pub mod lens_error; |
| `lint` | libs/compass/src/lib.rs | module | pub | 30 | pub mod lint; |
| `lsp` | libs/compass/src/lib.rs | module | pub | 31 | pub mod lsp; |
| `outline` | libs/compass/src/lib.rs | module | pub | 32 | pub mod outline; |
| `output` | libs/compass/src/lib.rs | module | pub | 33 | pub mod output; |
| `refactoring` | libs/compass/src/lib.rs | module | pub | 34 | pub mod refactoring; |
| `schemas` | libs/compass/src/lib.rs | module | pub | 35 | pub mod schemas; |
| `search` | libs/compass/src/lib.rs | module | pub | 36 | pub mod search; |
| `semantic` | libs/compass/src/lib.rs | module | pub | 37 | pub mod semantic; |
| `server` | libs/compass/src/lib.rs | module | pub | 38 | pub mod server; |
| `spec` | libs/compass/src/lib.rs | module | pub | 39 | pub mod spec; |
| `storage` | libs/compass/src/lib.rs | module | pub | 40 | pub mod storage; |
| `syntax` | libs/compass/src/lib.rs | module | pub | 41 | pub mod syntax; |
| `type_inference` | libs/compass/src/lib.rs | module | pub | 42 | pub mod type_inference; |
| `watch` | libs/compass/src/lib.rs | module | pub | 43 | pub mod watch; |
| `check_paths` | libs/compass/src/lib.rs | re-export | pub | 46 | pub use checker::{check_paths, check_paths_with_propagation, FileResult, LintConfig}; |
| `check_paths_with_propagation` | libs/compass/src/lib.rs | re-export | pub | 46 | pub use checker::{check_paths, check_paths_with_propagation, FileResult, LintConfig}; |
| `FileResult` | libs/compass/src/lib.rs | re-export | pub | 46 | pub use checker::{check_paths, check_paths_with_propagation, FileResult, LintConfig}; |
| `LintConfig` | libs/compass/src/lib.rs | re-export | pub | 46 | pub use checker::{check_paths, check_paths_with_propagation, FileResult, LintConfig}; |
| `ArgusConfig` | libs/compass/src/lib.rs | re-export | pub | 47 | pub use core::{ArgusConfig, LanguageConfig}; |
| `LanguageConfig` | libs/compass/src/lib.rs | re-export | pub | 47 | pub use core::{ArgusConfig, LanguageConfig}; |
| `Diagnostic` | libs/compass/src/lib.rs | re-export | pub | 48 | pub use diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range}; |
| `DiagnosticCategory` | libs/compass/src/lib.rs | re-export | pub | 48 | pub use diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range}; |
| `DiagnosticSeverity` | libs/compass/src/lib.rs | re-export | pub | 48 | pub use diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range}; |
| `Position` | libs/compass/src/lib.rs | re-export | pub | 48 | pub use diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range}; |
| `Range` | libs/compass/src/lib.rs | re-export | pub | 48 | pub use diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range}; |
| `CodeGenerator` | libs/compass/src/lib.rs | re-export | pub | 49 | pub use gen::{CodeGenerator, GenContext, GenError, GenResult, GeneratedCode, TechStack}; |
| `GenContext` | libs/compass/src/lib.rs | re-export | pub | 49 | pub use gen::{CodeGenerator, GenContext, GenError, GenResult, GeneratedCode, TechStack}; |
| `GenError` | libs/compass/src/lib.rs | re-export | pub | 49 | pub use gen::{CodeGenerator, GenContext, GenError, GenResult, GeneratedCode, TechStack}; |
| `GenResult` | libs/compass/src/lib.rs | re-export | pub | 49 | pub use gen::{CodeGenerator, GenContext, GenError, GenResult, GeneratedCode, TechStack}; |
| `GeneratedCode` | libs/compass/src/lib.rs | re-export | pub | 49 | pub use gen::{CodeGenerator, GenContext, GenError, GenResult, GeneratedCode, TechStack}; |
| `TechStack` | libs/compass/src/lib.rs | re-export | pub | 49 | pub use gen::{CodeGenerator, GenContext, GenError, GenResult, GeneratedCode, TechStack}; |
| `ArgusError` | libs/compass/src/lib.rs | re-export | pub | 50 | pub use lens_error::ArgusError; |
| `Checker` | libs/compass/src/lib.rs | re-export | pub | 51 | pub use lint::{Checker, CheckerRegistry}; |
| `CheckerRegistry` | libs/compass/src/lib.rs | re-export | pub | 51 | pub use lint::{Checker, CheckerRegistry}; |
| `outline` | libs/compass/src/lib.rs | re-export | pub | 52 | pub use outline::{outline, outline_parsed, FunctionDef, FunctionKind}; |
| `outline_parsed` | libs/compass/src/lib.rs | re-export | pub | 52 | pub use outline::{outline, outline_parsed, FunctionDef, FunctionKind}; |
| `FunctionDef` | libs/compass/src/lib.rs | re-export | pub | 52 | pub use outline::{outline, outline_parsed, FunctionDef, FunctionKind}; |
| `FunctionKind` | libs/compass/src/lib.rs | re-export | pub | 52 | pub use outline::{outline, outline_parsed, FunctionDef, FunctionKind}; |
| `OutputFormat` | libs/compass/src/lib.rs | re-export | pub | 53 | pub use output::reporter::{OutputFormat, Reporter}; |
| `Reporter` | libs/compass/src/lib.rs | re-export | pub | 53 | pub use output::reporter::{OutputFormat, Reporter}; |
| `ArgusDaemon` | libs/compass/src/lib.rs | re-export | pub | 54 | pub use server::{ArgusDaemon, DaemonClient, DaemonConfig, RequestHandler}; |
| `DaemonClient` | libs/compass/src/lib.rs | re-export | pub | 54 | pub use server::{ArgusDaemon, DaemonClient, DaemonConfig, RequestHandler}; |
| `DaemonConfig` | libs/compass/src/lib.rs | re-export | pub | 54 | pub use server::{ArgusDaemon, DaemonClient, DaemonConfig, RequestHandler}; |
| `RequestHandler` | libs/compass/src/lib.rs | re-export | pub | 54 | pub use server::{ArgusDaemon, DaemonClient, DaemonConfig, RequestHandler}; |
| `Language` | libs/compass/src/lib.rs | re-export | pub | 55 | pub use syntax::{Language, MultiParser, ParsedFile}; |
| `MultiParser` | libs/compass/src/lib.rs | re-export | pub | 55 | pub use syntax::{Language, MultiParser, ParsedFile}; |
| `ParsedFile` | libs/compass/src/lib.rs | re-export | pub | 55 | pub use syntax::{Language, MultiParser, ParsedFile}; |
| `FileWatcher` | libs/compass/src/lib.rs | re-export | pub | 56 | pub use watch::{FileWatcher, WatchConfig, WatchEvent}; |
| `WatchConfig` | libs/compass/src/lib.rs | re-export | pub | 56 | pub use watch::{FileWatcher, WatchConfig, WatchEvent}; |
| `WatchEvent` | libs/compass/src/lib.rs | re-export | pub | 56 | pub use watch::{FileWatcher, WatchConfig, WatchEvent}; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! # cclab-compass
//!
//! Code intelligence arsenal for the cclab ecosystem. Compass gives developers
//! and AI agents the ability to **navigate** a codebase — tree-sitter parsing,
//! type inference, semantic analysis, LSP integration, file watching,
//! refactoring, and lint infrastructure.
//!
//! ## Naming
//!
//! "Compass" = navigation. Code intelligence is about finding your way
//! through an unfamiliar codebase: jump to definition, find references,
//! impact analysis, dependency graph. The tool is the compass; the
//! codebase is the terrain.
//!
//! ## Consumers
//!
//! - `projects/agentic-workflow/` — local Rust CLI (direct dependency)
//! - `projects/conductor/` — cloud web
//! - `sdd` — library crate re-exports compass for backward compat

pub mod check_pipeline;
pub mod checker;
pub mod core;
// generate/ module moved to sdd crate (consolidate-codegen)
pub mod diagnostic;
pub mod format;
pub mod gen;
pub mod graph;
pub mod lens_error;
pub mod lint;
pub mod lsp;
pub mod outline;
pub mod output;
pub mod refactoring;
pub mod schemas;
pub mod search;
pub mod semantic;
pub mod server;
pub mod spec;
pub mod storage;
pub mod syntax;
pub mod type_inference;
pub mod watch;

// Re-export commonly used types (matches the surface previously exposed by sdd)
pub use checker::{check_paths, check_paths_with_propagation, FileResult, LintConfig};
pub use core::{ArgusConfig, LanguageConfig};
pub use diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
pub use gen::{CodeGenerator, GenContext, GenError, GenResult, GeneratedCode, TechStack};
pub use lens_error::ArgusError;
pub use lint::{Checker, CheckerRegistry};
pub use outline::{outline, outline_parsed, FunctionDef, FunctionKind};
pub use output::reporter::{OutputFormat, Reporter};
pub use server::{ArgusDaemon, DaemonClient, DaemonConfig, RequestHandler};
pub use syntax::{Language, MultiParser, ParsedFile};
pub use watch::{FileWatcher, WatchConfig, WatchEvent};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lib.rs` captured during libs codegen standardization.
```
