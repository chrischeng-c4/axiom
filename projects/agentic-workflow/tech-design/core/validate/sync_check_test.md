---
id: projects-sdd-tests-sync-check-test-rs
fill_sections: [overview, tests, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue/backend validation TDs verify AW Core client boundary behavior."
---

# Sync Check Integration Tests

## Overview
<!-- type: overview lang: markdown -->

Codegenerated integration tests for `aw sync --check` semantics. The test
file is emitted through the Rust tests template using raw Rust preamble and
test bodies, keeping the cross-language section type as `tests`.

## Tests
<!-- type: tests lang: yaml -->

```yaml
preamble: |
  //! External integration tests for `aw sync --check` semantics.
  //!
  //! Test T22 from the spec test plan: --check output references config.toml.
  //!
  //! REQ: REQ-009, REQ-011

  use std::fs;
  use tempfile::TempDir;

  use agentic_workflow::services::project_registry::check_drift;
  use agentic_workflow::shared::workspace::{SYNC_BEGIN_MARKER, SYNC_END_MARKER};

  fn make_score_root() -> TempDir {
      let tmp = TempDir::new().unwrap();
      fs::create_dir_all(tmp.path().join(".aw")).unwrap();
      tmp
  }

  fn write_config(root: &std::path::Path, content: &str) {
      fs::write(root.join(".aw").join("config.toml"), content).unwrap();
  }
imports: []
tests:
  - name: check_targets_config_toml
    body: |
      let tmp = make_score_root();

      // config.toml with stale projects inside the marker block
      // (no actual directories on disk, so fresh discovery yields empty)
      let stale_content = format!(
          "{begin}\n\n[[projects]]\nname = \"stale-project\"\npath = \"crates/stale-project\"\n\n[[projects.workspaces]]\nname = \"stale-project\"\npaths = [\"crates/stale-project/**\"]\ntarget = \"rust\"\ntest_cmd = \"cargo test -p stale-project\"\n\n{end}\n",
          begin = SYNC_BEGIN_MARKER,
          end = SYNC_END_MARKER,
      );
      write_config(tmp.path(), &stale_content);

      // R11: check_drift should detect drift
      let diff = check_drift(tmp.path()).unwrap();
      let diff_text = diff.expect("expected drift to be detected when config.toml is out of date");

      // R11: diff output must reference config.toml (not projects.toml)
      assert!(
          diff_text.contains("config.toml"),
          "aw sync --check output must reference config.toml;\ngot:\n{diff_text}"
      );

      // R9: must not reference the old projects.toml
      assert!(
          !diff_text.contains("projects.toml"),
          "aw sync --check output must not reference projects.toml;\ngot:\n{diff_text}"
      );
  - name: check_no_drift_when_up_to_date
    body: |
      let tmp = make_score_root();

      // No projects discovered (no directories), write empty sync block
      agentic_workflow::services::project_registry::write_projects_config(tmp.path(), &[]).unwrap();

      // check_drift with no real projects should report no drift
      let diff = check_drift(tmp.path()).unwrap();
      assert!(
          diff.is_none(),
          "check_drift must return None when config.toml is up to date; got: {:?}",
          diff
      );
  - name: check_does_not_modify_config_toml
    body: |
      let tmp = make_score_root();

      // Stale marker block
      let stale_content = format!(
          "{begin}\n\n[[projects]]\nname = \"ghost\"\npath = \"crates/ghost\"\n\n[[projects.workspaces]]\nname = \"ghost\"\npaths = [\"crates/ghost/**\"]\ntarget = \"rust\"\n\n{end}\n",
          begin = SYNC_BEGIN_MARKER,
          end = SYNC_END_MARKER,
      );
      write_config(tmp.path(), &stale_content);

      let original = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();

      // check_drift must NOT write anything
      check_drift(tmp.path()).unwrap();

      let after = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();
      assert_eq!(original, after, "check_drift must not modify config.toml");
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/sync_check_test.rs
    action: modify
    section: tests
    impl_mode: codegen
    description: |
      Generate the complete sync-check integration test file from the Tests
      section. The target file contains only the CODEGEN block for this
      section.
```
