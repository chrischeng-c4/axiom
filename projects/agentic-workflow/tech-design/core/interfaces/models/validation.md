---
id: sdd-models-validation
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Validation Model Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/validation.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DocumentType` | projects/agentic-workflow/src/models/validation.rs | enum | pub | 11 |  |
| `ErrorCategory` | projects/agentic-workflow/src/models/validation.rs | enum | pub | 35 |  |
| `JsonValidationError` | projects/agentic-workflow/src/models/validation.rs | struct | pub | 166 |  |
| `Severity` | projects/agentic-workflow/src/models/validation.rs | enum | pub | 23 |  |
| `SeverityMap` | projects/agentic-workflow/src/models/validation.rs | struct | pub | 99 |  |
| `ValidationCounts` | projects/agentic-workflow/src/models/validation.rs | struct | pub | 154 |  |
| `ValidationError` | projects/agentic-workflow/src/models/validation.rs | struct | pub | 61 |  |
| `ValidationJsonOutput` | projects/agentic-workflow/src/models/validation.rs | struct | pub | 139 |  |
| `ValidationOptions` | projects/agentic-workflow/src/models/validation.rs | struct | pub | 125 |  |
| `ValidationResult` | projects/agentic-workflow/src/models/validation.rs | struct | pub | 117 |  |
| `ValidationRules` | projects/agentic-workflow/src/models/validation.rs | struct | pub | 77 |  |
| `count_by_severity` | projects/agentic-workflow/src/models/validation.rs | function | pub | 462 | count_by_severity(&self, severity: Severity) -> usize |
| `for_document_type` | projects/agentic-workflow/src/models/validation.rs | function | pub | 372 | for_document_type(doc_type: DocumentType) -> Self |
| `for_prd` | projects/agentic-workflow/src/models/validation.rs | function | pub | 303 | for_prd() -> Self |
| `for_spec` | projects/agentic-workflow/src/models/validation.rs | function | pub | 346 | for_spec() -> Self |
| `for_task` | projects/agentic-workflow/src/models/validation.rs | function | pub | 321 | for_task() -> Self |
| `format` | projects/agentic-workflow/src/models/validation.rs | function | pub | 239 | format(&self) -> String |
| `format_errors` | projects/agentic-workflow/src/models/validation.rs | function | pub | 478 | format_errors(&self) -> String |
| `from_path` | projects/agentic-workflow/src/models/validation.rs | function | pub | 184 | from_path(path: &std::path::Path) -> Self |
| `get` | projects/agentic-workflow/src/models/validation.rs | function | pub | 428 | get(&self, category: ErrorCategory) -> Severity |
| `has_errors` | projects/agentic-workflow/src/models/validation.rs | function | pub | 457 | has_errors(&self) -> bool |
| `high_severity_errors` | projects/agentic-workflow/src/models/validation.rs | function | pub | 470 | high_severity_errors(&self) -> Vec<&ValidationError> |
| `is_fixable` | projects/agentic-workflow/src/models/validation.rs | function | pub | 265 | is_fixable(&self) -> bool |
| `is_valid` | projects/agentic-workflow/src/models/validation.rs | function | pub | 452 | is_valid(&self) -> bool |
| `name` | projects/agentic-workflow/src/models/validation.rs | function | pub | 210 | name(&self) -> &'static str |
| `name` | projects/agentic-workflow/src/models/validation.rs | function | pub | 275 | name(&self) -> &'static str |
| `new` | projects/agentic-workflow/src/models/validation.rs | function | pub | 222 | new(         message: impl Into<String>,         file: impl Into<PathBuf>,         line: Option<usize>,         severity: Severity,         category: ErrorCategory,     ) -> Self |
| `new` | projects/agentic-workflow/src/models/validation.rs | function | pub | 447 | new(errors: Vec<ValidationError>) -> Self |
| `new` | projects/agentic-workflow/src/models/validation.rs | function | pub | 490 | new() -> Self |
| `symbol` | projects/agentic-workflow/src/models/validation.rs | function | pub | 201 | symbol(&self) -> &'static str |
| `with_fix` | projects/agentic-workflow/src/models/validation.rs | function | pub | 513 | with_fix(mut self, fix: bool) -> Self |
| `with_json` | projects/agentic-workflow/src/models/validation.rs | function | pub | 507 | with_json(mut self, json: bool) -> Self |
| `with_strict` | projects/agentic-workflow/src/models/validation.rs | function | pub | 495 | with_strict(mut self, strict: bool) -> Self |
| `with_verbose` | projects/agentic-workflow/src/models/validation.rs | function | pub | 501 | with_verbose(mut self, verbose: bool) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  DocumentType:
    type: string
    enum: [Prd, Task, Spec]
    description: Document type for type-specific validation.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]

  Severity:
    type: string
    enum: [High, Medium, Low]
    description: Severity level for validation errors.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]

  ErrorCategory:
    type: string
    enum: [MissingHeading, InvalidRequirementFormat, MissingScenario, MissingWhenThen, DuplicateRequirement, BrokenReference, InvalidStructure, EmptyContent, Inconsistency, CircularDependency]
    description: Category of validation error.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]

  ValidationError:
    type: object
    required: [message, file, line, severity, category]
    description: A validation error found in a spec file.
    properties:
      message:
        type: string
        description: "Error message describing what's wrong."
      file:
        type: string
        x-rust-type: "PathBuf"
        description: "File where the error was found."
      line:
        type: integer
        x-rust-type: "Option<usize>"
        description: "Line number (1-indexed)."
      severity:
        type: string
        x-rust-type: "Severity"
        description: "Severity level."
      category:
        type: string
        x-rust-type: "ErrorCategory"
        description: "Error category for grouping."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ValidationRules:
    type: object
    required: [required_headings, requirement_pattern, scenario_pattern, scenario_min_count, require_when_then, when_pattern, then_pattern, severity_map]
    description: Validation rules loaded from configuration.
    properties:
      required_headings:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Required top-level headings (in order)."
      requirement_pattern:
        type: string
        description: "Regex pattern for requirement naming."
      scenario_pattern:
        type: string
        description: "Regex pattern for scenario format."
      scenario_min_count:
        type: integer
        x-rust-type: "usize"
        description: "Minimum number of scenarios per requirement."
      require_when_then:
        type: boolean
        description: "Whether to require WHEN/THEN clauses."
      when_pattern:
        type: string
        description: "Pattern for WHEN clause."
      then_pattern:
        type: string
        description: "Pattern for THEN clause."
      severity_map:
        type: object
        x-rust-type: "SeverityMap"
        description: "Severity mapping for different error types."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SeverityMap:
    type: object
    required: [missing_heading, invalid_requirement_format, missing_scenario, missing_when_then, duplicate_requirement, broken_reference]
    description: Mapping of error categories to severity levels.
    properties:
      missing_heading:
        type: string
        x-rust-type: "Severity"
        description: "Severity for missing heading."
      invalid_requirement_format:
        type: string
        x-rust-type: "Severity"
        description: "Severity for invalid requirement format."
      missing_scenario:
        type: string
        x-rust-type: "Severity"
        description: "Severity for missing scenario."
      missing_when_then:
        type: string
        x-rust-type: "Severity"
        description: "Severity for missing WHEN/THEN."
      duplicate_requirement:
        type: string
        x-rust-type: "Severity"
        description: "Severity for duplicate requirement."
      broken_reference:
        type: string
        x-rust-type: "Severity"
        description: "Severity for broken reference."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ValidationResult:
    type: object
    required: [errors]
    description: Result of validation.
    properties:
      errors:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ValidationError>"
        description: "List of all validation errors found."
    x-rust-struct:
      derive: [Debug]

  ValidationOptions:
    type: object
    required: [strict, verbose, json, fix]
    description: Validation CLI options.
    properties:
      strict:
        type: boolean
        description: "Treat warnings as errors."
      verbose:
        type: boolean
        description: "Show verbose output."
      json:
        type: boolean
        description: "Output as JSON."
      fix:
        type: boolean
        description: "Attempt to auto-fix fixable errors."
    x-rust-struct:
      derive: [Debug, Clone, Default]

  ValidationJsonOutput:
    type: object
    required: [valid, counts, errors, stale_files]
    description: JSON output format for validation results.
    properties:
      valid:
        type: boolean
        description: "Whether validation passed."
      counts:
        type: object
        x-rust-type: "ValidationCounts"
        description: "Counts by severity."
      errors:
        type: array
        items: { type: object }
        x-rust-type: "Vec<JsonValidationError>"
        description: "List of all errors."
      stale_files:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-skip-if: "Vec::is_empty"
        description: "Stale files detected."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ValidationCounts:
    type: object
    required: [high, medium, low]
    description: Validation counts by severity.
    properties:
      high:
        type: integer
        x-rust-type: "usize"
        description: "High severity count."
      medium:
        type: integer
        x-rust-type: "usize"
        description: "Medium severity count."
      low:
        type: integer
        x-rust-type: "usize"
        description: "Low severity count."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  JsonValidationError:
    type: object
    required: [severity, category, message, file, line]
    description: JSON-friendly validation error.
    properties:
      severity:
        type: string
        description: "Severity name."
      category:
        type: string
        description: "Category name."
      message:
        type: string
        description: "Error message."
      file:
        type: string
        x-rust-type: "Option<String>"
        x-serde-skip-if: "Option::is_none"
        description: "Optional file path."
      line:
        type: integer
        x-rust-type: "Option<usize>"
        x-serde-skip-if: "Option::is_none"
        description: "Optional line number."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/validation.rs -->
```rust
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Document type for type-specific validation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    #[serde(rename = "Prd")]
    Prd,
    #[serde(rename = "Task")]
    Task,
    #[serde(rename = "Spec")]
    Spec,
}

