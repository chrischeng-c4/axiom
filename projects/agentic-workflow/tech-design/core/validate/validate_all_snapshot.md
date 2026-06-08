---
id: projects-sdd-tests-validate-all-snapshot-rs
fill_sections: [overview, tests, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Validate All Snapshot Tests

## Overview
<!-- type: overview lang: markdown -->

Codegenerated integration snapshot tests for the `--all` validation walk over
a curated mini-fixture set. The test file is emitted from the Rust tests
template using a raw Rust preamble and raw test bodies, keeping the section
type at `tests` while language detail stays in the generator template data.

## Tests
<!-- type: tests lang: yaml -->

```yaml
preamble: |
  //! T8 — integration snapshot test for the `--all` batch walk over a
  //! curated mini-fixture set. We don't shell out to the `score` binary
  //! (cyclic dep); instead we exercise the same library entry points that
  //! `validate_spec_structure::run_all()` calls — `resolve_spec_files` plus
  //! `run_rules` — and assert the produced violation lines match the
  //! expected snapshot.
  //!
  //! Fixtures are written to a tempdir so the test is hermetic.
  //!
  //! @spec projects/agentic-workflow/tech-design/core/validate/section-format-rule.md#test-plan

  use std::path::PathBuf;

  fn write(root: &std::path::Path, rel: &str, content: &str) -> PathBuf {
      let p = root.join(rel);
      if let Some(parent) = p.parent() {
          std::fs::create_dir_all(parent).unwrap();
      }
      std::fs::write(&p, content).unwrap();
      p
  }

  const VALID_PROSE: &str = "## Overview\n\
                             <!-- type: overview lang: markdown -->\n\
                             \n\
                             Plain prose body. No fence required for prose types.\n";

  const VALID_STRUCTURAL: &str = "## Schema\n\
                                  <!-- type: schema lang: yaml -->\n\
                                  \n\
                                  ```yaml\n\
                                  foo: bar\n\
                                  ```\n";

  const VALID_MERMAID_PLUS: &str = "## Logic\n\
                                    <!-- type: logic lang: mermaid -->\n\
                                    \n\
                                    ```mermaid\n\
                                    ---\n\
                                    id: my-flow\n\
                                    ---\n\
                                    flowchart TD\n\
                                    a --> b\n\
                                    ```\n";

  const INVALID_STRUCTURAL: &str = "## Schema\n\
                                    <!-- type: schema lang: yaml -->\n\
                                    \n\
                                    prose only — no fence and no placeholder\n";

  const INVALID_MERMAID: &str = "## Logic\n\
                                 <!-- type: logic lang: mermaid -->\n\
                                 \n\
                                 ```mermaid\n\
                                 flowchart TD\n\
                                 a --> b\n\
                                 ```\n";
imports: []
tests:
  - name: validate_all_snapshot_matches_expected_violations
    body: |
      let tmp = tempfile::tempdir().unwrap();
      let root = tmp.path();

      write(root, "valid_prose.md", VALID_PROSE);
      write(root, "valid_structural.md", VALID_STRUCTURAL);
      write(root, "valid_mermaid_plus.md", VALID_MERMAID_PLUS);
      write(root, "invalid_structural.md", INVALID_STRUCTURAL);
      write(root, "invalid_mermaid.md", INVALID_MERMAID);

      let shape = agentic_workflow::validate::PathShape::Prefix(root.to_path_buf());
      let files = agentic_workflow::validate::resolve_spec_files(&shape).unwrap();
      assert_eq!(files.len(), 5, "expected 5 fixture files");

      let report = agentic_workflow::validate::run_rules(&files);

      // Filter to just SectionFormat findings — other rules (R3a..R3g) may
      // also fire on these tiny fixtures, but T8 covers the section-format
      // rule's snapshot.
      let mut sf_lines: Vec<String> = report
          .findings
          .iter()
          .filter(|f| matches!(f.rule, agentic_workflow::validate::RuleId::SectionFormat))
          .map(|f| {
              let name = f
                  .file
                  .file_name()
                  .and_then(|n| n.to_str())
                  .unwrap_or("?")
                  .to_string();
              format!("{}:{}: [{}]", name, f.line.unwrap_or(0), f.rule.short())
          })
          .collect();
      sf_lines.sort();

      let expected = vec![
          "invalid_mermaid.md:2: [R3h:section-format]".to_string(),
          "invalid_structural.md:2: [R3h:section-format]".to_string(),
      ];
      assert_eq!(
          sf_lines, expected,
          "section-format violation snapshot mismatch"
      );
  - name: validate_all_emits_zero_findings_on_clean_input
    body: |
      let tmp = tempfile::tempdir().unwrap();
      let root = tmp.path();
      write(root, "a.md", VALID_PROSE);
      write(root, "b.md", VALID_STRUCTURAL);
      write(root, "c.md", VALID_MERMAID_PLUS);

      let shape = agentic_workflow::validate::PathShape::Prefix(root.to_path_buf());
      let files = agentic_workflow::validate::resolve_spec_files(&shape).unwrap();
      let report = agentic_workflow::validate::run_rules(&files);

      let sf_count = report
          .findings
          .iter()
          .filter(|f| matches!(f.rule, agentic_workflow::validate::RuleId::SectionFormat))
          .count();
      assert_eq!(
          sf_count, 0,
          "expected zero SectionFormat findings on clean fixture set; \
           findings: {:?}",
          report.findings,
      );
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/validate_all_snapshot.rs
    action: modify
    section: tests
    impl_mode: codegen
    description: |
      Generate the complete validate-all snapshot test file from the Tests
      section. The target file contains only the CODEGEN block for this
      section.
```
