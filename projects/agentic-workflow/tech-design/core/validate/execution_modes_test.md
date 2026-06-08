---
id: projects-sdd-tests-execution-modes-test-rs
fill_sections: [overview, tests, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Execution Modes Legacy Hook Retirement Tests

## Overview
<!-- type: overview lang: markdown -->

Codegenerated integration tests for the retired Claude agent bash hook layout.
Phase 2 moved SDD execution to mainthread-owned apply/validate routing, so the
old `agents/_shared/pretooluse-*.sh` hooks must not be required for the test
suite. The file is generated through the Rust tests template using raw Rust
preamble and test bodies; those fields are generator template data, not new
section types.

## Tests
<!-- type: tests lang: yaml -->

```yaml
preamble: |
  //! Integration tests for retired legacy agent execution hooks
  //!
  //! Covers:
  //! - R5: legacy Claude agent bash hooks are retired
  //! - R6: stale mainthread Claude hooks are retired
  //!
  //! R4 (multi_claude_agents agent frontmatter) tests were retired when
  //! Phase 2 mainthread-only execution deleted `.claude/agents/score-*.md`.
  //! The mainthread now drives every `--apply` step directly; there are no
  //! `score-change-*` / `score-review` agent definitions to validate.

  use std::path::PathBuf;

  /// Project root (2 levels up from the crate directory)
  fn project_root() -> PathBuf {
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
  }
imports: []
tests:
  - name: test_legacy_agent_pretooluse_hooks_are_not_required
    body: |
      // R5: retired Claude agent hook paths must not be required by current SDD tests.
      let root = project_root();
      assert!(
          !root
              .join(".claude/hooks/agents/_shared/pretooluse-readonly-bash.sh")
              .exists(),
          "legacy readonly agent hook should stay retired"
      );
      assert!(
          !root
              .join(".claude/hooks/agents/_shared/pretooluse-safe-bash.sh")
              .exists(),
          "legacy safe agent hook should stay retired"
      );
  - name: test_mainthread_hooks_are_not_required
    body: |
      // R6: stale Claude framework hooks should not be required by current AW execution.
      let root = project_root();
      for hook in [
          ".claude/hooks/hook1-post-apply-validate.sh",
          ".claude/hooks/hook2-pre-apply-guard.sh",
          ".claude/hooks/hook5-session-start-idle.sh",
      ] {
          assert!(!root.join(hook).exists(), "{hook} should stay retired");
      }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/execution_modes_test.rs
    action: modify
    section: tests
    impl_mode: codegen
    description: |
      Generate the complete execution-modes integration test file from the
      Tests section. The target file contains only the CODEGEN block for this
      section.
```
