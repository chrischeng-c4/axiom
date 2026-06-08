//! Schema-based validation for Kubernetes manifests and GitLab CI configs.
//!
//! Builds JSON Schemas programmatically in Rust (no bundled JSON files)
//! and validates parsed YAML/JSON values against them using the `jsonschema` crate.

pub mod frontmatter;
mod gitlab;
mod k8s;

use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use jsonschema::Validator;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Schema registry — lazily compiles and caches JSON Schema validators.
pub struct SchemaRegistry {
    /// K8s validators keyed by (kind, version). Version is stored for future
    /// differentiation; currently all versions share the same schema.
    k8s_validators: HashMap<String, Validator>,
    /// Single GitLab CI validator.
    gitlab_ci_validator: Validator,
    /// The K8s version used (stored for diagnostics).
    _k8s_version: String,
}

/// Global singleton so callers don't have to thread the registry everywhere.
static GLOBAL_REGISTRY: OnceLock<SchemaRegistry> = OnceLock::new();

impl SchemaRegistry {
    /// Build a new registry with compiled validators for all K8s resource
    /// kinds and the GitLab CI schema.
    pub fn new(k8s_version: &str) -> Self {
        let mut k8s_validators = HashMap::new();

        for (kind, schema) in k8s::build_all_schemas() {
            match Validator::new(&schema) {
                Ok(v) => {
                    k8s_validators.insert(kind, v);
                }
                Err(e) => {
                    tracing::warn!("Failed to compile K8s schema for {}: {}", kind, e);
                }
            }
        }

        let gitlab_schema = gitlab::build_gitlab_ci_schema();
        let gitlab_ci_validator =
            Validator::new(&gitlab_schema).expect("GitLab CI schema must compile");

        Self {
            k8s_validators,
            gitlab_ci_validator,
            _k8s_version: k8s_version.to_string(),
        }
    }

    /// Get (or create) the global shared registry.
    pub fn global() -> &'static SchemaRegistry {
        GLOBAL_REGISTRY.get_or_init(|| SchemaRegistry::new("1.30"))
    }

    // -----------------------------------------------------------------
    // K8s validation
    // -----------------------------------------------------------------

    /// Validate a parsed K8s manifest value.
    ///
    /// The `kind` field is extracted from the value to select the right schema.
    /// Returns diagnostics with JSON-path locations.
    pub fn validate_k8s(&self, value: &Value, _version: &str) -> Vec<Diagnostic> {
        let kind = match value.get("kind").and_then(|v| v.as_str()) {
            Some(k) => k.to_string(),
            None => return Vec::new(), // no kind — handled by K8001
        };

        let validator = match self.k8s_validators.get(&kind) {
            Some(v) => v,
            None => return Vec::new(), // unknown kind — no schema
        };

        let mut diagnostics = Vec::new();
        for error in validator.iter_errors(value) {
            let path = error.instance_path.to_string();
            let msg = if path.is_empty() {
                format!("[{}] {}", kind, error)
            } else {
                format!("[{}] {}: {}", kind, path, error)
            };

            diagnostics.push(Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                DiagnosticSeverity::Warning,
                "K8002",
                DiagnosticCategory::Logic,
                msg,
            ));
        }
        diagnostics
    }

    // -----------------------------------------------------------------
    // GitLab CI validation
    // -----------------------------------------------------------------

    /// Validate a parsed GitLab CI config value (top-level object).
    pub fn validate_gitlab_ci(&self, value: &Value) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for error in self.gitlab_ci_validator.iter_errors(value) {
            let path = error.instance_path.to_string();
            let msg = if path.is_empty() {
                format!("GitLab CI schema: {}", error)
            } else {
                format!("GitLab CI schema {}: {}", path, error)
            };

            diagnostics.push(Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                DiagnosticSeverity::Warning,
                "GL002",
                DiagnosticCategory::Logic,
                msg,
            ));
        }
        diagnostics
    }
}
