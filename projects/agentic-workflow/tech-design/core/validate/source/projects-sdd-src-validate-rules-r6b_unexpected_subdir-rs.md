---
id: projects-sdd-src-validate-rules-r6b_unexpected_subdir-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/rules/r6b_unexpected_subdir.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/rules/r6b_unexpected_subdir.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `UnexpectedSubdirRule` | projects/agentic-workflow/src/validate/rules/r6b_unexpected_subdir.rs | struct | pub | 32 |  |
## Source
<!-- type: source lang: rust -->

````rust
//! R6b — flag spec files inside an unexpected top-level subdirectory of a
//! crate spec root.
//!
//! Allowed top-level subdirs are `interfaces`, `logic`, `config`, `tools`,
//! `skills`, `generate`, `validate`, `semantic`, `specs`, `commands`,
//! `bugs`, and `changes`. Anything else (e.g. `docs`, `notes`) is reported.

use crate::validate::{Finding, Rule, RuleId, RuleReport};
use std::path::Path;

use super::r6a_loose_root_file::{is_codegen_fixture_spec, locate_in_crate_spec_root};

const ALLOWED_TOP_DIRS: &[&str] = &[
    "interfaces",
    "logic",
    "config",
    "tools",
    "skills",
    "generate",
    "validate",
    "semantic",
    "specs",
    "commands",
    "bugs",
    "changes",
];

#[derive(Debug, Default, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r6b_unexpected_subdir-rs.md#source
pub struct UnexpectedSubdirRule {}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r6b_unexpected_subdir-rs.md#source
impl Rule for UnexpectedSubdirRule {
    fn id(&self) -> RuleId {
        RuleId::UnexpectedSubdir
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        if is_codegen_fixture_spec(content) {
            return;
        }
        let Some((_, rel)) = locate_in_crate_spec_root(spec_path) else {
            return;
        };
        let parts: Vec<&str> = rel.split('/').collect();
        if parts.len() < 2 {
            return; // single-component is R6a's territory
        }
        let top = parts[0];
        if !ALLOWED_TOP_DIRS.contains(&top) {
            report.push(Finding::error(
                RuleId::UnexpectedSubdir,
                spec_path,
                format!(
                    "file lives under unexpected top-level subdir `{}`; allowed: {}",
                    top,
                    ALLOWED_TOP_DIRS.join(", ")
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(path: &str) -> RuleReport {
        let mut r = RuleReport::new();
        UnexpectedSubdirRule {}.check(&PathBuf::from(path), "", &mut r);
        r
    }

    #[test]
    fn allowed_subdir_is_ok() {
        for dir in &[
            "interfaces",
            "logic",
            "config",
            "tools",
            "skills",
            "generate",
            "validate",
        ] {
            let path = format!("projects/agentic-workflow/tech-design/core/{}/foo.md", dir);
            assert!(run(&path).is_empty(), "{} should be ok", dir);
        }
    }

    #[test]
    fn unexpected_subdir_flagged() {
        let r = run("projects/agentic-workflow/tech-design/core/docs/notes.md");
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].rule, RuleId::UnexpectedSubdir);
        assert!(r.findings[0].message.contains("docs"));
    }

    #[test]
    fn root_level_file_not_handled_here() {
        // R6a covers single-component files; R6b stays silent.
        let r = run("projects/agentic-workflow/tech-design/core/state.md");
        assert!(r.is_empty());
    }

    #[test]
    fn nested_unexpected_dir_flagged_only_at_top_level() {
        // Top-level is allowed (logic/), the nested foo/ is not validated by R6b.
        let r = run("projects/agentic-workflow/tech-design/core/logic/foo/bar.md");
        assert!(r.is_empty());
    }
}
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r6b_unexpected_subdir.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
