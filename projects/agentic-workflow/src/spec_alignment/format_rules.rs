// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/format_rules_preamble_source.md#source
// CODEGEN-BEGIN
//! Format compliance rules for spec alignment checking.
//!
//! Three rules:
//! - `missing_section_annotation`: every `## Heading` must have an annotation
//! - `duplicate_section`: no duplicate heading text within a file
//! - `format_priority_violation`: typed sections must contain matching code blocks

use std::collections::HashMap;

#[cfg(test)]
use super::models::{CodeBlock, SectionAnnotation, SpecSection};
use super::models::{SpecDocument, Violation, ViolationKind};
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/format_rules_runtime_source.md#source
// CODEGEN-BEGIN
// @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R2
// @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R3
// @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R4
// @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R5
/// Section types that require a code block of the declared lang.
/// Maps section_type -> required code fence lang.
///
/// All structured data sections use yaml (not json).
/// requirements and unit-test use mermaid (Mermaid Plus requirementDiagram).
/// scenarios uses yaml (GWT structured format).
const REQUIRED_CODE_BLOCK_TYPES: &[(&str, &str)] = &[
    ("config", "yaml"), // was "json" — yaml is more token-efficient
    ("logic", "mermaid"),
    ("rpc-api", "yaml"), // was "json" — OpenRPC as YAML
    ("state-machine", "mermaid"),
    ("cli", "yaml"),
    ("changes", "yaml"),
    ("schema", "yaml"), // was "json" — JSON Schema as YAML
    ("rest-api", "yaml"),
    ("async-api", "yaml"),
    ("db-model", "mermaid"),
    ("dependency", "mermaid"),
    ("interaction", "mermaid"),
    ("wireframe", "yaml"),
    ("component", "yaml"),    // was "json" — Custom Elements Manifest as YAML
    ("design-token", "yaml"), // was "json" — W3C DTCG as YAML
    ("runtime-image", "yaml"),
    ("deployment", "yaml"),
    ("tool-contract", "yaml"), // AW -> native vat/rig/meter/guard/arena manifest bridge
    ("mindmap", "mermaid"),
    ("requirements", "mermaid"), // Mermaid Plus requirementDiagram (SysML v1.6)
    ("unit-test", "mermaid"),    // Mermaid Plus requirementDiagram with verifies
    ("e2e-test", "yaml"),        // Product journey + side-effect assertions
    ("test-plan", "mermaid"),    // Legacy alias accepted during migration
    ("tests", "yaml"),           // Legacy alias accepted during migration
    ("scenarios", "yaml"),       // YAML GWT structured format
];

/// Prose-only section types exempt from code block requirements.
/// Only overview and doc remain prose-only.
const PROSE_ONLY_TYPES: &[&str] = &["overview", "doc"];

/// Run all format compliance rules against a parsed `SpecDocument`.
///
/// Returns a list of violations found.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/format_rules_runtime_source.md#source
pub fn check(doc: &SpecDocument) -> Vec<Violation> {
    let mut violations = Vec::new();

    check_missing_annotations(doc, &mut violations);
    check_duplicate_sections(doc, &mut violations);
    check_format_priority(doc, &mut violations);

    violations
}

/// R2: Every `## Heading` must have a `<!-- type: X lang: Y -->` annotation.
fn check_missing_annotations(doc: &SpecDocument, violations: &mut Vec<Violation>) {
    for section in &doc.sections {
        if section.annotation.is_none() {
            violations.push(Violation {
                kind: ViolationKind::MissingSectionAnnotation,
                message: format!(
                    "Section '{}' at line {} has no type annotation (expected <!-- type: X lang: Y -->)",
                    section.heading, section.line
                ),
                heading: Some(section.heading.clone()),
                line: Some(section.line),
                lines: None,
                name: None,
                expected_lang: None,
                field: None,
                details: None,
            });
        }
    }
}

/// R3: No duplicate `## Heading` text within a single file.
fn check_duplicate_sections(doc: &SpecDocument, violations: &mut Vec<Violation>) {
    let mut heading_lines: HashMap<&str, Vec<usize>> = HashMap::new();

    for section in &doc.sections {
        heading_lines
            .entry(&section.heading)
            .or_default()
            .push(section.line);
    }

    for (heading, lines) in &heading_lines {
        if lines.len() > 1 {
            violations.push(Violation {
                kind: ViolationKind::DuplicateSection,
                message: format!(
                    "Duplicate section heading '{}' at lines {:?}",
                    heading, lines
                ),
                heading: Some(heading.to_string()),
                line: Some(lines[0]),
                lines: Some(lines.clone()),
                name: None,
                expected_lang: None,
                field: None,
                details: None,
            });
        }
    }
}