/// Severity level for validation errors.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    #[serde(rename = "High")]
    High,
    #[serde(rename = "Medium")]
    Medium,
    #[serde(rename = "Low")]
    Low,
}

/// Category of validation error.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    #[serde(rename = "MissingHeading")]
    MissingHeading,
    #[serde(rename = "InvalidRequirementFormat")]
    InvalidRequirementFormat,
    #[serde(rename = "MissingScenario")]
    MissingScenario,
    #[serde(rename = "MissingWhenThen")]
    MissingWhenThen,
    #[serde(rename = "DuplicateRequirement")]
    DuplicateRequirement,
    #[serde(rename = "BrokenReference")]
    BrokenReference,
    #[serde(rename = "InvalidStructure")]
    InvalidStructure,
    #[serde(rename = "EmptyContent")]
    EmptyContent,
    #[serde(rename = "Inconsistency")]
    Inconsistency,
    #[serde(rename = "CircularDependency")]
    CircularDependency,
}

/// A validation error found in a spec file.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message describing what's wrong.
    pub message: String,
    /// File where the error was found.
    pub file: PathBuf,
    /// Line number (1-indexed).
    pub line: Option<usize>,
    /// Severity level.
    pub severity: Severity,
    /// Error category for grouping.
    pub category: ErrorCategory,
}

