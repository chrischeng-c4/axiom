---
id: projects-sdd-tests-project-registry-test-rs
fill_sections: [overview, tests, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue/backend validation TDs verify AW Core client boundary behavior."
---

# Project Registry Integration Tests

## Overview
<!-- type: overview lang: markdown -->

Codegenerated integration tests for project registry config marker upsert,
idempotent round trips, workspace codegen profile loading, stale
`.aw/projects.toml` migration, and drift output references. Test-level
REQ comments are emitted through the Rust tests template `leading` field.

## Tests
<!-- type: tests lang: yaml -->

```yaml
preamble: |
  //! External integration tests for project_registry.
  //!
  //! Tests T17, T18, T21 from the spec test plan.
  //!
  //! REQ: REQ-001, REQ-002, REQ-003, REQ-005, REQ-006, REQ-009, REQ-010
  
  use std::fs;
  use std::path::PathBuf;
  use tempfile::TempDir;
  
  use agentic_workflow::models::project::{Project, Workspace};
  use agentic_workflow::models::tech_stack::Language;
  use agentic_workflow::services::project_registry::{check_drift, load_projects, write_projects_config};
  use agentic_workflow::shared::workspace::{SYNC_BEGIN_MARKER, SYNC_END_MARKER};
  
  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------
  
  fn make_score_root() -> TempDir {
      let tmp = TempDir::new().unwrap();
      fs::create_dir_all(tmp.path().join(".aw")).unwrap();
      tmp
  }
  
  fn make_project(name: &str, target: Language, test_cmd: Option<&str>) -> Project {
      Project {
          name: name.to_string(),
          path: PathBuf::from(format!("crates/{}", name)),
          tech_design_dir: None,
          workspaces: vec![Workspace {
              name: Some(name.to_string()),
              paths: vec![format!("crates/{}/**", name)],
              target,
              test_cmd: test_cmd.map(|s| s.to_string()),
              codegen: None,
          }],
      }
  }
  
  fn write_config(root: &std::path::Path, content: &str) {
      fs::write(root.join(".aw").join("config.toml"), content).unwrap();
  }
imports: []
tests:
  - name: marker_upsert_first_run
    leading: |
      // ---------------------------------------------------------------------------
      // T17: marker_upsert_first_run
      // Verifies R1, R2, R3
      // ---------------------------------------------------------------------------
      
      // REQ: REQ-001
      // REQ: REQ-002
      // REQ: REQ-003
    indent_body: false
    body: |
      let tmp = make_score_root();
      
      // Existing user config — no markers yet
      write_config(
          tmp.path(),
          "# user comment\n[agentic_workflow.test.scope]\nroots = [\"crates\", \"projects\"]\n",
      );
      
      let projects = vec![make_project(
          "sdd",
          Language::Rust,
          Some("cargo test -p agentic-workflow"),
      )];
      write_projects_config(tmp.path(), &projects).unwrap();
      
      let content = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();
      
      // R2: BEGIN marker must be present
      assert!(
          content.contains(SYNC_BEGIN_MARKER),
          "config.toml must contain BEGIN AW SYNC marker after first sync;\ngot:\n{content}"
      );
      
      // R2: END marker must be present
      assert!(
          content.contains(SYNC_END_MARKER),
          "config.toml must contain END AW SYNC marker after first sync;\ngot:\n{content}"
      );
      
      // R1: [[projects]] entries present
      assert!(
          content.contains("[[projects]]"),
          "config.toml must contain [[projects]] entries;\ngot:\n{content}"
      );
      assert!(
          content.contains("\"sdd\""),
          "config.toml must contain discovered project name;\ngot:\n{content}"
      );
      
      // R3: user-authored content untouched
      assert!(
          content.contains("# user comment"),
          "user comment must survive sync;\ngot:\n{content}"
      );
      assert!(
          content.contains("[agentic_workflow.test.scope]"),
          "user section must survive sync;\ngot:\n{content}"
      );
      assert!(
          content.contains("roots = [\"crates\", \"projects\"]"),
          "user content must be byte-identical after sync;\ngot:\n{content}"
      );
  - name: marker_upsert_round_trip
    leading: |
      // ---------------------------------------------------------------------------
      // T18: marker_upsert_round_trip (idempotency + R3 preservation + R6 enumeration)
      // Verifies R3, R5, R6
      // ---------------------------------------------------------------------------
      
      // REQ: REQ-003
      // REQ: REQ-005
      // REQ: REQ-006
    indent_body: false
    body: |
      let tmp = make_score_root();
      
      // User content with formatting details that must survive round-trips
      write_config(
          tmp.path(),
          "# top comment\n[agentic_workflow.test.scope]\nroots = [\"crates\", \"projects\", \"packages\"]\n\n[defaults.workspace]\ncodegen.target = \"rust\"\n",
      );
      
      let projects = vec![
          make_project("alpha", Language::Rust, Some("cargo test -p alpha")),
          make_project(
              "beta",
              Language::Python,
              Some("cd projects/beta && uv run pytest"),
          ),
          make_project("gamma", Language::TypeScript, None),
      ];
      
      // First sync
      write_projects_config(tmp.path(), &projects).unwrap();
      let after_first = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();
      
      // Second sync with identical input (idempotency, R5)
      write_projects_config(tmp.path(), &projects).unwrap();
      let after_second = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();
      
      // R5: idempotency — diff must be empty
      assert_eq!(
          after_first, after_second,
          "double sync with identical input must produce zero diff (R5)"
      );
      
      // R3: non-generated sections preserved byte-identical
      assert!(
          after_second.contains("# top comment"),
          "top comment must be preserved;\ngot:\n{after_second}"
      );
      assert!(
          after_second.contains("[agentic_workflow.test.scope]"),
          "sdd section must be preserved;\ngot:\n{after_second}"
      );
      assert!(
          after_second.contains("[defaults.workspace]"),
          "defaults section must be preserved;\ngot:\n{after_second}"
      );
      
      // R6: full enumeration — all projects round-trip through load_projects
      let loaded = load_projects(tmp.path()).unwrap();
      assert_eq!(
          loaded.len(),
          projects.len(),
          "R6: all {} projects must be written and loadable; got {}",
          projects.len(),
          loaded.len()
      );
      let loaded_names: Vec<&str> = loaded.iter().map(|p| p.name.as_str()).collect();
      for p in &projects {
          assert!(
              loaded_names.contains(&p.name.as_str()),
              "R6: project '{}' must be present after sync; got: {:?}",
              p.name,
              loaded_names
          );
      }
  - name: load_projects_reads_workspace_codegen_profile
    indent_body: false
    body: |
      let tmp = make_score_root();
      write_config(
          tmp.path(),
          r#"
      [[projects]]
      name = "agentic-workflow"
      path = "projects/agentic-workflow"
      td_path = "projects/agentic-workflow/tech-design/core"
      label = "project:agentic-workflow"
      
      [[projects.workspaces]]
      paths = ["projects/agentic-workflow/**"]
      target = "rust"
      test_cmd = "cargo test -p agentic-workflow"
      codegen.profile = "rust/score-crate"
      "#,
      );
      
      let loaded = load_projects(tmp.path()).unwrap();
      let workspace = &loaded[0].workspaces[0];
      let codegen = workspace.codegen.as_ref().expect("codegen profile");
      
      assert_eq!(codegen.profile.as_deref(), Some("rust/score-crate"));
      assert_eq!(codegen.target, None);
      assert_eq!(workspace.target, Language::Rust);
  - name: migration_deletes_projects_toml
    leading: |
      // ---------------------------------------------------------------------------
      // T21: migration_deletes_projects_toml
      // Verifies R10
      // ---------------------------------------------------------------------------
      
      // REQ: REQ-010
    indent_body: false
    body: |
      let tmp = make_score_root();
      
      // Stale projects.toml from the old format
      let stale_path = tmp.path().join(".aw").join("projects.toml");
      fs::write(
          &stale_path,
          "# Auto-generated by `aw sync`\n[[projects]]\nname = \"old-entry\"\npath = \"crates/old-entry\"\n",
      )
      .unwrap();
      assert!(
          stale_path.exists(),
          "stale projects.toml must exist before migration sync"
      );
      
      let projects = vec![make_project(
          "new-entry",
          Language::Rust,
          Some("cargo test -p new-entry"),
      )];
      write_projects_config(tmp.path(), &projects).unwrap();
      
      // R10: stale file must be deleted
      assert!(
          !stale_path.exists(),
          ".aw/projects.toml must be deleted after successful sync (R10 migration)"
      );
      
      // New data is in config.toml
      let loaded = load_projects(tmp.path()).unwrap();
      assert_eq!(loaded.len(), 1);
      assert_eq!(loaded[0].name, "new-entry");
  - name: check_drift_references_config_toml_in_output
    leading: |
      // ---------------------------------------------------------------------------
      // Additional: check_drift targets config.toml (T22 prerequisite)
      // Verifies R9, R11
      // ---------------------------------------------------------------------------
      
      // REQ: REQ-009
      // REQ: REQ-011
    indent_body: false
    body: |
      let tmp = make_score_root();
      
      // Config with stale ghost project in the marker block
      let stale_block = format!(
          "{}\n\n[[projects]]\nname = \"ghost\"\npath = \"crates/ghost\"\n\n[[projects.workspaces]]\nname = \"ghost\"\npaths = [\"crates/ghost/**\"]\ntarget = \"rust\"\n\n{}\n",
          SYNC_BEGIN_MARKER,
          SYNC_END_MARKER
      );
      write_config(tmp.path(), &stale_block);
      
      let drift = check_drift(tmp.path()).unwrap().expect("expected drift");
      
      // R9/R11: diff output must reference config.toml (not projects.toml)
      assert!(
          drift.contains("config.toml"),
          "--check output must reference config.toml, not projects.toml;\ngot:\n{drift}"
      );
      assert!(
          !drift.contains("projects.toml"),
          "--check output must not reference projects.toml;\ngot:\n{drift}"
      );
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/project_registry_test.rs
    action: modify
    section: tests
    impl_mode: codegen
    generator: rust.tests
    description: |
      Emit project registry integration tests from the Rust tests template,
      preserving test-level REQ comments via the leading field.
```
