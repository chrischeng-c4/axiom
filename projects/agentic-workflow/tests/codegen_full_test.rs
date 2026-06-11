// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/codegen_full_test.md#tests
// CODEGEN-BEGIN

//! Integration tests: run codegen apply across all tech design specs.
//!
//! Ensures the apply pipeline never panics on any spec file and that
//! codegen-ready specs produce meaningful output.

use agentic_workflow::generate::apply::{run_apply, ApplyReport};
use std::path::{Path, PathBuf};

const PYTHON_MODELS_SPEC: &str = r#"---
id: python-backend-video-qc-api-models
fixture_for: python-backend-emitter
type: codegen-fixture
project: fixture_platform
priority: p2
summary: Captured fixture for the Python pydantic-model emitter.
---

## Models
<!-- type: schema lang: yaml -->

```yaml
section_type: schema
spec_id: python-backend-video-qc-api-models
module_docstring: |
  Video QC API models.

  Spec: cclab/specs/backend/workspace/bases/README.md
imports:
  - { module: beanie,   names: [] }
  - { module: pydantic, names: [] }
  - { module: src,      names: [bases] }
pydantic_models:
  - name: VideoQCAPIRequestModel
    base: bases.BaseAPIRequestModel
    docstring: |
      VideoQCAPIRequestModel.

      Spec: cclab/specs/backend/workspace/bases/README.md

      <!-- spec-ref --> Spec: cclab/specs/backend/workspace/bases/README.md <!-- /spec-ref -->
    fields:
      - { name: video_url, py_type: "pydantic.HttpUrl" }
  - name: VideoQCPredictAPIRequestModel
    base: bases.BaseAPIResponseModel
    docstring: |
      VideoQCPredictAPIRequestModel.

      Spec: cclab/specs/backend/workspace/bases/README.md

      <!-- spec-ref --> Spec: cclab/specs/backend/workspace/bases/README.md <!-- /spec-ref -->
    fields:
      - { name: video_url, py_type: "pydantic.HttpUrl" }
      - { name: blob_id,   py_type: "beanie.PydanticObjectId" }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: fixtures/python_backend/src/features/automations/video_qc/api_models.py
    action: modify
    section: models
    description: Regenerate Video QC API pydantic models from the Python backend emitter.
```
"#;

const PYTHON_MODELS_EXPECTED: &str = r#"# SPEC-MANAGED: fixtures/python_backend/tech_design/api_models.md#models
# CODEGEN-BEGIN
"""Video QC API models.

Spec: cclab/specs/backend/workspace/bases/README.md
"""
import beanie
import pydantic

from src import bases


class VideoQCAPIRequestModel(bases.BaseAPIRequestModel):
    """VideoQCAPIRequestModel.

    Spec: cclab/specs/backend/workspace/bases/README.md

    <!-- spec-ref --> Spec: cclab/specs/backend/workspace/bases/README.md <!-- /spec-ref -->
    """
    video_url: pydantic.HttpUrl


class VideoQCPredictAPIRequestModel(bases.BaseAPIResponseModel):
    """VideoQCPredictAPIRequestModel.

    Spec: cclab/specs/backend/workspace/bases/README.md

    <!-- spec-ref --> Spec: cclab/specs/backend/workspace/bases/README.md <!-- /spec-ref -->
    """
    video_url: pydantic.HttpUrl
    blob_id: beanie.PydanticObjectId
# CODEGEN-END
"#;

/// Derive project root from CARGO_MANIFEST_DIR (projects/agentic-workflow/ → project root).
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to resolve project root")
}

/// Walk all .md files under a directory recursively.
fn walk_md_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return files;
    }
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().map_or(false, |ext| ext == "md") {
            files.push(entry.path().to_path_buf());
        }
    }
    files
}

/// Check if a spec file has a Changes section with file entries.
fn has_changes_section(content: &str) -> bool {
    let mut in_changes = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") && trimmed.to_lowercase().contains("change") {
            in_changes = true;
            continue;
        }
        if in_changes && trimmed.starts_with("## ") {
            break;
        }
        if in_changes && (trimmed.starts_with("- path:") || trimmed.starts_with("- file:")) {
            return true;
        }
    }
    false
}

/// Run apply in dry_run mode, returning Ok(report) or Err(message).
fn try_apply(spec_path: &Path, tmpdir: &Path) -> Result<ApplyReport, String> {
    std::panic::catch_unwind(|| run_apply(spec_path, tmpdir, true))
        .map_err(|_| "panicked".to_string())?
        .map_err(|e| format!("{}", e))
}

