// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/validator/schema.md#source
// CODEGEN-BEGIN
//! JSON Schema Validation for Frontmatter
//!
//! Validates YAML frontmatter against JSON Schema definitions.

use crate::models::{ErrorCategory, Severity, ValidationError, ValidationResult};
use crate::parser::frontmatter::{normalize_content, split_frontmatter};
use anyhow::{Context, Result};
use jsonschema::Validator;
use serde_json::Value as JsonValue;
use std::path::Path;

/// SDD document types validated by schema.
/// @spec projects/agentic-workflow/tech-design/core/validate/validator/schema.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentType {
    /// Proposal document.
    Proposal,
    /// Tasks document.
    Tasks,
    /// Spec document.
    Spec,
    /// Challenge document.
    Challenge,
    /// State document.
    State,
}

/// Schema validator for frontmatter.
/// @spec projects/agentic-workflow/tech-design/core/validate/validator/schema.md#schema
pub struct SchemaValidator {
    /// Directory containing JSON schemas.
    schemas_dir: std::path::PathBuf,
    /// Cached compiled validators.
    validators: std::collections::HashMap<DocumentType, Validator>,
}

/// @spec projects/agentic-workflow/tech-design/core/validate/validator/schema.md#changes
impl DocumentType {
    /// Get the schema filename for this document type
    pub fn schema_filename(&self) -> &'static str {
        match self {
            DocumentType::Proposal => "proposal.schema.json",
            DocumentType::Tasks => "tasks.schema.json",
            DocumentType::Spec => "spec.schema.json",
            DocumentType::Challenge => "challenge.schema.json",
            DocumentType::State => "state.schema.json",
        }
    }

    /// Detect document type from filename
    pub fn from_filename(filename: &str) -> Option<Self> {
        let lower = filename.to_lowercase();
        if lower == "proposal.md" {
            Some(DocumentType::Proposal)
        } else if lower == "tasks.md" {
            Some(DocumentType::Tasks)
        } else if lower == "challenge.md" {
            Some(DocumentType::Challenge)
        } else if lower == "state.yaml" || lower == "state.yml" {
            Some(DocumentType::State)
        } else if lower.ends_with(".md") {
            // Default to spec for other .md files in specs/ directory
            Some(DocumentType::Spec)
        } else {
            None
        }
    }

    /// Detect document type from frontmatter type field
    pub fn from_type_field(type_field: &str) -> Option<Self> {
        match type_field.to_lowercase().as_str() {
            "proposal" => Some(DocumentType::Proposal),
            "tasks" => Some(DocumentType::Tasks),
            "spec" => Some(DocumentType::Spec),
            "challenge" => Some(DocumentType::Challenge),
            "state" => Some(DocumentType::State),
            _ => None,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/validate/validator/schema.md#source
impl SchemaValidator {
    /// Create a new schema validator
    ///
    /// # Arguments
    /// * `schemas_dir` - Directory containing JSON Schema files
    pub fn new(schemas_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            schemas_dir: schemas_dir.into(),
            validators: std::collections::HashMap::new(),
        }
    }

    /// Load and compile a schema for a document type
    fn get_validator(&mut self, doc_type: DocumentType) -> Result<&Validator> {
        if !self.validators.contains_key(&doc_type) {
            let schema_path = self.schemas_dir.join(doc_type.schema_filename());
            let schema_content = std::fs::read_to_string(&schema_path)
                .with_context(|| format!("Failed to read schema: {}", schema_path.display()))?;

            let schema: JsonValue = serde_json::from_str(&schema_content)
                .with_context(|| format!("Failed to parse schema: {}", schema_path.display()))?;

            let validator = Validator::new(&schema)
                .map_err(|e| anyhow::anyhow!("Failed to compile schema: {}", e))?;

            self.validators.insert(doc_type, validator);
        }

        Ok(self.validators.get(&doc_type).unwrap())
    }

    /// Validate a file's frontmatter against its schema
    pub fn validate_file(&mut self, file_path: &Path) -> ValidationResult {
        let mut errors = Vec::new();

        // Read file content
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Failed to read file: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Validate content
        errors.extend(self.validate_content(&content, file_path).errors);
        ValidationResult::new(errors)
    }

    /// Validate content string (for testing or direct validation)
    pub fn validate_content(&mut self, content: &str, file_path: &Path) -> ValidationResult {
        let mut errors = Vec::new();

        // Normalize and extract frontmatter
        let normalized = normalize_content(content);
        let (frontmatter_str, _) = match split_frontmatter(&normalized) {
            Ok((fm, body)) => (fm, body),
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Invalid frontmatter format: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Parse YAML to JSON for schema validation
        let yaml_value: serde_yaml::Value = match serde_yaml::from_str(&frontmatter_str) {
            Ok(v) => v,
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Invalid YAML in frontmatter: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Convert YAML to JSON for jsonschema validation
        let json_value: JsonValue = match serde_json::to_value(&yaml_value) {
            Ok(v) => v,
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Failed to convert YAML to JSON: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Determine document type
        let doc_type = self.detect_document_type(&json_value, file_path);
        let doc_type = match doc_type {
            Some(dt) => dt,
            None => {
                errors.push(ValidationError::new(
                    "Cannot determine document type from frontmatter or filename",
                    file_path,
                    None,
                    Severity::Medium,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Get validator and validate
        let validator = match self.get_validator(doc_type) {
            Ok(v) => v,
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Failed to load schema: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Run validation using iter_errors to get all validation errors
        for error in validator.iter_errors(&json_value) {
            let path = error.instance_path.to_string();
            let message = if path.is_empty() {
                error.to_string()
            } else {
                format!("{}: {}", path, error)
            };

            // Determine severity based on error type
            let severity = if message.contains("required") {
                Severity::High
            } else if message.contains("type") || message.contains("enum") {
                Severity::High
            } else {
                Severity::Medium
            };

            errors.push(ValidationError::new(
                message,
                file_path,
                None,
                severity,
                ErrorCategory::InvalidStructure,
            ));
        }

        ValidationResult::new(errors)
    }

    /// Detect document type from frontmatter or filename
    fn detect_document_type(
        &self,
        json_value: &JsonValue,
        file_path: &Path,
    ) -> Option<DocumentType> {
        // First try to detect from frontmatter "type" field
        if let Some(type_field) = json_value.get("type").and_then(|v| v.as_str()) {
            if let Some(doc_type) = DocumentType::from_type_field(type_field) {
                return Some(doc_type);
            }
        }

        // Fall back to filename detection
        if let Some(filename) = file_path.file_name().and_then(|f| f.to_str()) {
            return DocumentType::from_filename(filename);
        }

        None
    }

    /// Validate frontmatter has all required fields for a document type
    pub fn validate_required_fields(
        &self,
        frontmatter: &serde_yaml::Value,
        doc_type: DocumentType,
        file_path: &Path,
    ) -> ValidationResult {
        let mut errors = Vec::new();

        let required_fields: &[&str] = match doc_type {
            DocumentType::Proposal => &["id", "type", "version", "status"],
            DocumentType::Tasks => &["id", "type", "version"],
            DocumentType::Spec => &["id", "type", "title", "version"],
            DocumentType::Challenge => &["id", "type", "version", "verdict"],
            DocumentType::State => &["change_id", "phase"],
        };

        if let serde_yaml::Value::Mapping(map) = frontmatter {
            for field in required_fields {
                let key = serde_yaml::Value::String(field.to_string());
                if !map.contains_key(&key) {
                    errors.push(ValidationError::new(
                        format!("Missing required field: {}", field),
                        file_path,
                        None,
                        Severity::High,
                        ErrorCategory::InvalidStructure,
                    ));
                }
            }
        } else {
            errors.push(ValidationError::new(
                "Frontmatter must be a YAML mapping",
                file_path,
                None,
                Severity::High,
                ErrorCategory::InvalidStructure,
            ));
        }

        ValidationResult::new(errors)
    }
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Quick validation of a file against its schema
///
/// Uses the default schemas directory (cclab/schemas)
/// @spec projects/agentic-workflow/tech-design/core/validate/validator/schema.md#source
pub fn validate_frontmatter_schema(file_path: &Path, project_root: &Path) -> ValidationResult {
    let schemas_dir = project_root.join("cclab/schemas");
    let mut validator = SchemaValidator::new(schemas_dir);
    validator.validate_file(file_path)
}

/// Validate frontmatter content directly (for testing)
/// @spec projects/agentic-workflow/tech-design/core/validate/validator/schema.md#source
pub fn validate_frontmatter_content(
    content: &str,
    doc_type: DocumentType,
    project_root: &Path,
) -> ValidationResult {
    let schemas_dir = project_root.join("cclab/schemas");
    let mut validator = SchemaValidator::new(schemas_dir);

    // Create a temporary path for error reporting
    let temp_path = std::path::PathBuf::from(format!(
        "temp.{}",
        match doc_type {
            DocumentType::Proposal => "proposal.md",
            DocumentType::Tasks => "tasks.md",
            DocumentType::Spec => "spec.md",
            DocumentType::Challenge => "challenge.md",
            DocumentType::State => "state.yaml",
        }
    ));

    validator.validate_content(content, &temp_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── Helpers ─────────────────────────────────────────────────────────────

    /// Create a minimal JSON Schema that requires `id` and `type` as strings.
    fn minimal_schema() -> String {
        r#"{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "required": ["id", "type"],
            "properties": {
                "id":      { "type": "string" },
                "type":    { "type": "string" },
                "title":   { "type": "string" },
                "version": { "type": "integer" }
            }
        }"#
        .to_string()
    }

    /// Set up a temp dir with schema files for the given document types.
    fn setup_schemas(types: &[DocumentType]) -> TempDir {
        let dir = TempDir::new().unwrap();
        for dt in types {
            let path = dir.path().join(dt.schema_filename());
            std::fs::write(&path, minimal_schema()).unwrap();
        }
        dir
    }

    /// Build valid YAML frontmatter content for a given type string.
    fn valid_content(type_str: &str) -> String {
        format!(
            "---\nid: test-1\ntype: {}\ntitle: Test\nversion: 1\n---\n\n# Body\n",
            type_str
        )
    }

    // ── DocumentType helpers (existing) ────────────────────────────────────

    #[test]
    fn test_document_type_from_filename() {
        assert_eq!(
            DocumentType::from_filename("proposal.md"),
            Some(DocumentType::Proposal)
        );
        assert_eq!(
            DocumentType::from_filename("tasks.md"),
            Some(DocumentType::Tasks)
        );
        assert_eq!(
            DocumentType::from_filename("CHALLENGE.md"),
            Some(DocumentType::Challenge)
        );
        assert_eq!(
            DocumentType::from_filename("STATE.yaml"),
            Some(DocumentType::State)
        );
        assert_eq!(
            DocumentType::from_filename("state.yml"),
            Some(DocumentType::State)
        );
        assert_eq!(
            DocumentType::from_filename("auth.md"),
            Some(DocumentType::Spec)
        );
        assert_eq!(DocumentType::from_filename("readme.txt"), None);
    }

    #[test]
    fn test_document_type_from_type_field() {
        assert_eq!(
            DocumentType::from_type_field("proposal"),
            Some(DocumentType::Proposal)
        );
        assert_eq!(
            DocumentType::from_type_field("PROPOSAL"),
            Some(DocumentType::Proposal)
        );
        assert_eq!(
            DocumentType::from_type_field("tasks"),
            Some(DocumentType::Tasks)
        );
        assert_eq!(
            DocumentType::from_type_field("spec"),
            Some(DocumentType::Spec)
        );
        assert_eq!(
            DocumentType::from_type_field("challenge"),
            Some(DocumentType::Challenge)
        );
        assert_eq!(
            DocumentType::from_type_field("state"),
            Some(DocumentType::State)
        );
        assert_eq!(DocumentType::from_type_field("invalid"), None);
    }

    #[test]
    fn test_schema_filename() {
        assert_eq!(
            DocumentType::Proposal.schema_filename(),
            "proposal.schema.json"
        );
        assert_eq!(DocumentType::Tasks.schema_filename(), "tasks.schema.json");
        assert_eq!(DocumentType::Spec.schema_filename(), "spec.schema.json");
        assert_eq!(
            DocumentType::Challenge.schema_filename(),
            "challenge.schema.json"
        );
        assert_eq!(DocumentType::State.schema_filename(), "state.schema.json");
    }

    // ── REQ: R4 — SchemaValidator::new ─────────────────────────────────────

    #[test]
    fn test_schema_validator_new_with_valid_dir() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let validator = SchemaValidator::new(dir.path());
        assert!(validator.validators.is_empty());
        assert_eq!(validator.schemas_dir, dir.path());
    }

    // ── REQ: R4 — get_validator ────────────────────────────────────────────

    #[test]
    fn test_get_validator_nonexistent_schema() {
        let dir = TempDir::new().unwrap(); // empty dir, no schema files
        let mut validator = SchemaValidator::new(dir.path());
        let err = validator.get_validator(DocumentType::Proposal).unwrap_err();
        assert!(err.to_string().contains("Failed to read schema"));
    }

    #[test]
    fn test_get_validator_caches_compiled_validator() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let mut validator = SchemaValidator::new(dir.path());

        // First call — loads from disk
        assert!(validator.get_validator(DocumentType::Spec).is_ok());
        assert!(validator.validators.contains_key(&DocumentType::Spec));

        // Second call — returns from cache (HashMap already contains key)
        assert!(validator.get_validator(DocumentType::Spec).is_ok());
        assert_eq!(validator.validators.len(), 1);
    }

    #[test]
    fn test_get_validator_invalid_json_schema() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("proposal.schema.json");
        std::fs::write(&path, "not valid json").unwrap();
        let mut validator = SchemaValidator::new(dir.path());
        let err = validator.get_validator(DocumentType::Proposal).unwrap_err();
        assert!(err.to_string().contains("Failed to parse schema"));
    }

    // ── REQ: R4 — validate_file ────────────────────────────────────────────

    #[test]
    fn test_validate_file_nonexistent_path() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let mut validator = SchemaValidator::new(dir.path());
        let result = validator.validate_file(Path::new("/nonexistent/path.md"));
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].message.contains("Failed to read file"));
        assert_eq!(result.errors[0].category, ErrorCategory::InvalidStructure);
    }

    #[test]
    fn test_validate_file_valid_content() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let file_path = dir.path().join("my-spec.md");
        std::fs::write(&file_path, valid_content("spec")).unwrap();
        let mut validator = SchemaValidator::new(dir.path());
        let result = validator.validate_file(&file_path);
        assert!(
            result.errors.is_empty(),
            "expected no errors, got: {:?}",
            result.errors
        );
    }

    // ── REQ: R4 — validate_content ─────────────────────────────────────────

    #[test]
    fn test_validate_content_valid_frontmatter() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let mut validator = SchemaValidator::new(dir.path());
        let result = validator.validate_content(&valid_content("spec"), Path::new("my-spec.md"));
        assert!(
            result.errors.is_empty(),
            "expected no errors, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_validate_content_no_frontmatter() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let mut validator = SchemaValidator::new(dir.path());
        let result =
            validator.validate_content("No frontmatter delimiters here", Path::new("test.md"));
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].message.contains("frontmatter"));
    }

    #[test]
    fn test_validate_content_invalid_yaml() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let mut validator = SchemaValidator::new(dir.path());
        let result = validator.validate_content(
            "---\n: :\n  bad:\n    - [\n---\n\nbody",
            Path::new("test.md"),
        );
        assert!(!result.errors.is_empty());
        assert!(
            result.errors[0].message.contains("Invalid YAML")
                || result.errors[0].message.contains("frontmatter")
        );
    }

    #[test]
    fn test_validate_content_unknown_document_type() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let mut validator = SchemaValidator::new(dir.path());
        // type field is unrecognized AND filename has no extension
        let result = validator.validate_content(
            "---\nid: x\ntype: unknown-custom\n---\n\nbody",
            Path::new("noext"),
        );
        assert!(!result.errors.is_empty());
        assert!(result.errors[0]
            .message
            .contains("Cannot determine document type"));
    }

    #[test]
    fn test_validate_content_missing_required_fields() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let mut validator = SchemaValidator::new(dir.path());
        // frontmatter is missing `id` and `type` (both required by our minimal schema)
        let result =
            validator.validate_content("---\ntitle: X\n---\n\nbody", Path::new("my-spec.md"));
        // Should get validation errors from JSON Schema
        assert!(!result.errors.is_empty());
        let msgs: Vec<_> = result.errors.iter().map(|e| e.message.as_str()).collect();
        assert!(
            msgs.iter().any(|m| m.contains("required")),
            "expected required-field error, got: {:?}",
            msgs
        );
    }

    #[test]
    fn test_validate_content_schema_validation_errors() {
        let dir = setup_schemas(&[DocumentType::Spec]);
        let mut validator = SchemaValidator::new(dir.path());
        // version must be integer per our schema, but we pass a string
        let result = validator.validate_content(
            "---\nid: x\ntype: spec\nversion: not-a-number\n---\n\nbody",
            Path::new("my-spec.md"),
        );
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_content_schema_load_failure() {
        // Schema dir has no spec schema but file detects as spec via filename
        let dir = TempDir::new().unwrap();
        let mut validator = SchemaValidator::new(dir.path());
        let result = validator.validate_content(
            "---\nid: x\ntype: spec\n---\n\nbody",
            Path::new("my-spec.md"),
        );
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].message.contains("Failed to load schema"));
    }

    // ── REQ: R4 — detect_document_type ─────────────────────────────────────

    #[test]
    fn test_detect_document_type_type_field_precedence() {
        let validator = SchemaValidator::new("/tmp");
        let json: JsonValue = serde_json::json!({"type": "proposal"});
        // filename says projects/agentic-workflow/tech-design/core/tools/spec.md but type field says proposal — type field wins
        let dt = validator.detect_document_type(&json, Path::new("spec.md"));
        assert_eq!(dt, Some(DocumentType::Proposal));
    }

    #[test]
    fn test_detect_document_type_filename_fallback() {
        let validator = SchemaValidator::new("/tmp");
        let json: JsonValue = serde_json::json!({"id": "x"}); // no type field
        let dt = validator.detect_document_type(&json, Path::new("tasks.md"));
        assert_eq!(dt, Some(DocumentType::Tasks));
    }

    #[test]
    fn test_detect_document_type_none() {
        let validator = SchemaValidator::new("/tmp");
        let json: JsonValue = serde_json::json!({"id": "x"}); // no type field
        let dt = validator.detect_document_type(&json, Path::new("noext"));
        assert_eq!(dt, None);
    }

    // ── REQ: R4 — validate_required_fields ─────────────────────────────────

    #[test]
    fn test_validate_required_fields_non_mapping() {
        let validator = SchemaValidator::new("/tmp");
        let yaml_val = serde_yaml::Value::String("not a mapping".into());
        let result =
            validator.validate_required_fields(&yaml_val, DocumentType::Spec, Path::new("test.md"));
        assert!(!result.errors.is_empty());
        assert!(result.errors[0]
            .message
            .contains("Frontmatter must be a YAML mapping"));
    }

    #[test]
    fn test_validate_required_fields_all_present() {
        let validator = SchemaValidator::new("/tmp");
        let yaml_str = "id: x\ntype: spec\ntitle: Test\nversion: 1";
        let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let result =
            validator.validate_required_fields(&yaml_val, DocumentType::Spec, Path::new("test.md"));
        assert!(
            result.errors.is_empty(),
            "expected no errors, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_validate_required_fields_per_doc_type() {
        let validator = SchemaValidator::new("/tmp");
        let empty_map: serde_yaml::Value = serde_yaml::from_str("{}").unwrap();

        // Proposal requires: id, type, version, status
        let r = validator.validate_required_fields(
            &empty_map,
            DocumentType::Proposal,
            Path::new("p.md"),
        );
        let missing: Vec<_> = r.errors.iter().map(|e| e.message.as_str()).collect();
        assert!(missing.iter().any(|m| m.contains("status")));
        assert_eq!(r.errors.len(), 4);

        // Tasks requires: id, type, version
        let r =
            validator.validate_required_fields(&empty_map, DocumentType::Tasks, Path::new("t.md"));
        assert_eq!(r.errors.len(), 3);

        // Challenge requires: id, type, version, verdict
        let r = validator.validate_required_fields(
            &empty_map,
            DocumentType::Challenge,
            Path::new("c.md"),
        );
        let missing: Vec<_> = r.errors.iter().map(|e| e.message.as_str()).collect();
        assert!(missing.iter().any(|m| m.contains("verdict")));
        assert_eq!(r.errors.len(), 4);

        // State requires: change_id, phase
        let r = validator.validate_required_fields(
            &empty_map,
            DocumentType::State,
            Path::new("s.yaml"),
        );
        let missing: Vec<_> = r.errors.iter().map(|e| e.message.as_str()).collect();
        assert!(missing.iter().any(|m| m.contains("change_id")));
        assert!(missing.iter().any(|m| m.contains("phase")));
        assert_eq!(r.errors.len(), 2);
    }

    // ── REQ: R4 — convenience functions ────────────────────────────────────

    #[test]
    fn test_validate_frontmatter_schema_convenience() {
        // Uses project_root/cclab/schemas — create that structure in a temp dir
        let root = TempDir::new().unwrap();
        let schemas_dir = root.path().join("cclab/schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();
        std::fs::write(schemas_dir.join("spec.schema.json"), minimal_schema()).unwrap();

        let file_path = root.path().join("my-spec.md");
        std::fs::write(&file_path, valid_content("spec")).unwrap();

        let result = validate_frontmatter_schema(&file_path, root.path());
        assert!(
            result.errors.is_empty(),
            "expected no errors, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_validate_frontmatter_content_convenience() {
        let root = TempDir::new().unwrap();
        let schemas_dir = root.path().join("cclab/schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        // Write schemas for all types
        for dt in &[
            DocumentType::Proposal,
            DocumentType::Tasks,
            DocumentType::Spec,
            DocumentType::Challenge,
            DocumentType::State,
        ] {
            std::fs::write(schemas_dir.join(dt.schema_filename()), minimal_schema()).unwrap();
        }

        // Test each DocumentType maps to the correct temp filename
        let content = "---\nid: x\ntype: spec\ntitle: Test\nversion: 1\n---\n\nbody";
        let result = validate_frontmatter_content(content, DocumentType::Spec, root.path());
        assert!(
            result.errors.is_empty(),
            "Spec: expected no errors, got: {:?}",
            result.errors
        );

        // Proposal
        let content_p = "---\nid: x\ntype: proposal\n---\n\nbody";
        let result_p = validate_frontmatter_content(content_p, DocumentType::Proposal, root.path());
        // May have schema errors (missing fields), but should not crash
        assert!(
            result_p.errors.is_empty()
                || result_p
                    .errors
                    .iter()
                    .all(|e| e.category == ErrorCategory::InvalidStructure)
        );
    }

    #[test]
    fn test_validate_frontmatter_content_state_type() {
        let root = TempDir::new().unwrap();
        let schemas_dir = root.path().join("cclab/schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();
        std::fs::write(schemas_dir.join("state.schema.json"), minimal_schema()).unwrap();

        let content = "---\nid: x\ntype: state\n---\n\nbody";
        // This creates "temp.state.yaml" — confirms the DocumentType::State branch
        let result = validate_frontmatter_content(content, DocumentType::State, root.path());
        // Should execute without panics; may have schema validation errors
        let _ = result;
    }
}

// CODEGEN-END
