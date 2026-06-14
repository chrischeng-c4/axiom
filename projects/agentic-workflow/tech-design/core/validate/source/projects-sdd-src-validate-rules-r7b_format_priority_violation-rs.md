---
id: projects-sdd-src-validate-rules-r7b_format_priority_violation-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/rules/r7b_format_priority_violation.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/rules/r7b_format_priority_violation.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FormatPriorityViolationRule` | projects/agentic-workflow/src/validate/rules/r7b_format_priority_violation.rs | struct | pub | 47 |  |
## Source
<!-- type: source lang: rust -->

````rust
//! R7b — sections whose annotated type requires a specific code-fence lang
//! must contain a matching fenced block (or the bare `N/A` sentinel).
//!
//! Ports `spec_alignment::format_rules::check_format_priority`. The required
//! mapping is local to this rule rather than re-exported from the legacy
//! module, so future spec types can be added without re-exporting through
//! the legacy module.

use crate::spec_alignment::parser;
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use std::path::Path;

const REQUIRED: &[(&str, &str)] = &[
    ("config", "yaml"),
    ("logic", "mermaid"),
    ("rpc-api", "yaml"),
    ("state-machine", "mermaid"),
    ("cli", "yaml"),
    ("changes", "yaml"),
    ("schema", "yaml"),
    ("rest-api", "yaml"),
    ("async-api", "yaml"),
    ("db-model", "mermaid"),
    ("dependency", "mermaid"),
    ("interaction", "mermaid"),
    ("wireframe", "yaml"),
    ("component", "yaml"),
    ("design-token", "yaml"),
    ("manifest", "yaml"),
    ("runtime-image", "yaml"),
    ("deployment", "yaml"),
    ("mindmap", "mermaid"),
    ("requirements", "mermaid"),
    ("unit-test", "mermaid"),
    ("e2e-test", "yaml"),
    ("rust-source-unit", "rust"),
    ("test-plan", "mermaid"),
    ("tests", "yaml"),
    ("scenarios", "yaml"),
];

const PROSE_ONLY: &[&str] = &["overview", "doc"];

#[derive(Debug, Default, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7b_format_priority_violation-rs.md#source
pub struct FormatPriorityViolationRule {}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7b_format_priority_violation-rs.md#source
impl Rule for FormatPriorityViolationRule {
    fn id(&self) -> RuleId {
        RuleId::FormatPriorityViolation
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        let path_str = spec_path.to_string_lossy();
        let doc = parser::parse(&path_str, content);
        for section in &doc.sections {
            let Some(annotation) = &section.annotation else {
                continue; // R7a's territory
            };
            let stype = annotation.section_type.as_str();
            if PROSE_ONLY.contains(&stype) {
                continue;
            }
            if section.body.trim() == "N/A" {
                continue; // NAP2 — explicit not-applicable sentinel
            }
            let Some(required_lang) = REQUIRED.iter().find(|(t, _)| *t == stype).map(|(_, l)| *l)
            else {
                continue; // unknown section type — no rule
            };
            let has_match = section
                .code_blocks
                .iter()
                .any(|cb| cb.lang == required_lang);
            if !has_match {
                report.push(
                    Finding::error(
                        RuleId::FormatPriorityViolation,
                        spec_path,
                        format!(
                            "section '{}' (type: {}) requires a ```{} fenced block",
                            section.heading, stype, required_lang
                        ),
                    )
                    .with_line(section.line),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        FormatPriorityViolationRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn schema_with_yaml_block_passes() {
        let spec = "## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\nfoo: 1\n```\n";
        assert!(run(spec).is_empty());
    }

    #[test]
    fn schema_without_block_flagged() {
        let spec = "## Schema\n<!-- type: schema lang: yaml -->\n\nprose only\n";
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].rule, RuleId::FormatPriorityViolation);
    }

    #[test]
    fn all_active_non_prose_section_types_have_format_rules() {
        use crate::models::spec_rules::SectionType;

        let missing: Vec<_> = SectionType::all_in_fill_order()
            .into_iter()
            .map(|section_type| section_type.as_str())
            .filter(|section_type| !PROSE_ONLY.contains(section_type))
            .filter(|section_type| {
                !REQUIRED
                    .iter()
                    .any(|(required_section, _)| required_section == section_type)
            })
            .collect();
        assert!(
            missing.is_empty(),
            "section types without validator format rules: {missing:?}"
        );
    }

    #[test]
    fn na_sentinel_exempts_section() {
        let spec = "## Schema\n<!-- type: schema lang: yaml -->\n\nN/A\n";
        assert!(run(spec).is_empty());
    }

    #[test]
    fn overview_prose_only_no_block_required() {
        let spec = "## Overview\n<!-- type: overview lang: markdown -->\n\nprose\n";
        assert!(run(spec).is_empty());
    }

    #[test]
    fn unannotated_section_skipped() {
        // R7a's job, not ours.
        let spec = "## No Annotation\n\nprose\n";
        assert!(run(spec).is_empty());
    }
}
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r7b_format_priority_violation.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