#[test]
fn test_all_specs_no_panic() {
    let root = project_root();
    let td_dir = root.join(".aw/tech-design");
    let specs = walk_md_files(&td_dir);
    assert!(
        !specs.is_empty(),
        "Should find spec files under .aw/tech-design/"
    );

    let tmpdir = tempfile::TempDir::new().unwrap();
    let mut total = 0;
    let mut with_changes = 0;
    let mut panicked = Vec::new();
    let mut errored = Vec::new();

    for spec in &specs {
        total += 1;
        let content = match std::fs::read_to_string(spec) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if !has_changes_section(&content) {
            continue;
        }
        with_changes += 1;

        match try_apply(spec, tmpdir.path()) {
            Ok(_) => {}
            Err(msg) if msg == "panicked" => {
                panicked.push(
                    spec.strip_prefix(&root)
                        .unwrap_or(spec)
                        .display()
                        .to_string(),
                );
            }
            Err(msg) => {
                errored.push(format!(
                    "{}: {}",
                    spec.strip_prefix(&root).unwrap_or(spec).display(),
                    msg
                ));
            }
        }
    }

    eprintln!(
        "codegen_full_test: {total} specs scanned, {with_changes} with Changes, {} panicked, {} errored",
        panicked.len(),
        errored.len(),
    );

    if !panicked.is_empty() {
        panic!(
            "Apply pipeline panicked on {} specs:\n  {}",
            panicked.len(),
            panicked.join("\n  ")
        );
    }

    // Errors are tolerable (e.g., missing files in dry_run) — just log them
    if !errored.is_empty() {
        eprintln!("Non-fatal errors ({}):", errored.len());
        for e in &errored {
            eprintln!("  {e}");
        }
    }
}

#[test]
fn test_gentest_produces_all_files() {
    let root = project_root();
    let spec = root.join(".aw/tech-design/test/gentest.md");
    if !spec.exists() {
        eprintln!("SKIP: gentest.md not found (test worktree may not exist)");
        return;
    }

    let tmpdir = tempfile::TempDir::new().unwrap();
    let report = run_apply(&spec, tmpdir.path(), false).expect("gentest apply should succeed");

    assert_eq!(report.files.len(), 3, "gentest should produce 3 files");
    assert_eq!(report.files_created(), 3, "all 3 files should be new");
    assert!(
        report.total_blocks_updated() >= 3,
        "should have at least 3 codegen blocks"
    );

    // Verify each file exists and has CODEGEN markers
    for f in &report.files {
        let path = tmpdir.path().join(&f.path);
        assert!(
            path.exists(),
            "generated file should exist: {}",
            f.path.display()
        );

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("CODEGEN-BEGIN"),
            "{}: should have CODEGEN-BEGIN",
            f.path.display()
        );
        assert!(
            content.contains("CODEGEN-END"),
            "{}: should have CODEGEN-END",
            f.path.display()
        );
    }

    // Verify specific section markers
    let state_content = std::fs::read_to_string(tmpdir.path().join("src/order/state.rs")).unwrap();
    assert!(
        state_content.contains("#state-machine"),
        "state.rs should reference state-machine section"
    );

    let pricing_content =
        std::fs::read_to_string(tmpdir.path().join("src/order/pricing.rs")).unwrap();
    assert!(
        pricing_content.contains("#logic"),
        "pricing.rs should reference logic section"
    );
    assert!(
        pricing_content.contains("fn calculate_price"),
        "pricing.rs should have the entry function"
    );

    let api_content = std::fs::read_to_string(tmpdir.path().join("src/order/api.rs")).unwrap();
    assert!(
        api_content.contains("#interaction"),
        "api.rs should reference interaction section"
    );
}

#[test]
fn test_task_state_machine_roundtrip() {
    let root = project_root();
    let spec = root.join(".aw/tech-design/crates/cclab-queue/logic/task-state-machine.md");
    if !spec.exists() {
        eprintln!("SKIP: task-state-machine.md not found");
        return;
    }

    let tmpdir = tempfile::TempDir::new().unwrap();
    let report =
        run_apply(&spec, tmpdir.path(), false).expect("task-state-machine apply should succeed");

    assert_eq!(
        report.files.len(),
        3,
        "should produce 3 file entries (state-machine + schema + test-plan)"
    );
    let generated = tmpdir.path().join("crates/cclab-queue/src/state.rs");
    assert!(generated.exists(), "generated state.rs should exist");

    let content = std::fs::read_to_string(&generated).unwrap();
    eprintln!("---- GENERATED state.rs ----");
    eprintln!("{}", content);
    eprintln!("---- END GENERATED ----");

    // Basic smoke-level assertions — true diff is inspected manually.
    assert!(content.contains("CODEGEN-BEGIN"));
    assert!(content.contains("CODEGEN-END"));
    // state-machine: enum + derives + can_transition_to
    assert!(
        content.contains("enum TaskState"),
        "should emit TaskState enum name via type_name override"
    );
    assert!(
        content.contains("Copy"),
        "derives should include Copy from x-rust"
    );
    assert!(
        content.contains("Default"),
        "derives should include Default from x-rust"
    );
    assert!(
        content.contains("SCREAMING_SNAKE_CASE"),
        "serde rename from x-rust"
    );
    assert!(
        content.contains("fn can_transition_to"),
        "edges should emit can_transition_to()"
    );
    // schema: TaskResult struct
    assert!(
        content.contains("struct TaskResult"),
        "schema definitions should produce TaskResult struct"
    );
    // test-plan: 39 stubs
    assert!(
        content.contains("fn default_is_pending"),
        "test-plan table should emit first test"
    );
    assert!(
        content.contains("fn task_result_is_clone"),
        "test-plan table should emit last test"
    );
}