/// Validation rules loaded from configuration.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Required top-level headings (in order).
    pub required_headings: Vec<String>,
    /// Regex pattern for requirement naming.
    pub requirement_pattern: String,
    /// Regex pattern for scenario format.
    pub scenario_pattern: String,
    /// Minimum number of scenarios per requirement.
    pub scenario_min_count: usize,
    /// Whether to require WHEN/THEN clauses.
    pub require_when_then: bool,
    /// Pattern for WHEN clause.
    pub when_pattern: String,
    /// Pattern for THEN clause.
    pub then_pattern: String,
    /// Severity mapping for different error types.
    pub severity_map: SeverityMap,
}

/// Mapping of error categories to severity levels.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityMap {
    /// Severity for missing heading.
    pub missing_heading: Severity,
    /// Severity for invalid requirement format.
    pub invalid_requirement_format: Severity,
    /// Severity for missing scenario.
    pub missing_scenario: Severity,
    /// Severity for missing WHEN/THEN.
    pub missing_when_then: Severity,
    /// Severity for duplicate requirement.
    pub duplicate_requirement: Severity,
    /// Severity for broken reference.
    pub broken_reference: Severity,
}

/// Result of validation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug)]
pub struct ValidationResult {
    /// List of all validation errors found.
    pub errors: Vec<ValidationError>,
}

/// Validation CLI options.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Default)]
pub struct ValidationOptions {
    /// Treat warnings as errors.
    pub strict: bool,
    /// Show verbose output.
    pub verbose: bool,
    /// Output as JSON.
    pub json: bool,
    /// Attempt to auto-fix fixable errors.
    pub fix: bool,
}

/// JSON output format for validation results.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationJsonOutput {
    /// Whether validation passed.
    pub valid: bool,
    /// Counts by severity.
    pub counts: ValidationCounts,
    /// List of all errors.
    pub errors: Vec<JsonValidationError>,
    /// Stale files detected.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stale_files: Vec<String>,
}

/// Validation counts by severity.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCounts {
    /// High severity count.
    pub high: usize,
    /// Medium severity count.
    pub medium: usize,
    /// Low severity count.
    pub low: usize,
}

