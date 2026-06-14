//! JavaScript symbol extraction
//!
//! JavaScript is a subset of TypeScript, so we delegate entirely
//! to the TypeScript symbol extractor. The `build_javascript` method
//! on `SymbolTableBuilder` calls `build_typescript` directly.
//!
//! This module exists to make the delegation explicit and allow
//! future JS-specific overrides if needed.