#[test]
#[ignore]
fn dump_task_state_machine_to_tmp() {
    let root = project_root();
    let spec = root.join(".aw/tech-design/crates/cclab-queue/logic/task-state-machine.md");
    let out_dir = std::path::Path::new("/tmp/aw-roundtrip");
    // Clean prior run
    let _ = std::fs::remove_dir_all(out_dir);
    std::fs::create_dir_all(out_dir).unwrap();

    let report = run_apply(&spec, out_dir, false).expect("apply failed");
    eprintln!("files: {}", report.files.len());
    for f in &report.files {
        eprintln!(
            "  {} — blocks updated: {}",
            f.path.display(),
            f.blocks_updated
        );
    }
    eprintln!(
        "\nGenerated → {}/crates/cclab-queue/src/state.rs",
        out_dir.display()
    );
}

#[test]
fn test_minimal_frontmatter_skips_gracefully() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let spec_content = r#"---
id: test-minimal
fill_sections: [overview, state-machine, changes]
---

## Overview
<!-- type: overview lang: markdown -->

Test spec with minimal Mermaid Plus frontmatter.

## State Machine
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: minimal-sm
initial: idle
---
stateDiagram-v2
    [*] --> idle
    idle --> done: finish
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: src/minimal.rs
    action: create
    section: state-machine
```
"#;
    let spec_path = tmpdir.path().join("minimal_test.md");
    std::fs::write(&spec_path, spec_content).unwrap();

    let out_dir = tempfile::TempDir::new().unwrap();
    let report =
        run_apply(&spec_path, out_dir.path(), false).expect("minimal frontmatter should not crash");

    // Should produce a file but with fallback marker-only output
    // (no nodes → state-machine generator skips → falls through to marker)
    assert_eq!(report.files.len(), 1, "should still produce 1 file entry");
}

#[test]
fn test_rpc_api_and_config_dispatchers() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let spec_content = r#"---
id: test-rpc-and-config
fill_sections: [rpc-api, config, changes]
---

## RPC API
<!-- type: rpc-api lang: yaml -->

```yaml
openrpc: "1.3.2"
info:
  title: TestApi
  version: "0.1.0"
methods:
  - name: ping
    summary: Health check
    params: []
    result:
      name: pong
      schema:
        type: string
  - name: add_item
    summary: Add an item
    params:
      - name: item_id
        schema:
          type: string
      - name: qty
        schema:
          type: integer
    result:
      name: ok
      schema:
        type: boolean
```

## Config
<!-- type: config lang: yaml -->

```yaml
title: ServerConfig
type: object
properties:
  port:
    type: integer
    default: 8080
    description: Listen port
  host:
    type: string
    default: "127.0.0.1"
    description: Bind address
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/cclab-sdd/src/test_rpc.rs
    action: create
    section: rpc-api
  - path: crates/cclab-sdd/src/test_config.rs
    action: create
    section: config
