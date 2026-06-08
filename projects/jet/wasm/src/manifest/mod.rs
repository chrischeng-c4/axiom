// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
// CODEGEN-BEGIN
//! React-compat binding manifest — `jet.declare.d.ts` parsing + defaults.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/binding-manifest.md
//!
//! Public surface:
//!   - [`ParsedManifest`] — top-level shape returned by [`parse_manifest`].
//!   - [`ModuleEntry`], [`ExportEntry`], [`JetImpl`] — manifest data types.
//!   - [`ManifestError`] — error variants with stable codes
//!     `MANIFEST_PARSE_001`..`MANIFEST_PARSE_005`.
//!   - [`parse_manifest`] — load + parse + overlay-merge across the
//!     nearest-ancestor chain to the workspace root.
//!   - [`DEFAULT_BINDINGS`] — v0 starter set (fetch / console /
//!     localStorage / JSON). Comprehensive WinterCG-aligned default set
//!     is a named follow-up issue per R11.
//!
//! The transpiler (`jet-tsx-to-rust`, when it lands as a Phase 1
//! deliverable) imports the public surface here and consumes
//! `ParsedManifest` at import-resolution time. This crate only owns
//! the parsing + merge contract; transpiler emit semantics live in
//! `transpiler.md`.

pub mod defaults;
pub mod parser;

pub use defaults::DEFAULT_BINDINGS;
pub use parser::{
    parse_manifest, parse_manifest_text, ExportEntry, ExportKind, JetImpl, ManifestError,
    ManifestErrorCode, ModuleEntry, ParsedManifest,
};
// CODEGEN-END
