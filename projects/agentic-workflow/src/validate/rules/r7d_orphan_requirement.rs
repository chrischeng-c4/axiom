// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7d_orphan_requirement-rs.md#source
// CODEGEN-BEGIN
//! R7d — every `R{N}` defined in a spec's Requirements table must be
//! referenced by at least one Scenarios body or Unit Test `Covers` cell.
//!
//! Ports `spec_alignment::requirement_coverage::check_with_content` into the
//! per-file `Rule` trait. Only fires for spec files that actually have a
//! Requirements table; specs without one are silently skipped.

use crate::spec_alignment::{parser, requirement_coverage};
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use std::path::Path;

#[derive(Debug, Default, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7d_orphan_requirement-rs.md#source
pub struct OrphanRequirementRule {}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7d_orphan_requirement-rs.md#source
impl Rule for OrphanRequirementRule {
    fn id(&self) -> RuleId {
        RuleId::OrphanRequirement
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        let path_str = spec_path.to_string_lossy();
        let doc = parser::parse(&path_str, content);
        let (violations, _) = requirement_coverage::check_with_content(&doc, content);
        for v in violations {
            let mut f = Finding::error(RuleId::OrphanRequirement, spec_path, v.message.clone());
            if let Some(line) = v.line {
                f = f.with_line(line);
            }
            if let Some(name) = v.name {
                f = f.with_path(name);
            }
            report.push(f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        OrphanRequirementRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn covered_requirement_passes() {
        let spec = r#"## Requirements

| ID | Description |
|----|-------------|
| R1 | foo |

## Scenarios

R1 covered here.
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn orphan_requirement_flagged() {
        let spec = r#"## Requirements

| ID | Description |
|----|-------------|
| R1 | foo |
| R2 | bar |

## Scenarios

R1 mentioned only.
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].rule, RuleId::OrphanRequirement);
        assert!(r.findings[0].message.contains("R2"));
    }

    #[test]
    fn spec_without_requirements_is_silent() {
        let spec = "## Overview\n\nNo requirements here.\n";
        assert!(run(spec).is_empty());
    }
}

// CODEGEN-END
