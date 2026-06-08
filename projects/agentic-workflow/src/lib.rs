// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/lib.md#source
// CODEGEN-BEGIN
// Agentic Workflow - spec-governed workflow engine and CLI
// A Rust-powered tool for iterative proposal refinement through AI orchestration
//
// Code intelligence modules (tree-sitter parsing, type inference, LSP, semantic
// analysis, refactoring, file watching, Argus daemon) moved to `cclab-compass`
// in #1164. They are re-exported below for backward compatibility with existing
// callers (e.g. projects/agentic-workflow/).

// === Agentic Workflow core modules ===
pub mod agents;
pub mod branch_switch;
pub mod cli;
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
pub use cli::find_project_root;
pub use colored::Colorize;

// generate/ is now directly owned by Agentic Workflow (moved from cclab-compass)
pub use generate::{call_tool, is_sdd_tool, JsonSchema, SddTools, SpecIR};

// Re-export commonly used workflow types
pub use models::{Challenge, Change, Requirement, Scenario, Verification};
pub use state::{StalenessReport, StateManager};

// === Compass re-exports (code intelligence — see projects/compass) ===
//
// These re-exports preserve the public API that existed before #1164 extracted
// the lens cluster into cclab-compass.
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

// CODEGEN-END
