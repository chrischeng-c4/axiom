---
id: projects-sdd-src-validate-rules-r7a_missing_section_annotation-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/rules/r7a_missing_section_annotation.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/rules/r7a_missing_section_annotation.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MissingSectionAnnotationRule` | projects/agentic-workflow/src/validate/rules/r7a_missing_section_annotation.rs | struct | pub | 16 |  |
## Source
<!-- type: source lang: rust -->

````rust
//! R7a — every `## Heading` must carry a legacy `<!-- type: X lang: Y -->`
//! or attr-style `<!-- score-section type="X" lang="Y" -->` annotation on
//! the line directly below it.
//!
//! Ports `spec_alignment::format_rules::check_missing_annotations` into the
//! per-file `Rule` interface used by `aw td validate`.

use crate::spec_alignment::parser;
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct MissingSectionAnnotationRule {}

impl Rule for MissingSectionAnnotationRule {
    fn id(&self) -> RuleId {
        RuleId::MissingSectionAnnotation
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        let path_str = spec_path.to_string_lossy();
        let doc = parser::parse(&path_str, content);
        for section in &doc.sections {
            if section.annotation.is_none() {
                report.push(
                    Finding::error(
                        RuleId::MissingSectionAnnotation,
                        spec_path,
                        format!(
                            "section '{}' has no section annotation (`<!-- type: X lang: Y -->` or `<!-- score-section type=\"X\" lang=\"Y\" -->`)",
                            section.heading
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
        MissingSectionAnnotationRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn annotated_section_passes() {
        let spec = "## Overview\n<!-- type: overview lang: markdown -->\n\nbody\n";
        assert!(run(spec).is_empty());
    }

    #[test]
    fn attr_style_annotated_section_passes() {
        let spec = "## Wireframe\n<!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" -->\n\n```yaml\ntasks: []\n```\n";
        assert!(run(spec).is_empty());
    }

    #[test]
    fn unannotated_section_flagged() {
        let spec = "## Bare\n\nbody\n";
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].rule, RuleId::MissingSectionAnnotation);
        assert!(r.findings[0].message.contains("Bare"));
    }

    #[test]
    fn each_unannotated_emits_one_finding() {
        let spec = "## A\n\nx\n## B\n\ny\n";
        assert_eq!(run(spec).findings.len(), 2);
    }

    #[test]
    fn review_h2_under_reviews_is_ignored() {
        let spec = "## Logic\n\
                    <!-- type: logic lang: mermaid -->\n\
                    body\n\
                    \n\
                    # Reviews\n\
                    \n\
                    ## Review 1\n\
                    **Verdict:** approved\n";
        assert!(run(spec).is_empty());
    }
}
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r7a_missing_section_annotation.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
