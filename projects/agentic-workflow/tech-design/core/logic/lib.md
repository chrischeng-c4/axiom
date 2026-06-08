---
id: projects-sdd-src-lib-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core logic modules define AW Core defaults, exports, and workflow invariants."
---

# Standardized projects/agentic-workflow/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/lib.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `agents` | projects/agentic-workflow/src/lib.rs | module | pub | 12 |  |
| `branch_switch` | projects/agentic-workflow/src/lib.rs | module | pub | 13 |  |
| `cli` | projects/agentic-workflow/src/lib.rs | module | pub | 14 |  |
| `context` | projects/agentic-workflow/src/lib.rs | module | pub | 15 |  |
| `context_builder` | projects/agentic-workflow/src/lib.rs | module | pub | 16 |  |
| `defaults` | projects/agentic-workflow/src/lib.rs | module | pub | 17 |  |
| `fillback` | projects/agentic-workflow/src/lib.rs | module | pub | 18 |  |
| `generate` | projects/agentic-workflow/src/lib.rs | module | pub | 19 |  |
| `generators` | projects/agentic-workflow/src/lib.rs | module | pub | 20 |  |
| `git` | projects/agentic-workflow/src/lib.rs | module | pub | 21 |  |
| `issues` | projects/agentic-workflow/src/lib.rs | module | pub | 22 |  |
| `models` | projects/agentic-workflow/src/lib.rs | module | pub | 23 |  |
| `parser` | projects/agentic-workflow/src/lib.rs | module | pub | 24 |  |
| `runtime` | projects/agentic-workflow/src/lib.rs | module | pub | 25 |  |
| `services` | projects/agentic-workflow/src/lib.rs | module | pub | 26 |  |
| `shared` | projects/agentic-workflow/src/lib.rs | module | pub | 27 |  |
| `spec_alignment` | projects/agentic-workflow/src/lib.rs | module | pub | 28 |  |
| `spec_ir` | projects/agentic-workflow/src/lib.rs | module | pub | 29 |  |
| `spec_store` | projects/agentic-workflow/src/lib.rs | module | pub | 30 |  |
| `state` | projects/agentic-workflow/src/lib.rs | module | pub | 31 |  |
| `td_ast` | projects/agentic-workflow/src/lib.rs | module | pub | 32 |  |
| `test_util` | projects/agentic-workflow/src/lib.rs | module | pub | 40 |  |
| `tools` | projects/agentic-workflow/src/lib.rs | module | pub | 33 |  |
| `ui` | projects/agentic-workflow/src/lib.rs | module | pub | 34 |  |
| `validate` | projects/agentic-workflow/src/lib.rs | module | pub | 35 |  |
| `validator` | projects/agentic-workflow/src/lib.rs | module | pub | 36 |  |
| `workflow` | projects/agentic-workflow/src/lib.rs | module | pub | 37 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/lib.rs -->
```rust
// SDD - Spec-driven Development Orchestrator
// A Rust-powered tool for iterative proposal refinement through AI orchestration
//
// Code intelligence modules (tree-sitter parsing, type inference, LSP, semantic
// analysis, refactoring, file watching, Argus daemon) moved to `cclab-compass`
// in #1164. They are re-exported below for backward compatibility with existing
// callers (e.g. projects/agentic-workflow/).

// === SDD modules (business logic) ===
pub mod agents;
pub mod branch_switch;
pub mod context;
pub mod context_builder;
pub mod defaults;
pub mod fillback;
pub mod generate;
pub mod generators;
pub mod git;
pub mod issues;
pub mod models;
pub mod parser;
pub mod runtime;
pub mod services;
pub mod shared;
pub mod spec_alignment;
pub mod spec_ir;
pub mod spec_store;
pub mod state;
pub mod td_ast;
pub mod tools;
pub mod ui;
pub mod validate;
pub mod validator;
pub mod workflow;

#[cfg(test)]
pub(crate) mod test_util;

pub use anyhow::{Context, Result};
pub use colored::Colorize;

// generate/ is now directly owned by sdd (moved from cclab-compass)
pub use generate::{call_tool, is_sdd_tool, JsonSchema, SddTools, SpecIR};

// Re-export commonly used SDD types
pub use models::{Challenge, Change, Requirement, Scenario, Verification};
pub use state::{StalenessReport, StateManager};

// === Compass re-exports (code intelligence — see projects/compass) ===
//
// These re-exports preserve the public API that existed before #1164 extracted
// the lens cluster into cclab-compass. Downstream callers (score, cclab-cli)
// continue to import from `agentic_workflow::checker`, `agentic_workflow::diagnostic`, etc.
// For new code, prefer importing from `cclab_compass::*` directly.

pub use cclab_compass::check_pipeline;
pub use cclab_compass::checker::{
    self, check_paths, check_paths_with_propagation, FileResult, LintConfig,
};
pub use cclab_compass::core::{self, ArgusConfig, LanguageConfig};
pub use cclab_compass::diagnostic::{
    self, Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range,
};
pub use cclab_compass::format;
pub use cclab_compass::gen::{
    self, CodeGenerator, GenContext, GenError, GenResult, GeneratedCode, TechStack,
};
pub use cclab_compass::graph;
pub use cclab_compass::lens_error::{self, ArgusError};
pub use cclab_compass::lint::{self, Checker, CheckerRegistry};
pub use cclab_compass::lsp;
pub use cclab_compass::output::{
    self,
    reporter::{OutputFormat, Reporter},
};
pub use cclab_compass::refactoring;
pub use cclab_compass::schemas;
pub use cclab_compass::search;
pub use cclab_compass::semantic;
pub use cclab_compass::server::{self, ArgusDaemon, DaemonClient, DaemonConfig, RequestHandler};
pub use cclab_compass::spec;
pub use cclab_compass::storage;
pub use cclab_compass::syntax::{self, Language, MultiParser, ParsedFile};
pub use cclab_compass::type_inference;
pub use cclab_compass::watch::{self, FileWatcher, WatchConfig, WatchEvent};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/lib.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Codegen owns the crate facade through a source template.
```
