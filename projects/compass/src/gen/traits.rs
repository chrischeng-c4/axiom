//! Code generator traits and configuration
//!
//! Defines the `CodeGenerator` trait that all generators implement,
//! along with `TechStack` enum and `GenContext` configuration.

use std::collections::HashMap;
use std::fmt;

use crate::spec::ir::{
    ControlFlowSpec, DataModelSpec, EventApiSpec, RestApiSpec, StateMachineSpec,
};

/// Result type for code generation
pub type GenResult<T> = Result<T, GenError>;

/// Code generation error
#[derive(Debug)]
pub enum GenError {
    /// Unsupported feature for the target stack
    UnsupportedFeature(String),
    /// Invalid input specification
    InvalidSpec(String),
    /// Type conversion error
    TypeConversion(String),
    /// IO error
    Io(std::io::Error),
    /// Other error
    Other(String),
}

impl fmt::Display for GenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenError::UnsupportedFeature(s) => write!(f, "unsupported feature: {}", s),
            GenError::InvalidSpec(s) => write!(f, "invalid spec: {}", s),
            GenError::TypeConversion(s) => write!(f, "type conversion error: {}", s),
            GenError::Io(e) => write!(f, "IO error: {}", e),
            GenError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for GenError {}

impl From<std::io::Error> for GenError {
    fn from(e: std::io::Error) -> Self {
        GenError::Io(e)
    }
}

/// Target technology stack for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TechStack {
    // Python stacks (cclab ecosystem)
    /// cclab.shield - Data validation models
    Shield,
    /// cclab.titan - PostgreSQL ORM
    Titan,
    /// cclab.nebula - MongoDB ORM
    Nebula,
    /// cclab.photon - HTTP client
    Photon,
    /// cclab.quasar - API route handlers
    Quasar,
    /// cclab.meteor - Task queue handlers
    Swarm,

    // Rust stacks
    /// serde - Rust structs with serialization
    Serde,
    /// Axum - Route handlers
    Axum,
    /// sqlx - Database models with query macros
    Sqlx,
    /// reqwest - HTTP client
    Reqwest,

    // Framework generators (from SDD generate module)
    /// FastAPI - Python API framework
    FastAPI,
    /// Express - TypeScript/Node.js API framework
    Express,
    /// AxumFramework - Rust API framework (full project generation)
    AxumFramework,

    // TypeScript stacks (Phase 2+)
    /// TypeScript interfaces
    TypeScriptInterface,
    /// Zod validation schemas
    Zod,
}

impl TechStack {
    /// Get the target language for this stack
    pub fn language(&self) -> Language {
        match self {
            TechStack::Shield
            | TechStack::Titan
            | TechStack::Nebula
            | TechStack::Photon
            | TechStack::Quasar
            | TechStack::Swarm => Language::Python,

            TechStack::Serde
            | TechStack::Axum
            | TechStack::Sqlx
            | TechStack::Reqwest
            | TechStack::AxumFramework => Language::Rust,

            TechStack::TypeScriptInterface | TechStack::Zod | TechStack::Express => {
                Language::TypeScript
            }

            TechStack::FastAPI => Language::Python,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            TechStack::Shield => "cclab.shield",
            TechStack::Titan => "cclab.titan",
            TechStack::Nebula => "cclab.nebula",
            TechStack::Photon => "cclab.photon",
            TechStack::Quasar => "cclab.quasar",
            TechStack::Swarm => "cclab.meteor",
            TechStack::Serde => "serde",
            TechStack::Axum => "axum",
            TechStack::Sqlx => "sqlx",
            TechStack::Reqwest => "reqwest",
            TechStack::TypeScriptInterface => "typescript",
            TechStack::Zod => "zod",
            TechStack::FastAPI => "fastapi",
            TechStack::Express => "express",
            TechStack::AxumFramework => "axum-framework",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "shield" | "cclab.shield" => Some(TechStack::Shield),
            "titan" | "cclab.titan" => Some(TechStack::Titan),
            "nebula" | "cclab.nebula" => Some(TechStack::Nebula),
            "photon" | "cclab.photon" => Some(TechStack::Photon),
            "quasar" | "cclab.quasar" => Some(TechStack::Quasar),
            "meteor" | "cclab.meteor" => Some(TechStack::Swarm),
            "serde" => Some(TechStack::Serde),
            "axum" => Some(TechStack::Axum),
            "sqlx" => Some(TechStack::Sqlx),
            "reqwest" => Some(TechStack::Reqwest),
            "typescript" | "ts" => Some(TechStack::TypeScriptInterface),
            "zod" => Some(TechStack::Zod),
            "fastapi" | "fast-api" => Some(TechStack::FastAPI),
            "express" => Some(TechStack::Express),
            "axum-framework" | "axum_framework" => Some(TechStack::AxumFramework),
            _ => None,
        }
    }
}

impl fmt::Display for TechStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Target programming language
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Python,
    Rust,
    TypeScript,
}

impl Language {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::Python => "py",
            Language::Rust => "rs",
            Language::TypeScript => "ts",
        }
    }
}

/// Code generation context/configuration
#[derive(Debug, Clone)]
pub struct GenContext {
    /// Target technology stack
    pub stack: TechStack,
    /// Output module/package name
    pub module_name: Option<String>,
    /// Whether to generate documentation comments
    pub generate_docs: bool,
    /// Whether to generate validation code
    pub generate_validation: bool,
    /// Custom type mappings (spec type -> target type)
    pub type_mappings: HashMap<String, String>,
    /// Import prefix for referenced types
    pub import_prefix: Option<String>,
    /// Indentation style
    pub indent: String,
    /// Whether to use strict null checks (TypeScript)
    pub strict_nulls: bool,
    /// Database schema name (for ORM)
    pub db_schema: Option<String>,
}

