// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7c_duplicate_section-rs.md#source
// CODEGEN-BEGIN
//! R7c — flag any `## Heading` that appears more than once within the same
//! spec file.
//!
//! Ports `spec_alignment::format_rules::check_duplicate_sections`.

use crate::spec_alignment::parser;
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Default, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7c_duplicate_section-rs.md#source
pub struct DuplicateSectionRule {}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7c_duplicate_section-rs.md#source
impl Rule for DuplicateSectionRule {
    fn id(&self) -> RuleId {
        RuleId::DuplicateSection
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        let path_str = spec_path.to_string_lossy();
        let doc = parser::parse(&path_str, content);

        let mut by_heading: HashMap<String, Vec<usize>> = HashMap::new();
        for section in &doc.sections {
            by_heading
                .entry(section.heading.clone())
                .or_default()
                .push(section.line);
        }

        for (heading, lines) in &by_heading {
            if lines.len() > 1 {
                report.push(
                    Finding::error(
                        RuleId::DuplicateSection,
                        spec_path,
                        format!(
                            "duplicate section heading '{}' at lines {:?}",
                            heading, lines
                        ),
                    )
                    .with_line(lines[0]),
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
        DuplicateSectionRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn unique_headings_pass() {
        let spec = "## A\n<!-- type: overview lang: markdown -->\n\nx\n## B\n<!-- type: overview lang: markdown -->\n\ny\n";
        assert!(run(spec).is_empty());
    }

    #[test]
    fn duplicate_heading_flagged() {
        let spec = "## Same\n\nx\n## Same\n\ny\n";
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].rule, RuleId::DuplicateSection);
        assert!(r.findings[0].message.contains("Same"));
    }

    #[test]
    fn review_heading_matching_section_name_is_ignored() {
        let spec = "## Logic\n\
                    <!-- type: logic lang: mermaid -->\n\
                    body\n\
                    \n\
                    # Reviews\n\
                    \n\
                    ## Logic\n\
                    **Verdict:** approved\n";
        assert!(run(spec).is_empty());
    }
}

// CODEGEN-END