/// JSON-friendly validation error.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonValidationError {
    /// Severity name.
    pub severity: String,
    /// Category name.
    pub category: String,
    /// Error message.
    pub message: String,
    /// Optional file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Optional line number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl DocumentType {
    /// Determine document type from file path
    pub fn from_path(path: &std::path::Path) -> Self {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name == "proposal.md" {
            DocumentType::Prd
        } else if file_name == "tasks.md" {
            DocumentType::Task
        } else {
            // Default to Spec for files in specs/ directory or any other .md files
            DocumentType::Spec
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl Severity {
    /// Get display symbol for severity
    pub fn symbol(&self) -> &'static str {
        match self {
            Severity::High => "🔴",
            Severity::Medium => "🟡",
            Severity::Low => "🔵",
        }
    }

    /// Get display name for severity
    pub fn name(&self) -> &'static str {
        match self {
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl ValidationError {
    /// Create a new validation error
    pub fn new(
        message: impl Into<String>,
        file: impl Into<PathBuf>,
        line: Option<usize>,
        severity: Severity,
        category: ErrorCategory,
    ) -> Self {
        Self {
            message: message.into(),
            file: file.into(),
            line,
            severity,
            category,
        }
    }

    /// Format error for display
    pub fn format(&self) -> String {
        let file_display = self.file.display();
        if let Some(line) = self.line {
            format!(
                "{} [{}] {}:{} - {}",
                self.severity.symbol(),
                self.severity.name(),
                file_display,
                line,
                self.message
            )
        } else {
            format!(
                "{} [{}] {} - {}",
                self.severity.symbol(),
                self.severity.name(),
                file_display,
                self.message
            )
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl ErrorCategory {
    /// Check if this error category can be automatically fixed
    pub fn is_fixable(&self) -> bool {
        matches!(
            self,
            ErrorCategory::MissingHeading
                | ErrorCategory::MissingWhenThen
                | ErrorCategory::MissingScenario
        )
    }

    /// Get display name for category
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::MissingHeading => "Missing Heading",
            ErrorCategory::InvalidRequirementFormat => "Invalid Requirement Format",
            ErrorCategory::MissingScenario => "Missing Scenario",
            ErrorCategory::MissingWhenThen => "Missing WHEN/THEN",
            ErrorCategory::DuplicateRequirement => "Duplicate Requirement",
            ErrorCategory::BrokenReference => "Broken Reference",
            ErrorCategory::InvalidStructure => "Invalid Structure",
            ErrorCategory::EmptyContent => "Empty Content",
            ErrorCategory::Inconsistency => "Inconsistency",
            ErrorCategory::CircularDependency => "Circular Dependency",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl Default for ValidationRules {
    fn default() -> Self {
        // Default rules are for Spec documents (most restrictive)
        Self::for_spec()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl ValidationRules {
    /// Get validation rules for PRD (proposal.md)
    /// PRD documents are less strict - they describe "why" and "what", not technical details
    pub fn for_prd() -> Self {
        Self {
            required_headings: vec![
                // PRD doesn't require spec-style headings
                // Common PRD sections are: Summary, Why, What Changes, Impact
            ],
            requirement_pattern: String::new(), // No requirement pattern for PRD
            scenario_pattern: String::new(),    // No scenarios required
            scenario_min_count: 0,              // No minimum scenarios
            require_when_then: false,           // No WHEN/THEN required
            when_pattern: String::new(),
            then_pattern: String::new(),
            severity_map: SeverityMap::default(),
        }
    }

    /// Get validation rules for Task list (tasks.md)
    /// Task documents describe implementation tasks, not formal specifications
    pub fn for_task() -> Self {
        Self {
            required_headings: vec![
                // Task files should have a "Tasks" heading or numbered sections
                // But we'll be lenient and not require specific headings
            ],
            requirement_pattern: String::new(), // No formal requirements
            scenario_pattern: String::new(),    // No scenarios in task files
            scenario_min_count: 0,              // No minimum scenarios
            require_when_then: false,           // No WHEN/THEN required
            when_pattern: String::new(),
            then_pattern: String::new(),
            severity_map: SeverityMap::default(),
        }
    }

    /// Get validation rules for Spec (Technical Design) documents
    ///
    /// Spec format (TD + AC):
    /// - # Spec: [Feature Name]
    /// - ## Overview
    /// - ## Flow (Mermaid diagrams)
    /// - ## Data Model (JSON Schema)
    /// - ## Interfaces (pseudo code)
    /// - ## Acceptance Criteria (WHEN/THEN)
    pub fn for_spec() -> Self {
        // Use central spec format rules
        let spec_rules = crate::models::spec_rules::SpecFormatRules::spec_defaults();

        Self {
            required_headings: spec_rules.required_headings.clone(),
            requirement_pattern: spec_rules.requirement_pattern.clone().unwrap_or_default(),
            // Use regex pattern from central rules (supports multiline format)
            scenario_pattern: spec_rules.scenario_regex_pattern(),
            scenario_min_count: spec_rules.min_scenarios,
            require_when_then: spec_rules.require_when_then,
            // Flexible patterns - match both plain "WHEN" and bold "**WHEN**"
            // Use \* to match literal asterisk (in raw string, \* becomes literal \* in regex)
            when_pattern: format!(
                r"\*\*{}\*\*|{}",
                spec_rules.when_keyword, spec_rules.when_keyword
            ),
            then_pattern: format!(
                r"\*\*{}\*\*|{}",
                spec_rules.then_keyword, spec_rules.then_keyword
            ),
            severity_map: SeverityMap::default(),
        }
    }

    /// Get validation rules based on document type
    pub fn for_document_type(doc_type: DocumentType) -> Self {
        match doc_type {
            DocumentType::Prd => Self::for_prd(),
            DocumentType::Task => Self::for_task(),
            DocumentType::Spec => Self::for_spec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_rules_use_central_format() {
        let rules = ValidationRules::for_spec();

        // Should use the multiline pattern
        assert!(
            rules.scenario_pattern.contains("(?m)"),
            "Spec rules should use multiline pattern, got: {}",
            rules.scenario_pattern
        );
        assert!(
            rules.scenario_pattern.contains("###"),
            "Spec rules should check for ### Scenario headings"
        );

        // Should require at least 1 scenario
        assert_eq!(
            rules.scenario_min_count, 1,
            "Spec should require at least 1 scenario"
        );

        // Should require WHEN/THEN
        assert!(rules.require_when_then, "Spec should require WHEN/THEN");
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl Default for SeverityMap {
    fn default() -> Self {
        Self {
            missing_heading: Severity::High,
            invalid_requirement_format: Severity::High,
            missing_scenario: Severity::High,
            missing_when_then: Severity::High,
            duplicate_requirement: Severity::High,
            broken_reference: Severity::Medium,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl SeverityMap {
    /// Get severity for a given error category
    pub fn get(&self, category: ErrorCategory) -> Severity {
        match category {
            ErrorCategory::MissingHeading => self.missing_heading,
            ErrorCategory::InvalidRequirementFormat => self.invalid_requirement_format,
            ErrorCategory::MissingScenario => self.missing_scenario,
            ErrorCategory::MissingWhenThen => self.missing_when_then,
            ErrorCategory::DuplicateRequirement => self.duplicate_requirement,
            ErrorCategory::BrokenReference => self.broken_reference,
            ErrorCategory::InvalidStructure => Severity::High,
            ErrorCategory::EmptyContent => Severity::High,
            ErrorCategory::Inconsistency => Severity::High,
            ErrorCategory::CircularDependency => Severity::High,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl ValidationResult {
    /// Create a new validation result
    pub fn new(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }

    /// Check if validation passed (no high-severity errors)
    pub fn is_valid(&self) -> bool {
        !self.errors.iter().any(|e| e.severity == Severity::High)
    }

    /// Check if there are any errors at all
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Count errors by severity
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.errors
            .iter()
            .filter(|e| e.severity == severity)
            .count()
    }

    /// Get all high-severity errors (blocking)
    pub fn high_severity_errors(&self) -> Vec<&ValidationError> {
        self.errors
            .iter()
            .filter(|e| e.severity == Severity::High)
            .collect()
    }

    /// Format all errors for display
    pub fn format_errors(&self) -> String {
        self.errors
            .iter()
            .map(|e| e.format())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl ValidationOptions {
    /// Create new options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set strict mode
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Set verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set JSON output mode
    pub fn with_json(mut self, json: bool) -> Self {
        self.json = json;
        self
    }

    /// Set fix mode
    pub fn with_fix(mut self, fix: bool) -> Self {
        self.fix = fix;
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/validation.md#source
impl From<&ValidationError> for JsonValidationError {
    fn from(error: &ValidationError) -> Self {
        Self {
            severity: error.severity.name().to_string(),
            category: error.category.name().to_string(),
            message: error.message.clone(),
            file: Some(error.file.display().to_string()),
            line: error.line,
        }
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/validation.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete validation model module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 11 types: 3 enums + 8 structs.
- [schema] All in `required:`; foreign types via x-rust-type; per-field skip_serializing_if.
- [changes] All eleven in `replaces`; 11 impl blocks all preserved hand-written.
