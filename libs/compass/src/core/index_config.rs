//! Index server configuration — scoped toolchain binding (#1127)
//!
//! Supports auto-discovery of project roots from marker files
//! (Cargo.toml, pyproject.toml, tsconfig.json) and per-scope
//! configuration of search paths, interpreters, and cache directories.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level index server configuration.
///
/// Deserialized from `[index]` section in `.aw/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexConfig {
    /// Auto-discover scopes from marker files (default: true)
    #[serde(default = "default_true")]
    pub auto_discover: bool,

    /// Explicitly configured scopes (merged with auto-discovered)
    #[serde(default, rename = "scope")]
    pub scopes: Vec<ScopeConfig>,
}

/// Per-scope configuration for a project within the monorepo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {
    /// Unique scope identifier (e.g., "rust-workspace", "py-conductor")
    pub id: String,

    /// Programming language
    pub lang: ScopeLang,

    /// Root directory relative to project root
    pub root: PathBuf,

    /// Python interpreter path (relative to project root)
    /// Auto-detected from .venv if not specified
    #[serde(default)]
    pub interpreter: Option<PathBuf>,

    /// Additional search paths for import resolution
    /// Auto-populated from toolchain if not specified
    #[serde(default)]
    pub search_paths: Vec<PathBuf>,

    /// Marker file that triggered auto-discovery (not user-configurable)
    #[serde(skip)]
    pub marker: Option<PathBuf>,
}

/// Supported scope languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScopeLang {
    Rust,
    Python,
    #[serde(alias = "ts")]
    Typescript,
    #[serde(alias = "js")]
    Javascript,
    Go,
}

impl std::fmt::Display for ScopeLang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeLang::Rust => write!(f, "rust"),
            ScopeLang::Python => write!(f, "python"),
            ScopeLang::Typescript => write!(f, "typescript"),
            ScopeLang::Javascript => write!(f, "javascript"),
            ScopeLang::Go => write!(f, "go"),
        }
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_index_config() {
        let toml_str = r#"
auto_discover = true

[[scope]]
id = "py-conductor"
lang = "python"
root = "projects/conductor"
interpreter = ".venv/bin/python"

[[scope]]
id = "rust-workspace"
lang = "rust"
root = "."
"#;
        let config: IndexConfig = toml::from_str(toml_str).unwrap();
        assert!(config.auto_discover);
        assert_eq!(config.scopes.len(), 2);
        assert_eq!(config.scopes[0].id, "py-conductor");
        assert_eq!(config.scopes[0].lang, ScopeLang::Python);
        assert_eq!(config.scopes[1].lang, ScopeLang::Rust);
    }

    #[test]
    fn test_default_config() {
        let config = IndexConfig::default();
        assert!(!config.auto_discover); // Default derives false, but default_true overrides in serde
        assert!(config.scopes.is_empty());
    }

    #[test]
    fn test_scope_lang_display() {
        assert_eq!(ScopeLang::Rust.to_string(), "rust");
        assert_eq!(ScopeLang::Python.to_string(), "python");
        assert_eq!(ScopeLang::Typescript.to_string(), "typescript");
    }
}
