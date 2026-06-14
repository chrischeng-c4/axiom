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