/// R4: Sections typed with a code-requiring type must contain at least one
/// matching code fence. Prose-only types are exempt.
///
/// REQ: change-spec.md#NAP2 — sections whose body is the bare `N/A` sentinel
/// are also exempt; the author has explicitly declared the section inapplicable.
/// The prune step (`prune_todo_sections`) removes them before review/commit.
fn check_format_priority(doc: &SpecDocument, violations: &mut Vec<Violation>) {
    for section in &doc.sections {
        let annotation = match &section.annotation {
            Some(a) => a,
            None => continue, // Missing annotation is caught by R2
        };

        // Skip prose-only types
        if PROSE_ONLY_TYPES.contains(&annotation.section_type.as_str()) {
            continue;
        }

        // REQ: change-spec.md#NAP2 — N/A sentinel exempts the section.
        if section.body.trim() == "N/A" {
            continue;
        }

        // Find the required lang for this section type
        let required_lang = REQUIRED_CODE_BLOCK_TYPES
            .iter()
            .find(|(st, _)| *st == annotation.section_type.as_str())
            .map(|(_, lang)| *lang);

        let required_lang = match required_lang {
            Some(l) => l,
            None => continue, // Unknown section type — no format rule
        };

        // Check if any code block matches the required lang
        let has_matching_block = section
            .code_blocks
            .iter()
            .any(|cb| cb.lang == required_lang);

        if !has_matching_block {
            violations.push(Violation {
                kind: ViolationKind::FormatPriorityViolation,
                message: format!(
                    "Section '{}' (type: {}) requires a ```{} code block but none found",
                    section.heading, annotation.section_type, required_lang
                ),
                heading: Some(section.heading.clone()),
                line: Some(section.line),
                lines: None,
                name: None,
                expected_lang: Some(required_lang.to_string()),
                field: None,
                details: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_doc(sections: Vec<SpecSection>) -> SpecDocument {
        SpecDocument {
            path: "test.md".to_string(),
            frontmatter: json!({}),
            sections,
        }
    }

    fn section_with_block(
        heading: &str,
        section_type: &str,
        lang_annotation: &str,
        block_lang: &str,
    ) -> SpecSection {
        SpecSection {
            heading: heading.to_string(),
            line: 1,
            annotation: Some(SectionAnnotation {
                section_type: section_type.to_string(),
                lang: lang_annotation.to_string(),
                attributes: Default::default(),
            }),
            code_blocks: vec![CodeBlock {
                lang: block_lang.to_string(),
                line: 2,
                content: "content".to_string(),
                parsed_json: None,
            }],
            body: String::new(),
        }
    }

    fn section_no_block(heading: &str, section_type: &str, lang_annotation: &str) -> SpecSection {
        SpecSection {
            heading: heading.to_string(),
            line: 1,
            annotation: Some(SectionAnnotation {
                section_type: section_type.to_string(),
                lang: lang_annotation.to_string(),
                attributes: Default::default(),
            }),
            code_blocks: vec![],
            body: String::new(),
        }
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R2
    #[test]
    fn test_schema_requires_yaml_not_json() {
        let doc_yaml = make_doc(vec![section_with_block("Schema", "schema", "yaml", "yaml")]);
        let doc_json = make_doc(vec![section_with_block("Schema", "schema", "yaml", "json")]);

        assert!(
            check(&doc_yaml).is_empty(),
            "yaml block should pass for schema"
        );
        assert!(
            check(&doc_json)
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "json block should fail for schema"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R2
    #[test]
    fn test_rpc_api_requires_yaml_not_json() {
        let doc_yaml = make_doc(vec![section_with_block(
            "RPC API", "rpc-api", "yaml", "yaml",
        )]);
        let doc_json = make_doc(vec![section_with_block(
            "RPC API", "rpc-api", "yaml", "json",
        )]);

        assert!(check(&doc_yaml).is_empty(), "yaml should pass for rpc-api");
        assert!(
            check(&doc_json)
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "json should fail for rpc-api"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R2
    #[test]
    fn test_config_requires_yaml() {
        let doc_yaml = make_doc(vec![section_with_block("Config", "config", "yaml", "yaml")]);
        assert!(check(&doc_yaml).is_empty(), "yaml should pass for config");

        let doc_json = make_doc(vec![section_with_block("Config", "config", "yaml", "json")]);
        assert!(
            check(&doc_json)
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "json should fail for config"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R4
    #[test]
    fn test_requirements_requires_mermaid() {
        let doc_mermaid = make_doc(vec![section_with_block(
            "Requirements",
            "requirements",
            "mermaid",
            "mermaid",
        )]);
        assert!(
            check(&doc_mermaid).is_empty(),
            "mermaid should pass for requirements"
        );

        let doc_no_block = make_doc(vec![section_no_block(
            "Requirements",
            "requirements",
            "mermaid",
        )]);
        assert!(
            check(&doc_no_block)
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "no mermaid block should fail for requirements"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R5
    #[test]
    fn test_scenarios_requires_yaml() {
        let doc_yaml = make_doc(vec![section_with_block(
            "Scenarios",
            "scenarios",
            "yaml",
            "yaml",
        )]);
        assert!(
            check(&doc_yaml).is_empty(),
            "yaml should pass for scenarios"
        );

        let doc_no_block = make_doc(vec![section_no_block("Scenarios", "scenarios", "yaml")]);
        assert!(
            check(&doc_no_block)
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "no yaml block should fail for scenarios"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R4
    #[test]
    fn test_unit_test_requires_mermaid() {
        let doc_mermaid = make_doc(vec![section_with_block(
            "Unit Test",
            "unit-test",
            "mermaid",
            "mermaid",
        )]);
        assert!(
            check(&doc_mermaid).is_empty(),
            "mermaid should pass for unit-test"
        );

        let doc_no_block = make_doc(vec![section_no_block("Unit Test", "unit-test", "mermaid")]);
        assert!(
            check(&doc_no_block)
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "no mermaid block should fail for unit-test"
        );
    }

    #[test]
    fn test_e2e_test_requires_yaml() {
        let doc_yaml = make_doc(vec![section_with_block(
            "E2E Test", "e2e-test", "yaml", "yaml",
        )]);
        assert!(check(&doc_yaml).is_empty(), "yaml should pass for e2e-test");

        let doc_no_block = make_doc(vec![section_no_block("E2E Test", "e2e-test", "yaml")]);
        assert!(
            check(&doc_no_block)
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "no yaml block should fail for e2e-test"
        );
    }

    #[test]
    fn test_overview_prose_only_no_code_required() {
        let doc = make_doc(vec![SpecSection {
            heading: "Overview".to_string(),
            line: 1,
            annotation: Some(SectionAnnotation {
                section_type: "overview".to_string(),
                lang: "markdown".to_string(),
                attributes: Default::default(),
            }),
            code_blocks: vec![],
            body: String::new(),
        }]);
        let violations = check(&doc);
        assert!(
            !violations
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "overview should not require code block"
        );
    }

    #[test]
    fn test_doc_prose_only_no_code_required() {
        let doc = make_doc(vec![SpecSection {
            heading: "Doc".to_string(),
            line: 1,
            annotation: Some(SectionAnnotation {
                section_type: "doc".to_string(),
                lang: "markdown".to_string(),
                attributes: Default::default(),
            }),
            code_blocks: vec![],
            body: String::new(),
        }]);
        let violations = check(&doc);
        assert!(
            !violations
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "doc should not require code block"
        );
    }

    #[test]
    fn test_missing_annotation_reported() {
        let doc = make_doc(vec![SpecSection {
            heading: "Unannotated Section".to_string(),
            line: 5,
            annotation: None,
            code_blocks: vec![],
            body: String::new(),
        }]);
        let violations = check(&doc);
        assert!(
            violations
                .iter()
                .any(|v| v.kind == ViolationKind::MissingSectionAnnotation),
            "missing annotation should be reported"
        );
    }

    #[test]
    fn test_duplicate_section_reported() {
        let doc = make_doc(vec![
            section_no_block("Overview", "overview", "markdown"),
            section_no_block("Overview", "overview", "markdown"),
        ]);
        let violations = check(&doc);
        assert!(
            violations
                .iter()
                .any(|v| v.kind == ViolationKind::DuplicateSection),
            "duplicate heading should be reported"
        );
    }

    // REQ: change-spec.md#NAP2 — bare `N/A` body exempts a code-requiring
    // section from format_priority_violation.
    #[test]
    fn test_na_body_exempts_format_priority_violation() {
        let mut section = section_no_block("Schema", "schema", "yaml");
        section.body = "N/A".to_string();
        let doc = make_doc(vec![section]);
        assert!(
            check(&doc).is_empty(),
            "N/A body should exempt schema from format_priority_violation"
        );
    }

    #[test]
    fn test_na_with_trailing_whitespace_exempts() {
        let mut section = section_no_block("Schema", "schema", "yaml");
        section.body = "  N/A  ".to_string();
        let doc = make_doc(vec![section]);
        assert!(
            check(&doc).is_empty(),
            "N/A body with surrounding whitespace should still exempt"
        );
    }

    #[test]
    fn test_body_containing_na_prose_is_not_exempt() {
        let mut section = section_no_block("Schema", "schema", "yaml");
        section.body = "N/A because reasons".to_string();
        let doc = make_doc(vec![section]);
        assert!(
            check(&doc)
                .iter()
                .any(|v| v.kind == ViolationKind::FormatPriorityViolation),
            "Only bare N/A sentinel should be exempt; prose containing N/A must still fail"
        );
    }
}
// CODEGEN-END
