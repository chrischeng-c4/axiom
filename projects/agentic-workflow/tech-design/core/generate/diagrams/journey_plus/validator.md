---
id: sdd-journey-plus-validator-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Journey Plus Validator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `JourneySeverity` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | enum | pub | 12 |  |
| `JourneyValidationError` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | struct | pub | 20 |  |
| `JourneyValidationResult` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | struct | pub | 30 |  |
| `JourneyValidator` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | struct | pub | 58 |  |
| `new` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | function | pub | 64 | new() -> Self |
| `ok` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | function | pub | 37 | ok() -> Self |
| `strict` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | function | pub | 68 | strict(mut self) -> Self |
| `validate` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | function | pub | 73 | validate(&self, journey: &JourneyDef) -> JourneyValidationResult |
| `with_error` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | function | pub | 45 | with_error(mut self, error: JourneyValidationError) -> Self |
| `with_warning` | projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs | function | pub | 51 | with_warning(mut self, warning: JourneyValidationError) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  JourneySeverity:
    type: string
    enum: [Error, Warning]
    description: Severity of a journey validation finding.
    x-rust-enum:
      derive: [Debug, Clone, "serde::Serialize", PartialEq]
      serde_rename_all: lowercase

  JourneyValidationError:
    type: object
    required: [code, message, path, severity]
    description: A single validation finding.
    properties:
      code:
        type: string
      message:
        type: string
      path:
        type: string
      severity:
        $ref: "#/definitions/JourneySeverity"
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]

  JourneyValidationResult:
    type: object
    required: [valid, errors, warnings]
    description: Aggregate validation outcome.
    properties:
      valid:
        type: boolean
      errors:
        type: array
        items:
          $ref: "#/definitions/JourneyValidationError"
      warnings:
        type: array
        items:
          $ref: "#/definitions/JourneyValidationError"
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize", Default]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs -->
```rust
//! Journey+ semantic validator

use super::schema::JourneyDef;
use std::collections::HashSet;

/// Severity of a journey validation finding.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JourneySeverity {
    Error,
    Warning,
}

/// A single validation finding.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct JourneyValidationError {
    pub code: String,
    pub message: String,
    pub path: String,
    pub severity: JourneySeverity,
}

/// Aggregate validation outcome.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct JourneyValidationResult {
    pub valid: bool,
    pub errors: Vec<JourneyValidationError>,
    pub warnings: Vec<JourneyValidationError>,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/validator.md#source
impl JourneyValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_error(mut self, error: JourneyValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: JourneyValidationError) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/validator.md#source
pub struct JourneyValidator {
    strict: bool,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/validator.md#source
impl JourneyValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn validate(&self, journey: &JourneyDef) -> JourneyValidationResult {
        let mut result = JourneyValidationResult::ok();

        // 1. Check for empty title
        if journey.title.trim().is_empty() {
            result = result.with_error(JourneyValidationError {
                code: "EMPTY_TITLE".to_string(),
                message: "Journey must have a non-empty title".to_string(),
                path: "title".to_string(),
                severity: JourneySeverity::Error,
            });
        }

        // 2. Check for empty sections
        if journey.sections.is_empty() {
            result = result.with_error(JourneyValidationError {
                code: "EMPTY_SECTIONS".to_string(),
                message: "Journey must have at least one section".to_string(),
                path: "sections".to_string(),
                severity: JourneySeverity::Error,
            });
        }

        // Collect all actors for consistency check
        let mut all_actors: HashSet<String> = HashSet::new();

        // 3. Validate sections and tasks
        for (sec_idx, section) in journey.sections.iter().enumerate() {
            // Check section name
            if section.name.trim().is_empty() {
                result = result.with_error(JourneyValidationError {
                    code: "EMPTY_SECTION_NAME".to_string(),
                    message: "Section must have a non-empty name".to_string(),
                    path: format!("sections[{}].name", sec_idx),
                    severity: JourneySeverity::Error,
                });
            }

            // Check section has tasks
            if section.tasks.is_empty() {
                result = result.with_warning(JourneyValidationError {
                    code: "EMPTY_SECTION_TASKS".to_string(),
                    message: format!("Section '{}' has no tasks", section.name),
                    path: format!("sections[{}].tasks", sec_idx),
                    severity: JourneySeverity::Warning,
                });
            }

            // Validate tasks
            for (task_idx, task) in section.tasks.iter().enumerate() {
                let task_path = format!("sections[{}].tasks[{}]", sec_idx, task_idx);

                // Check task name
                if task.name.trim().is_empty() {
                    result = result.with_error(JourneyValidationError {
                        code: "EMPTY_TASK_NAME".to_string(),
                        message: "Task must have a non-empty name".to_string(),
                        path: format!("{}.name", task_path),
                        severity: JourneySeverity::Error,
                    });
                }

                // Check score range (1-5)
                if task.score < 1 || task.score > 5 {
                    result = result.with_error(JourneyValidationError {
                        code: "INVALID_SCORE".to_string(),
                        message: format!(
                            "Task '{}' has invalid score {} (must be 1-5)",
                            task.name, task.score
                        ),
                        path: format!("{}.aw", task_path),
                        severity: JourneySeverity::Error,
                    });
                }

                // Check actors
                if task.actors.is_empty() {
                    result = result.with_error(JourneyValidationError {
                        code: "NO_ACTORS".to_string(),
                        message: format!("Task '{}' must have at least one actor", task.name),
                        path: format!("{}.actors", task_path),
                        severity: JourneySeverity::Error,
                    });
                }

                for actor in &task.actors {
                    all_actors.insert(actor.clone());
                }
            }
        }

        // 4. Warn if only one actor (might indicate missing collaboration modeling)
        if all_actors.len() == 1 {
            result = result.with_warning(JourneyValidationError {
                code: "SINGLE_ACTOR".to_string(),
                message: "Journey has only one actor, consider modeling collaborations".to_string(),
                path: "sections".to_string(),
                severity: JourneySeverity::Warning,
            });
        }

        if self.strict {
            let strict_codes = ["EMPTY_SECTION_TASKS", "SINGLE_ACTOR"];
            let (promoted, remaining): (Vec<_>, Vec<_>) = result
                .warnings
                .into_iter()
                .partition(|w| strict_codes.contains(&w.code.as_str()));
            result.warnings = remaining;
            for mut warning in promoted {
                warning.severity = JourneySeverity::Error;
                result.errors.push(warning);
            }
            if !result.errors.is_empty() {
                result.valid = false;
            }
        }

        result
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/validator.md#source
impl Default for JourneyValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_journey(json: serde_json::Value) -> JourneyDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_valid_journey() {
        let journey = parse_journey(json!({
            "id": "test",
            "title": "Test Journey",
            "sections": [
                {
                    "name": "Start",
                    "tasks": [
                        { "name": "Task 1", "score": 4, "actors": ["User", "System"] }
                    ]
                }
            ]
        }));

        let result = JourneyValidator::new().validate(&journey);
        assert!(result.valid);
    }

    #[test]
    fn test_invalid_score() {
        let journey = parse_journey(json!({
            "id": "test",
            "title": "Test",
            "sections": [
                {
                    "name": "Section",
                    "tasks": [
                        { "name": "Task", "score": 6, "actors": ["User"] }
                    ]
                }
            ]
        }));

        let result = JourneyValidator::new().validate(&journey);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "INVALID_SCORE"));
    }

    #[test]
    fn test_no_actors() {
        let journey = parse_journey(json!({
            "id": "test",
            "title": "Test",
            "sections": [
                {
                    "name": "Section",
                    "tasks": [
                        { "name": "Task", "score": 3, "actors": [] }
                    ]
                }
            ]
        }));

        let result = JourneyValidator::new().validate(&journey);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "NO_ACTORS"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/journey_plus/validator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Journey+ validator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview/schema/changes] OK.
