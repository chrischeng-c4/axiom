// SPEC-MANAGED: libs/compass/tech-design/semantic/source/libs-compass-src-core-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Core infrastructure for Argus
//!
//! Provides configuration, workspace management, and shared utilities.

mod config;
pub mod index_config;

pub use config::{
    ArgusConfig, ArgusSettings, LanguageConfig, LintConfig, PythonConfig, RustConfig,
    TypeScriptConfig,
};
// CODEGEN-END
