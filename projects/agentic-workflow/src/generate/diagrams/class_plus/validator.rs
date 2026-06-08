// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/validator.md#source
// CODEGEN-BEGIN
//! Class+ semantic validator

use super::schema::{ClassDiagramDef, ClassStereotype};
use std::collections::HashSet;

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClassSeverity {
    Error,
    Warning,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClassValidationError {
    pub code: String,
    pub message: String,
    pub path: String,
    pub severity: ClassSeverity,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct ClassValidationResult {
    pub valid: bool,
    pub errors: Vec<ClassValidationError>,
    pub warnings: Vec<ClassValidationError>,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/validator.md#source
impl ClassValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_error(mut self, error: ClassValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: ClassValidationError) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/validator.md#source
pub struct ClassValidator {
    strict: bool,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/validator.md#source
impl ClassValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn validate(&self, diagram: &ClassDiagramDef) -> ClassValidationResult {
        let mut result = ClassValidationResult::ok();
        let class_names: HashSet<String> = diagram.classes.keys().cloned().collect();

        // 1. Check for empty diagram
        if diagram.classes.is_empty() {
            result = result.with_error(ClassValidationError {
                code: "EMPTY_DIAGRAM".to_string(),
                message: "Class diagram must have at least one class".to_string(),
                path: "classes".to_string(),
                severity: ClassSeverity::Error,
            });
        }

        // 2. Validate relationship endpoints
        for (idx, rel) in diagram.relationships.iter().enumerate() {
            if !class_names.contains(&rel.from) {
                result = result.with_error(ClassValidationError {
                    code: "INVALID_RELATIONSHIP_FROM".to_string(),
                    message: format!("Relationship source '{}' not found", rel.from),
                    path: format!("relationships[{}].from", idx),
                    severity: ClassSeverity::Error,
                });
            }
            if !class_names.contains(&rel.to) {
                result = result.with_error(ClassValidationError {
                    code: "INVALID_RELATIONSHIP_TO".to_string(),
                    message: format!("Relationship target '{}' not found", rel.to),
                    path: format!("relationships[{}].to", idx),
                    severity: ClassSeverity::Error,
                });
            }
        }

        // 3. Validate namespace class references
        for (ns_idx, ns) in diagram.namespaces.iter().enumerate() {
            for (class_idx, class_name) in ns.classes.iter().enumerate() {
                if !class_names.contains(class_name) {
                    result = result.with_error(ClassValidationError {
                        code: "INVALID_NAMESPACE_CLASS".to_string(),
                        message: format!(
                            "Namespace '{}' references non-existent class '{}'",
                            ns.name, class_name
                        ),
                        path: format!("namespaces[{}].classes[{}]", ns_idx, class_idx),
                        severity: ClassSeverity::Error,
                    });
                }
            }
        }

        // 4. Validate interface constraints
        for (class_name, class_def) in &diagram.classes {
            if let Some(ClassStereotype::Interface) = &class_def.stereotype {
                // Interfaces should not have non-abstract methods with implementations
                for attr in &class_def.attributes {
                    if attr.default_value.is_some() {
                        result = result.with_warning(ClassValidationError {
                            code: "INTERFACE_WITH_STATE".to_string(),
                            message: format!(
                                "Interface '{}' has attribute with default value",
                                class_name
                            ),
                            path: format!("classes.{}.attributes", class_name),
                            severity: ClassSeverity::Warning,
                        });
                    }
                }
            }
        }

        // 5. Check for duplicate class in multiple namespaces
        let mut class_namespace_map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for ns in &diagram.namespaces {
            for class_name in &ns.classes {
                class_namespace_map
                    .entry(class_name.clone())
                    .or_default()
                    .push(ns.name.clone());
            }
        }
        for (class_name, namespaces) in &class_namespace_map {
            if namespaces.len() > 1 {
                result = result.with_warning(ClassValidationError {
                    code: "CLASS_IN_MULTIPLE_NAMESPACES".to_string(),
                    message: format!(
                        "Class '{}' is in multiple namespaces: {}",
                        class_name,
                        namespaces.join(", ")
                    ),
                    path: format!("classes.{}", class_name),
                    severity: ClassSeverity::Warning,
                });
            }
        }

        if self.strict {
            let strict_codes = ["INTERFACE_WITH_STATE", "CLASS_IN_MULTIPLE_NAMESPACES"];
            let (promoted, remaining): (Vec<_>, Vec<_>) = result
                .warnings
                .into_iter()
                .partition(|w| strict_codes.contains(&w.code.as_str()));
            result.warnings = remaining;
            for mut warning in promoted {
                warning.severity = ClassSeverity::Error;
                result.errors.push(warning);
            }
            if !result.errors.is_empty() {
                result.valid = false;
            }
        }

        result
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/validator.md#source
impl Default for ClassValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_diagram(json: serde_json::Value) -> ClassDiagramDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_valid_diagram() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "classes": {
                "A": {},
                "B": {}
            },
            "relationships": [
                { "from": "A", "to": "B", "type": "association" }
            ]
        }));

        let result = ClassValidator::new().validate(&diagram);
        assert!(result.valid);
    }

    #[test]
    fn test_invalid_relationship() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "classes": { "A": {} },
            "relationships": [
                { "from": "A", "to": "NonExistent", "type": "association" }
            ]
        }));

        let result = ClassValidator::new().validate(&diagram);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == "INVALID_RELATIONSHIP_TO"));
    }
}

// CODEGEN-END
