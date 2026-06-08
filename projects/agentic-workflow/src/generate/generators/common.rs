//! Common generator types and traits

use crate::generate::engine::TemplateEngine;
use crate::generate::schema::JsonSchema;
use std::collections::BTreeMap;
use std::path::PathBuf;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/common.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Status of a generated file.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    Written,
    Skipped,
    Error,
}

/// A generated file entry.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub status: FileStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Generator error types.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common.md#schema
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("Template set missing at: {0}")]
    TemplateSetMissing(PathBuf),
    #[error("Template render error in '{template}': {message}")]
    TemplateRenderError { template: String, message: String },
    #[error("Overwrite not allowed for file: {0}")]
    OverwriteNotAllowed(PathBuf),
    #[error("IO error for '{path}': {message}")]
    IoError { path: PathBuf, message: String },
    #[error("Schema error: {0}")]
    SchemaError(String),
}

/// Generator settings.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorSettings {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub lang: String,
    #[serde(default)]
    pub output_dir: PathBuf,
    #[serde(default)]
    pub overwrite_policy: OverwritePolicy,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common.md#schema.trait-impls.Default
impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            name: "app".to_string(),
            version: "0.1.0".to_string(),
            lang: "".to_string(),
            output_dir: PathBuf::from("."),
            overwrite_policy: OverwritePolicy::default(),
        }
    }
}

/// Manifest of generated files (sorted by path for determinism).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub files: BTreeMap<PathBuf, GeneratedFile>,
}

/// Overwrite policy for generated files.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common.md#schema
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverwritePolicy {
    /// Error on overwrite (default).
    #[default]
    Error,
    /// Skip on overwrite.
    Skip,
    /// Always overwrite.
    Overwrite,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/common_helpers.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common_helpers.md#source
impl GeneratedFile {
    pub fn written(path: PathBuf, content: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());

        Self {
            path,
            status: FileStatus::Written,
            content_hash: Some(hash),
            error: None,
        }
    }

    pub fn skipped(path: PathBuf) -> Self {
        Self {
            path,
            status: FileStatus::Skipped,
            content_hash: None,
            error: None,
        }
    }

    pub fn error(path: PathBuf, error: impl Into<String>) -> Self {
        Self {
            path,
            status: FileStatus::Error,
            content_hash: None,
            error: Some(error.into()),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common_helpers.md#source
impl Manifest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, file: GeneratedFile) {
        self.files.insert(file.path.clone(), file);
    }

    pub fn written_count(&self) -> usize {
        self.files
            .values()
            .filter(|f| f.status == FileStatus::Written)
            .count()
    }

    pub fn skipped_count(&self) -> usize {
        self.files
            .values()
            .filter(|f| f.status == FileStatus::Skipped)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.files
            .values()
            .filter(|f| f.status == FileStatus::Error)
            .count()
    }
}

/// Trait for code generators backed by JSON Schema / OpenAPI input.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common_helpers.md#source
pub trait Generator {
    /// Generate code from a schema
    fn generate(
        &self,
        schema: &JsonSchema,
        settings: &GeneratorSettings,
        engine: &TemplateEngine,
    ) -> Result<Manifest, GeneratorError>;

    /// Get the template subdirectory name
    fn template_dir(&self) -> &'static str;
}

/// Trait for code generators that consume [`crate::generate::spec_ir::SpecIR`] directly.
///
/// Used by the new section-type generators (deploy, wireframe, component,
/// design-token) that receive structured spec payloads rather than raw JSON Schema.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/common_helpers.md#source
pub trait SpecIRGenerator {
    /// Return `true` if this generator can handle the given [`SpecIR`] variant.
    fn can_generate(&self, spec: &crate::generate::spec_ir::SpecIR) -> bool;

    /// Generate code from a [`SpecIR`] item.
    fn generate_from_ir(
        &self,
        spec: &crate::generate::spec_ir::SpecIR,
        settings: &GeneratorSettings,
        engine: &TemplateEngine,
    ) -> Result<Manifest, GeneratorError>;

    /// Get the template subdirectory name used for Tera template look-ups.
    fn template_dir(&self) -> &'static str;
}
// CODEGEN-END
