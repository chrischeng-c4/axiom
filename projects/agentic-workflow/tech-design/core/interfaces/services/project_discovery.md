---
id: projects-sdd-src-services-project-discovery-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Service interfaces expose AW Core project, issue, and platform boundary behavior to clients."
---

# Standardized projects/agentic-workflow/src/services/project_discovery.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/project_discovery.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `discover_projects` | projects/agentic-workflow/src/services/project_discovery.rs | function | pub | 23 | discover_projects(root: &Path) -> Result<Vec<Project>> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/services/project_discovery.rs -->
```rust
//! Auto-discovery of project → workspace hierarchy.
//!
//! Walks `{crates,projects,packages}/*` and applies rules A-F in priority order
//! to infer the workspace layout and tech stack for each directory.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::models::project::{Project, Workspace};
use crate::models::tech_stack::Language;
use crate::services::tech_stack_service::infer_tech_stack;

/// Discovery root directories (relative to repo root).
const DISCOVERY_ROOTS: &[&str] = &["crates", "projects", "packages"];

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R1
/// Auto-discover all project-level dirs under `crates/`, `projects/`, `packages/`
/// and return a `Vec<Project>` with inferred workspace information.
pub fn discover_projects(root: &Path) -> Result<Vec<Project>> {
    let mut projects = Vec::new();

    for discovery_root in DISCOVERY_ROOTS {
        let dir = root.join(discovery_root);
        if !dir.is_dir() {
            continue;
        }

        let mut entries: Vec<PathBuf> = fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();

        // Sort for deterministic output
        entries.sort();

        for entry in entries {
            let name = match entry.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            // Relative path from repo root
            let rel_path = match entry.strip_prefix(root) {
                Ok(p) => p.to_path_buf(),
                Err(_) => continue,
            };

            let workspaces = apply_rules(root, &entry, &name);
            if workspaces.is_empty() {
                continue;
            }

            projects.push(Project {
                name,
                path: rel_path,
                tech_design_dir: None,
                // Discovery never invents EC bindings; they are declared by
                // hand in `.aw/config.toml` (wi-13).
                ec: Default::default(),
                workspaces,
            });
        }
    }

    Ok(projects)
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R2
/// Apply discovery rules A-F in priority order to produce workspaces for `dir`.
fn apply_rules(root: &Path, dir: &Path, project_name: &str) -> Vec<Workspace> {
    // Rule A: be/ AND fe/ both exist → 2 workspaces
    if let Some(ws) = rule_a(root, dir) {
        return ws;
    }
    // Rule B: Cargo.toml at root
    if let Some(ws) = rule_b(root, dir, project_name) {
        return vec![ws];
    }
    // Rule C: pyproject.toml at root
    if let Some(ws) = rule_c(root, dir, project_name) {
        return vec![ws];
    }
    // Rule D: package.json at root
    if let Some(ws) = rule_d(root, dir, project_name) {
        return vec![ws];
    }
    // Rule E: exactly one nested Cargo.toml (no root manifest)
    if let Some(ws) = rule_e(root, dir) {
        return vec![ws];
    }
    // Rule F: no manifest found
    vec![rule_f(root, dir, project_name)]
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R4
/// Rule A: directory has both `be/` and `fe/` subdirectories.
fn rule_a(root: &Path, dir: &Path) -> Option<Vec<Workspace>> {
    let be = dir.join("be");
    let fe = dir.join("fe");
    if !be.is_dir() || !fe.is_dir() {
        return None;
    }

    let be_target = infer_language_for_subdir(root, &be);
    let fe_target = infer_language_for_subdir(root, &fe);

    let be_rel = relative(root, &be);
    let fe_rel = relative(root, &fe);

    let be_ws = Workspace {
        name: Some("be".to_string()),
        paths: vec![format!("{}/**", be_rel)],
        target: be_target,
        test_cmd: infer_test_cmd_relative(root, &be, be_target, "be"),
        codegen: None,
    };
    let fe_ws = Workspace {
        name: Some("fe".to_string()),
        paths: vec![format!("{}/**", fe_rel)],
        target: fe_target,
        test_cmd: infer_test_cmd_relative(root, &fe, fe_target, "fe"),
        codegen: None,
    };

    Some(vec![be_ws, fe_ws])
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R2
/// Rule B: `Cargo.toml` at directory root.
fn rule_b(root: &Path, dir: &Path, project_name: &str) -> Option<Workspace> {
    if !dir.join("Cargo.toml").is_file() {
        return None;
    }
    let rel = relative(root, dir);
    Some(Workspace {
        name: None,
        paths: vec![format!("{}/**", rel)],
        target: Language::Rust,
        test_cmd: Some(format!("cargo test -p {}", project_name)),
        codegen: None,
    })
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R3
// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R7
/// Rule C: `pyproject.toml` at directory root.
fn rule_c(root: &Path, dir: &Path, _project_name: &str) -> Option<Workspace> {
    if !dir.join("pyproject.toml").is_file() {
        return None;
    }
    let rel = relative(root, dir);
    let test_cmd = if dir.join("uv.lock").is_file() {
        Some(format!("cd {} && uv run pytest", rel))
    } else {
        None
    };
    Some(Workspace {
        name: None,
        paths: vec![format!("{}/**", rel)],
        target: Language::Python,
        test_cmd,
        codegen: None,
    })
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R3
// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R7
/// Rule D: `package.json` at directory root.
fn rule_d(root: &Path, dir: &Path, _project_name: &str) -> Option<Workspace> {
    let pkg_json = dir.join("package.json");
    if !pkg_json.is_file() {
        return None;
    }
    let ts = infer_tech_stack(dir);
    let target = match ts.language {
        Some(Language::TypeScript) => Language::TypeScript,
        _ => Language::JavaScript,
    };
    let rel = relative(root, dir);
    let test_cmd = if has_vitest(&pkg_json) {
        Some(format!("cd {} && npx vitest run", rel))
    } else {
        None
    };
    Some(Workspace {
        name: None,
        paths: vec![format!("{}/**", rel)],
        target,
        test_cmd,
        codegen: None,
    })
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R2
// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R8
/// Rule E: exactly one single-level nested `Cargo.toml` with no root manifest.
///
/// The workspace name and `cargo test -p <name>` arg are derived from
/// `[package].name` in the nested Cargo.toml, not from the directory basename.
fn rule_e(root: &Path, dir: &Path) -> Option<Workspace> {
    // No root Cargo.toml already handled (B didn't fire)
    let entries: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect(),
        Err(_) => return None,
    };

    let nested_cargo: Vec<&PathBuf> = entries
        .iter()
        .filter(|sub| sub.join("Cargo.toml").is_file())
        .collect();

    if nested_cargo.len() != 1 {
        return None;
    }

    let sub = nested_cargo[0];
    let rel = relative(root, sub);

    // R8: read [package].name from nested Cargo.toml; fall back to directory basename
    let pkg_name = read_cargo_package_name(&sub.join("Cargo.toml")).unwrap_or_else(|| {
        sub.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    });

    Some(Workspace {
        name: Some(pkg_name.clone()),
        paths: vec![format!("{}/**", rel)],
        target: Language::Rust,
        test_cmd: Some(format!("cargo test -p {}", pkg_name)),
        codegen: None,
    })
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R2
/// Rule F: no manifest found anywhere — emit a schemas workspace.
fn rule_f(root: &Path, dir: &Path, project_name: &str) -> Workspace {
    let rel = relative(root, dir);
    Workspace {
        name: Some(project_name.to_string()),
        paths: vec![format!("{}/**", rel)],
        target: Language::Schemas,
        test_cmd: Some("true".to_string()),
        codegen: None,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Infer the language for a subdirectory (used in Rule A for be/fe).
fn infer_language_for_subdir(root: &Path, dir: &Path) -> Language {
    let ts = infer_tech_stack(dir);
    if ts.language.is_some() {
        return ts.language.unwrap();
    }
    // Check pyproject.toml explicitly since infer_tech_stack requires content
    if dir.join("Cargo.toml").is_file() {
        return Language::Rust;
    }
    if dir.join("pyproject.toml").is_file() {
        return Language::Python;
    }
    if dir.join("package.json").is_file() {
        return Language::TypeScript;
    }
    // Fall back to parent-based logic
    let _ = root;
    Language::Schemas
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R7
/// Infer the test command for a workspace based on its target, using a
/// **project-relative** path (e.g. `projects/conductor/be`), not an absolute path.
fn infer_test_cmd_relative(
    root: &Path,
    dir: &Path,
    target: Language,
    workspace_name: &str,
) -> Option<String> {
    match target {
        Language::Rust => Some(format!("cargo test -p {}", workspace_name)),
        Language::Python => {
            if dir.join("uv.lock").is_file() {
                let rel = relative(root, dir);
                Some(format!("cd {} && uv run pytest", rel))
            } else {
                None
            }
        }
        Language::TypeScript | Language::JavaScript => {
            if has_vitest(&dir.join("package.json")) {
                let rel = relative(root, dir);
                Some(format!("cd {} && npx vitest run", rel))
            } else {
                None
            }
        }
        Language::Schemas => Some("true".to_string()),
    }
}

/// Return the relative path from `root` to `path` as a forward-slash string.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Check whether `package.json` lists `vitest` in `devDependencies` or `dependencies`.
fn has_vitest(pkg_json: &Path) -> bool {
    let Ok(content) = fs::read_to_string(pkg_json) else {
        return false;
    };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    for key in ["dependencies", "devDependencies"] {
        if let Some(obj) = doc.get(key).and_then(|v| v.as_object()) {
            if obj.contains_key("vitest") {
                return true;
            }
        }
    }
    false
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R8
/// Read `[package].name` from a `Cargo.toml` file. Returns `None` if the file
/// is missing, malformed, or lacks a `[package]` table.
fn read_cargo_package_name(cargo_toml: &Path) -> Option<String> {
    let content = fs::read_to_string(cargo_toml).ok()?;
    let doc: toml::Value = toml::from_str(&content).ok()?;
    doc.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Create a minimal repo layout: `<tmp>/crates/<proj_name>/` and return
    /// (TempDir, project_dir path).  The TempDir is the "repo root".
    fn make_repo(proj_name: &str) -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let proj = tmp.path().join("crates").join(proj_name);
        fs::create_dir_all(&proj).unwrap();
        (tmp, proj)
    }

    // REQ: REQ-001
    #[test]
    fn rule_a_be_fe() {
        let (tmp, proj) = make_repo("my-proj");

        let be = proj.join("be");
        let fe = proj.join("fe");
        fs::create_dir_all(&be).unwrap();
        fs::create_dir_all(&fe).unwrap();
        fs::write(be.join("Cargo.toml"), "[package]\nname = \"be\"\n").unwrap();
        fs::write(
            fe.join("package.json"),
            r#"{"name":"fe","devDependencies":{"vitest":"^1.0.0"}}"#,
        )
        .unwrap();

        let projects = discover_projects(tmp.path()).unwrap();

        assert_eq!(projects.len(), 1);
        let p = &projects[0];
        assert_eq!(p.name, "my-proj");
        assert_eq!(p.workspaces.len(), 2);

        let names: Vec<Option<&str>> = p.workspaces.iter().map(|w| w.name.as_deref()).collect();
        assert!(names.contains(&Some("be")));
        assert!(names.contains(&Some("fe")));
    }

    // REQ: REQ-002
    #[test]
    fn rule_b_cargo() {
        let (tmp, proj) = make_repo("my-crate");
        fs::write(proj.join("Cargo.toml"), "[package]\nname = \"my-crate\"\n").unwrap();

        let projects = discover_projects(tmp.path()).unwrap();

        assert_eq!(projects.len(), 1);
        let p = &projects[0];
        assert_eq!(p.workspaces.len(), 1);
        let ws = &p.workspaces[0];
        assert_eq!(ws.target, Language::Rust);
        assert_eq!(ws.test_cmd.as_deref(), Some("cargo test -p my-crate"));
    }

    // REQ: REQ-003
    #[test]
    fn rule_c_pyproject_with_uv_lock() {
        let (tmp, proj) = make_repo("my-py");
        fs::write(proj.join("pyproject.toml"), "[project]\nname = \"my-py\"\n").unwrap();
        fs::write(proj.join("uv.lock"), "# lockfile\n").unwrap();

        let projects = discover_projects(tmp.path()).unwrap();

        assert_eq!(projects.len(), 1);
        let ws = &projects[0].workspaces[0];
        assert_eq!(ws.target, Language::Python);
        let cmd = ws.test_cmd.as_deref().expect("expected test_cmd");
        assert!(cmd.contains("uv run pytest"), "got: {cmd}");
    }

    // REQ: REQ-003
    #[test]
    fn rule_c_pyproject_no_uv_lock() {
        let (tmp, proj) = make_repo("my-py-nolock");
        fs::write(
            proj.join("pyproject.toml"),
            "[project]\nname = \"my-py-nolock\"\n",
        )
        .unwrap();

        let projects = discover_projects(tmp.path()).unwrap();

        assert_eq!(projects.len(), 1);
        let ws = &projects[0].workspaces[0];
        assert_eq!(ws.target, Language::Python);
        assert!(
            ws.test_cmd.is_none(),
            "expected no test_cmd without uv.lock"
        );
    }

    // REQ: REQ-004
    #[test]
    fn rule_d_package_json_with_vitest() {
        let (tmp, proj) = make_repo("my-ts");
        fs::write(
            proj.join("package.json"),
            r#"{"name":"my-ts","devDependencies":{"vitest":"^1.0.0","typescript":"^5.0.0"}}"#,
        )
        .unwrap();

        let projects = discover_projects(tmp.path()).unwrap();

        assert_eq!(projects.len(), 1);
        let ws = &projects[0].workspaces[0];
        let cmd = ws.test_cmd.as_deref().expect("expected test_cmd");
        assert!(cmd.contains("vitest run"), "got: {cmd}");
    }

    // REQ: REQ-004
    #[test]
    fn rule_d_package_json_no_vitest() {
        let (tmp, proj) = make_repo("my-js");
        fs::write(
            proj.join("package.json"),
            r#"{"name":"my-js","devDependencies":{"jest":"^29.0.0"}}"#,
        )
        .unwrap();

        let projects = discover_projects(tmp.path()).unwrap();

        assert_eq!(projects.len(), 1);
        let ws = &projects[0].workspaces[0];
        assert!(ws.test_cmd.is_none(), "expected no test_cmd without vitest");
    }

    // REQ: REQ-005
    #[test]
    fn rule_e_nested_cargo() {
        let (tmp, proj) = make_repo("my-multi");
        let cli = proj.join("cli");
        fs::create_dir_all(&cli).unwrap();
        fs::write(cli.join("Cargo.toml"), "[package]\nname = \"cli\"\n").unwrap();

        let projects = discover_projects(tmp.path()).unwrap();

        assert_eq!(projects.len(), 1);
        let ws = &projects[0].workspaces[0];
        assert_eq!(ws.name.as_deref(), Some("cli"));
        assert_eq!(ws.target, Language::Rust);
    }

    // REQ: REQ-006
    #[test]
    fn rule_f_no_manifest() {
        let (tmp, proj) = make_repo("schemas-proj");
        // Empty project directory — no manifests at any level.
        let _ = proj; // already created by make_repo

        let projects = discover_projects(tmp.path()).unwrap();

        assert_eq!(projects.len(), 1);
        let ws = &projects[0].workspaces[0];
        assert_eq!(ws.target, Language::Schemas);
        assert_eq!(ws.test_cmd.as_deref(), Some("true"));
    }

    // REQ: REQ-008 (R8: Rule E uses [package].name from nested Cargo.toml)
    // T19: rule_e_package_name
    #[test]
    fn rule_e_package_name() {
        // dir name = "cli", but [package].name = "agentic-workflow"
        let (tmp, proj) = make_repo("my-score");
        let cli = proj.join("cli");
        fs::create_dir_all(&cli).unwrap();
        // Cargo.toml has a DIFFERENT name from the directory basename "cli"
        fs::write(cli.join("Cargo.toml"), "[package]\nname = \"score\"\n").unwrap();

        let projects = discover_projects(tmp.path()).unwrap();

        assert_eq!(projects.len(), 1);
        let ws = &projects[0].workspaces[0];

        // R8: workspace name comes from [package].name, not directory name
        assert_eq!(
            ws.name.as_deref(),
            Some("score"),
            "Rule E workspace name must come from [package].name in nested Cargo.toml"
        );
        assert_eq!(
            ws.test_cmd.as_deref(),
            Some("cargo test -p agentic-workflow"),
            "Rule E test_cmd must use [package].name"
        );
    }

    // REQ: REQ-007 (R7: test_cmd uses project-relative paths, not absolute)
    // T20: test_cmd_relative_path
    #[test]
    fn test_cmd_relative_path() {
        // Layout: tmp/projects/conductor/ with pyproject.toml + uv.lock at the project root.
        // Discovery finds 'conductor' as a project, Rule C fires because pyproject.toml is at root.
        let tmp = TempDir::new().unwrap();
        let conductor = tmp.path().join("projects").join("conductor");
        fs::create_dir_all(&conductor).unwrap();
        fs::write(
            conductor.join("pyproject.toml"),
            "[project]\nname = \"conductor\"\n",
        )
        .unwrap();
        fs::write(conductor.join("uv.lock"), "# lockfile\n").unwrap();

        let projects = discover_projects(tmp.path()).unwrap();
        assert_eq!(projects.len(), 1);

        let ws = &projects[0].workspaces[0];
        let cmd = ws
            .test_cmd
            .as_deref()
            .expect("expected test_cmd for Rule C with uv.lock");

        // R7: must NOT contain the absolute tmp path
        let abs_prefix = tmp.path().to_string_lossy();
        assert!(
            !cmd.contains(abs_prefix.as_ref()),
            "test_cmd must not contain absolute path; got: {cmd}"
        );

        // R7: must use project-relative path (projects/conductor, not absolute)
        assert!(
            cmd.starts_with("cd projects/conductor"),
            "test_cmd must start with project-relative cd; got: {cmd}"
        );
        assert!(
            cmd.contains("uv run pytest"),
            "test_cmd must contain uv run pytest; got: {cmd}"
        );
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/project_discovery.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete project discovery module.
```
