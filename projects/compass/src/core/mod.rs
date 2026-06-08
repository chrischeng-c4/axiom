//! Core infrastructure for Argus
//!
//! Provides configuration, workspace management, and shared utilities.

mod config;
pub mod index_config;

pub use config::{
    ArgusConfig, ArgusSettings, LanguageConfig, LintConfig, PythonConfig, RustConfig,
    TypeScriptConfig,
};