```
"#;
    // Spec lives in a repo root that contains a Cargo.toml so the
    // tech-stack gate sees a Rust workspace.
    let spec_path = tmpdir.path().join("rpc_config_test.md");
    std::fs::write(&spec_path, spec_content).unwrap();

    let out_dir = tempfile::TempDir::new().unwrap();
    // Seed a root Cargo.toml so check_workspace_language detects Rust.
    std::fs::write(
        out_dir.path().join("Cargo.toml"),
        "[workspace]\nmembers = []\n",
    )
    .unwrap();

    let report = run_apply(&spec_path, out_dir.path(), false)
        .expect("rpc-api + config apply should succeed");

    assert_eq!(report.files.len(), 2, "should produce 2 file entries");

    // rpc-api file
    let rpc_path = out_dir.path().join("crates/cclab-sdd/src/test_rpc.rs");
    assert!(rpc_path.exists(), "rpc-api target file should exist");
    let rpc_content = std::fs::read_to_string(&rpc_path).unwrap();
    assert!(
        rpc_content.contains("CODEGEN-BEGIN"),
        "rpc-api: CODEGEN-BEGIN marker"
    );
    assert!(
        rpc_content.contains("CODEGEN-END"),
        "rpc-api: CODEGEN-END marker"
    );
    assert!(
        rpc_content.contains("async fn ping"),
        "rpc-api: should emit async fn ping"
    );
    assert!(
        rpc_content.contains("async fn add_item"),
        "rpc-api: should emit async fn add_item"
    );
    assert!(
        rpc_content.contains("item_id:"),
        "rpc-api: should emit typed param"
    );

    // config file
    let cfg_path = out_dir.path().join("crates/cclab-sdd/src/test_config.rs");
    assert!(cfg_path.exists(), "config target file should exist");
    let cfg_content = std::fs::read_to_string(&cfg_path).unwrap();
    assert!(
        cfg_content.contains("CODEGEN-BEGIN"),
        "config: CODEGEN-BEGIN marker"
    );
    assert!(
        cfg_content.contains("struct ServerConfig"),
        "config: struct name from title"
    );
    assert!(
        cfg_content.contains("impl Default for ServerConfig"),
        "config: Default impl"
    );
    assert!(cfg_content.contains("8080"), "config: port default value");
}

#[test]
fn test_unsupported_language_errors_loud() {
    let tmpdir = tempfile::TempDir::new().unwrap();

    // Minimal spec targeting a file under a pyproject.toml-rooted workspace.
    let spec_content = r#"---
id: test-unsupported
fill_sections: [config, changes]
---

## Config
<!-- type: config lang: yaml -->

```yaml
title: PyConfig
type: object
properties:
  enabled:
    type: boolean
    default: true
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: pyapp/src/config.py
    action: create
    section: config
```
"#;
    let spec_path = tmpdir.path().join("unsupported_lang.md");
    std::fs::write(&spec_path, spec_content).unwrap();

    let out_dir = tempfile::TempDir::new().unwrap();
    // Create a python workspace under the target path.
    std::fs::create_dir_all(out_dir.path().join("pyapp/src")).unwrap();
    std::fs::write(
        out_dir.path().join("pyapp/pyproject.toml"),
        "[project]\nname = \"pyapp\"\n",
    )
    .unwrap();

    let result = run_apply(&spec_path, out_dir.path(), false);
    // Greenfield-generator contract: an entry no generator matches is NOT a
    // hard error — it produces marker-only output (SPEC-REF + TODO) so the
    // gap surfaces as a HANDWRITE marker instead of a blocked pipeline.
    let report = result.expect("marker-only output for unmatched generator");
    assert_eq!(report.files.len(), 1);
    assert!(report.wrote_files);
    let generated = out_dir.path().join("pyapp/src/config.py");
    let content = std::fs::read_to_string(&generated).unwrap();
    assert!(
        content.contains("SPEC-REF") && content.contains("TODO"),
        "marker-only output should carry SPEC-REF + TODO: {content}"
    );
}

#[test]
fn test_python_backend_models_apply_dispatches_python_emitter() {
    let spec_rel = Path::new("fixtures/python_backend/tech_design/api_models.md");
    let target_rel =
        Path::new("fixtures/python_backend/src/features/automations/video_qc/api_models.py");

    let out_dir = tempfile::TempDir::new().unwrap();
    let temp_spec = out_dir.path().join(spec_rel);
    let temp_target = out_dir.path().join(target_rel);
    std::fs::create_dir_all(temp_spec.parent().unwrap()).unwrap();
    std::fs::create_dir_all(temp_target.parent().unwrap()).unwrap();
    std::fs::write(&temp_spec, PYTHON_MODELS_SPEC).unwrap();
    std::fs::write(
        out_dir
            .path()
            .join("fixtures/python_backend/pyproject.toml"),
        "[project]\nname = \"python-backend-fixture\"\n",
    )
    .unwrap();

    let spec_ref = "fixtures/python_backend/tech_design/api_models.md#models";
    std::fs::write(
        &temp_target,
        format!(
            "# SPEC-MANAGED: {spec_ref}\n# CODEGEN-BEGIN\n# SPEC-REF: {spec_ref}\n# TODO: marker-only fallback should be replaced\n# CODEGEN-END\n",
        ),
    )
    .unwrap();

    let report =
        run_apply(&temp_spec, out_dir.path(), false).expect("python models apply should succeed");

    assert_eq!(
        report.files.len(),
        1,
        "one Python model file should be routed"
    );
    assert!(
        report.files[0].updated,
        "stale marker-only output should be replaced"
    );
    assert_eq!(report.files[0].blocks_updated, 1);

    let generated = std::fs::read_to_string(&temp_target).unwrap();
    assert_eq!(generated, PYTHON_MODELS_EXPECTED);
    assert!(
        !generated.contains("SPEC-REF:"),
        "Python model apply should use the emitter, not marker-only fallback"
    );
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