impl Default for GenContext {
    fn default() -> Self {
        Self {
            stack: TechStack::Shield,
            module_name: None,
            generate_docs: true,
            generate_validation: true,
            type_mappings: HashMap::new(),
            import_prefix: None,
            indent: "    ".to_string(), // 4 spaces
            strict_nulls: true,
            db_schema: None,
        }
    }
}

impl GenContext {
    pub fn new(stack: TechStack) -> Self {
        Self {
            stack,
            ..Default::default()
        }
    }

    pub fn with_module(mut self, name: impl Into<String>) -> Self {
        self.module_name = Some(name.into());
        self
    }

    pub fn with_docs(mut self, generate: bool) -> Self {
        self.generate_docs = generate;
        self
    }

    pub fn with_validation(mut self, generate: bool) -> Self {
        self.generate_validation = generate;
        self
    }

    pub fn with_type_mapping(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.type_mappings.insert(from.into(), to.into());
        self
    }
}

/// Generated code output
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    /// File name (without extension)
    pub name: String,
    /// Generated code content
    pub content: String,
    /// Imports/dependencies needed
    pub imports: Vec<String>,
    /// Language
    pub language: Language,
}

impl GeneratedCode {
    pub fn new(name: impl Into<String>, content: impl Into<String>, language: Language) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            imports: Vec::new(),
            language,
        }
    }

    pub fn with_imports(mut self, imports: Vec<String>) -> Self {
        self.imports = imports;
        self
    }

    /// Get full file content with imports
    pub fn full_content(&self) -> String {
        if self.imports.is_empty() {
            self.content.clone()
        } else {
            let imports_section = self.imports.join("\n");
            format!("{}\n\n{}", imports_section, self.content)
        }
    }

    /// Get filename with extension
    pub fn filename(&self) -> String {
        format!("{}.{}", self.name, self.language.file_extension())
    }
}

/// Trait for code generators
///
/// Each generator is responsible for converting SpecIR to code for a specific
/// technology stack. Supports both the legacy per-type methods and the new
/// unified SpecIR-based `generate` / `can_generate` interface.
///
/// The SpecIR-aware methods accept `serde_json::Value` to avoid a circular
/// dependency between cclab-compass and sdd (the generate module now lives
/// in sdd). Typed SpecIR dispatch is provided by `sdd::generate::generators`.
pub trait CodeGenerator {
    /// Generator name for display and routing.
    fn name(&self) -> &str;

    /// Check whether this generator can handle the given SpecIR (as JSON).
    fn can_generate(&self, _spec: &serde_json::Value) -> bool {
        false
    }

    /// Generate code from a unified SpecIR input (preferred path).
    ///
    /// Default implementation returns an error. Override for generators that
    /// consume SpecIR directly. The value is a serialized SpecIR enum.
    fn generate(
        &self,
        _spec: &serde_json::Value,
        _ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        Err(GenError::UnsupportedFeature(
            "SpecIR-based generation not yet implemented for this generator".into(),
        ))
    }

    // --- Legacy per-type methods (kept for existing per-crate generators) ---

    /// Generate code from data model spec
    fn generate_data_models(
        &self,
        spec: &DataModelSpec,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>>;

    /// Generate code from REST API spec (optional, not all generators support this)
    fn generate_rest_api(
        &self,
        _spec: &RestApiSpec,
        _ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        Err(GenError::UnsupportedFeature(
            "REST API generation not supported".into(),
        ))
    }

    /// Generate code from event API spec (optional)
    fn generate_event_api(
        &self,
        _spec: &EventApiSpec,
        _ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        Err(GenError::UnsupportedFeature(
            "Event API generation not supported".into(),
        ))
    }

    /// Generate code from state machine spec (optional)
    fn generate_state_machine(
        &self,
        _spec: &StateMachineSpec,
        _ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        Err(GenError::UnsupportedFeature(
            "State machine generation not supported".into(),
        ))
    }

    /// Generate code from control flow spec (optional)
    fn generate_control_flow(
        &self,
        _spec: &ControlFlowSpec,
        _ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        Err(GenError::UnsupportedFeature(
            "Control flow generation not supported".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tech_stack_language() {
        assert_eq!(TechStack::Shield.language(), Language::Python);
        assert_eq!(TechStack::Titan.language(), Language::Python);
        assert_eq!(TechStack::Serde.language(), Language::Rust);
        assert_eq!(TechStack::Axum.language(), Language::Rust);
    }

    #[test]
    fn test_tech_stack_from_str() {
        assert_eq!(TechStack::from_str("shield"), Some(TechStack::Shield));
        assert_eq!(TechStack::from_str("cclab.titan"), Some(TechStack::Titan));
        assert_eq!(TechStack::from_str("serde"), Some(TechStack::Serde));
        assert_eq!(TechStack::from_str("invalid"), None);
    }

    #[test]
    fn test_gen_context_builder() {
        let ctx = GenContext::new(TechStack::Shield)
            .with_module("models")
            .with_docs(true)
            .with_type_mapping("CustomId", "str");

        assert_eq!(ctx.stack, TechStack::Shield);
        assert_eq!(ctx.module_name, Some("models".to_string()));
        assert!(ctx.generate_docs);
        assert_eq!(ctx.type_mappings.get("CustomId"), Some(&"str".to_string()));
    }

    #[test]
    fn test_generated_code() {
        let code = GeneratedCode::new("user", "class User: pass", Language::Python)
            .with_imports(vec!["from cclab.shield import BaseModel".to_string()]);

        assert_eq!(code.filename(), "user.py");
        assert!(code.full_content().contains("from cclab.shield"));
    }
}
