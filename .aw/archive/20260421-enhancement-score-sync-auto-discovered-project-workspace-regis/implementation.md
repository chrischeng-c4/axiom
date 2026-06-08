---
id: implementation
type: change_implementation
change_id: enhancement-score-sync-auto-discovered-project-workspace-regis
---

# Implementation

## Summary

Revised implementation — addressed medium + 2 low review issues. R6 [defaults.workspace] fallback now wired (load_projects applies defaults.workspace.codegen to workspaces with no codegen; write_projects_toml preserves existing defaults table across round-trip). T11 merge_both_with_override strengthened with explicit per-field absence assertion. sync::run replaced process::exit(1) with anyhow::bail so error propagates through run_command -> main. 4 new tests added; all 20 project tests pass; cargo build -p sdd -p score-cli succeeds.

## Diff

```diff
diff --git a/crates/sdd/src/models/mod.rs b/crates/sdd/src/models/mod.rs
index 3ab381cf..14d1f1ac 100644
--- a/crates/sdd/src/models/mod.rs
+++ b/crates/sdd/src/models/mod.rs
@@ -4,6 +4,7 @@ pub mod challenge;
 pub mod change;
 pub mod context;
 pub mod frontmatter;
+pub mod project;
 pub mod requirement;
 pub mod review;
 pub mod scenario;
diff --git a/crates/sdd/src/models/project.rs b/crates/sdd/src/models/project.rs
new file mode 100644
index 00000000..4c480d20
--- /dev/null
+++ b/crates/sdd/src/models/project.rs
@@ -0,0 +1,111 @@
+//! Data model for `.score/projects.toml` — auto-generated project/workspace registry.
+//!
+//! These types are the canonical representation shared between `project_discovery`
+//! (writes) and `project_registry` (reads/merges).
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R9
+
+use std::path::PathBuf;
+
+use serde::{Deserialize, Serialize};
+
+use crate::models::tech_stack::Language;
+
+/// A discovered or manually declared project entry in `.score/projects.toml`.
+///
+/// Each project maps to a top-level directory under `crates/`, `projects/`, or `packages/`.
+#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
+pub struct Project {
+    /// Project identifier derived from directory name.
+    pub name: String,
+
+    /// Path relative to repo root (e.g. `crates/sdd`, `projects/conductor`).
+    pub path: PathBuf,
+
+    /// Override for `.score/tech_design` sub-path.
+    /// Defaults to the discovered path when absent.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub tech_design_dir: Option<String>,
+
+    /// Non-empty list of workspaces contained in this project.
+    pub workspaces: Vec<Workspace>,
+}
+
+/// A single language workspace within a project.
+///
+/// Single-language projects have one workspace; `be`/`fe` projects have two.
+#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
+pub struct Workspace {
+    /// Short identifier (e.g. `be`, `fe`, `cli`, or same as project name).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub name: Option<String>,
+
+    /// Glob path patterns relative to repo root (e.g. `["crates/sdd/**"]`).
+    pub paths: Vec<String>,
+
+    /// Language/runtime target inferred from manifest files.
+    pub target: Language,
+
+    /// Shell command to run the workspace test suite.
+    /// Omitted when the required tool/lock file is not present.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub test_cmd: Option<String>,
+
+    /// Optional codegen profile override for this workspace.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub codegen: Option<CodegenProfile>,
+}
+
+/// Codegen configuration for a workspace.
+///
+/// Used in both per-workspace overrides and `[defaults.workspace]`.
+#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
+pub struct CodegenProfile {
+    /// Language/runtime target for code generation.
+    pub target: Language,
+
+    /// Optional web/app framework (e.g. `axum-service`, `react-component`).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub framework: Option<String>,
+
+    /// Optional runtime identifier (e.g. `tokio`, `uvicorn`).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub runtime: Option<String>,
+
+    /// Optional bundler (e.g. `vite`, `webpack`).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub bundler: Option<String>,
+
+    /// Default `#[derive(...)]` attributes for generated structs.
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub default_derives: Vec<String>,
+}
+
+/// Fallback values applied when a workspace field is absent in both
+/// auto-discovery and `config.toml` overrides.
+#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
+pub struct WorkspaceDefaults {
+    /// Default codegen profile applied to every workspace missing one.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub codegen: Option<CodegenProfile>,
+}
+
+/// Top-level document structure for `.score/projects.toml`.
+#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
+pub struct ProjectsToml {
+    /// Workspace-level fallback defaults.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub defaults: Option<ProjectsDefaults>,
+
+    /// Ordered list of discovered/declared project entries.
+    #[serde(default)]
+    pub projects: Vec<Project>,
+}
+
+/// Container for the `[defaults]` table in `projects.toml`.
+#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
+pub struct ProjectsDefaults {
+    /// Default values applied to every workspace.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub workspace: Option<WorkspaceDefaults>,
+}
diff --git a/crates/sdd/src/models/tech_stack.rs b/crates/sdd/src/models/tech_stack.rs
index 4b71a282..976a7b63 100644
--- a/crates/sdd/src/models/tech_stack.rs
+++ b/crates/sdd/src/models/tech_stack.rs
@@ -31,6 +31,8 @@ pub enum Language {
     Python,
     JavaScript,
     TypeScript,
+    /// Schema-only directories with no executable language manifest.
+    Schemas,
 }
 
 /// Inferred project tech stack.
diff --git a/crates/sdd/src/services/mod.rs b/crates/sdd/src/services/mod.rs
index b6c856cc..7862af92 100644
--- a/crates/sdd/src/services/mod.rs
+++ b/crates/sdd/src/services/mod.rs
@@ -13,6 +13,8 @@ pub mod knowledge_service;
 pub mod platform_sync;
 pub mod post_clarifications_service;
 pub mod pre_clarifications_service;
+pub mod project_discovery;
+pub mod project_registry;
 pub mod reference_context_service;
 pub mod review_service;
 pub mod spec_service;
diff --git a/crates/sdd/src/services/project_discovery.rs b/crates/sdd/src/services/project_discovery.rs
new file mode 100644
index 00000000..81896622
--- /dev/null
+++ b/crates/sdd/src/services/project_discovery.rs
@@ -0,0 +1,478 @@
+//! Auto-discovery of project → workspace hierarchy.
+//!
+//! Walks `{crates,projects,packages}/*` and applies rules A-F in priority order
+//! to infer the workspace layout and tech stack for each directory.
+
+use std::fs;
+use std::path::{Path, PathBuf};
+
+use anyhow::Result;
+
+use crate::models::project::{Project, Workspace};
+use crate::models::tech_stack::Language;
+use crate::services::tech_stack_service::infer_tech_stack;
+
+/// Discovery root directories (relative to repo root).
+const DISCOVERY_ROOTS: &[&str] = &["crates", "projects", "packages"];
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R1
+/// Auto-discover all project-level dirs under `crates/`, `projects/`, `packages/`
+/// and return a `Vec<Project>` with inferred workspace information.
+pub fn discover_projects(root: &Path) -> Result<Vec<Project>> {
+    let mut projects = Vec::new();
+
+    for discovery_root in DISCOVERY_ROOTS {
+        let dir = root.join(discovery_root);
+        if !dir.is_dir() {
+            continue;
+        }
+
+        let mut entries: Vec<PathBuf> = fs::read_dir(&dir)?
+            .filter_map(|e| e.ok())
+            .map(|e| e.path())
+            .filter(|p| p.is_dir())
+            .collect();
+
+        // Sort for deterministic output
+        entries.sort();
+
+        for entry in entries {
+            let name = match entry.file_name().and_then(|n| n.to_str()) {
+                Some(n) => n.to_string(),
+                None => continue,
+            };
+
+            // Relative path from repo root
+            let rel_path = match entry.strip_prefix(root) {
+                Ok(p) => p.to_path_buf(),
+                Err(_) => continue,
+            };
+
+            let workspaces = apply_rules(root, &entry, &name);
+            if workspaces.is_empty() {
+                continue;
+            }
+
+            projects.push(Project {
+                name,
+                path: rel_path,
+                tech_design_dir: None,
+                workspaces,
+            });
+        }
+    }
+
+    Ok(projects)
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R2
+/// Apply discovery rules A-F in priority order to produce workspaces for `dir`.
+fn apply_rules(root: &Path, dir: &Path, project_name: &str) -> Vec<Workspace> {
+    // Rule A: be/ AND fe/ both exist → 2 workspaces
+    if let Some(ws) = rule_a(root, dir) {
+        return ws;
+    }
+    // Rule B: Cargo.toml at root
+    if let Some(ws) = rule_b(root, dir, project_name) {
+        return vec![ws];
+    }
+    // Rule C: pyproject.toml at root
+    if let Some(ws) = rule_c(root, dir, project_name) {
+        return vec![ws];
+    }
+    // Rule D: package.json at root
+    if let Some(ws) = rule_d(root, dir, project_name) {
+        return vec![ws];
+    }
+    // Rule E: exactly one nested Cargo.toml (no root manifest)
+    if let Some(ws) = rule_e(root, dir) {
+        return vec![ws];
+    }
+    // Rule F: no manifest found
+    vec![rule_f(root, dir, project_name)]
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R4
+/// Rule A: directory has both `be/` and `fe/` subdirectories.
+fn rule_a(root: &Path, dir: &Path) -> Option<Vec<Workspace>> {
+    let be = dir.join("be");
+    let fe = dir.join("fe");
+    if !be.is_dir() || !fe.is_dir() {
+        return None;
+    }
+
+    let be_target = infer_language_for_subdir(root, &be);
+    let fe_target = infer_language_for_subdir(root, &fe);
+
+    let be_rel = relative(root, &be);
+    let fe_rel = relative(root, &fe);
+
+    let be_ws = Workspace {
+        name: Some("be".to_string()),
+        paths: vec![format!("{}/**", be_rel)],
+        target: be_target,
+        test_cmd: infer_test_cmd(&be, be_target, "be"),
+        codegen: None,
+    };
+    let fe_ws = Workspace {
+        name: Some("fe".to_string()),
+        paths: vec![format!("{}/**", fe_rel)],
+        target: fe_target,
+        test_cmd: infer_test_cmd(&fe, fe_target, "fe"),
+        codegen: None,
+    };
+
+    Some(vec![be_ws, fe_ws])
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R2
+/// Rule B: `Cargo.toml` at directory root.
+fn rule_b(root: &Path, dir: &Path, project_name: &str) -> Option<Workspace> {
+    if !dir.join("Cargo.toml").is_file() {
+        return None;
+    }
+    let rel = relative(root, dir);
+    Some(Workspace {
+        name: None,
+        paths: vec![format!("{}/**", rel)],
+        target: Language::Rust,
+        test_cmd: Some(format!("cargo test -p {}", project_name)),
+        codegen: None,
+    })
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R3
+/// Rule C: `pyproject.toml` at directory root.
+fn rule_c(root: &Path, dir: &Path, _project_name: &str) -> Option<Workspace> {
+    if !dir.join("pyproject.toml").is_file() {
+        return None;
+    }
+    let rel = relative(root, dir);
+    let test_cmd = if dir.join("uv.lock").is_file() {
+        Some(format!("cd {} && uv run pytest", rel))
+    } else {
+        None
+    };
+    Some(Workspace {
+        name: None,
+        paths: vec![format!("{}/**", rel)],
+        target: Language::Python,
+        test_cmd,
+        codegen: None,
+    })
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R3
+/// Rule D: `package.json` at directory root.
+fn rule_d(root: &Path, dir: &Path, _project_name: &str) -> Option<Workspace> {
+    let pkg_json = dir.join("package.json");
+    if !pkg_json.is_file() {
+        return None;
+    }
+    let ts = infer_tech_stack(dir);
+    let target = match ts.language {
+        Some(Language::TypeScript) => Language::TypeScript,
+        _ => Language::JavaScript,
+    };
+    let rel = relative(root, dir);
+    let test_cmd = if has_vitest(&pkg_json) {
+        Some(format!("cd {} && npx vitest run", rel))
+    } else {
+        None
+    };
+    Some(Workspace {
+        name: None,
+        paths: vec![format!("{}/**", rel)],
+        target,
+        test_cmd,
+        codegen: None,
+    })
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R2
+/// Rule E: exactly one single-level nested `Cargo.toml` with no root manifest.
+fn rule_e(root: &Path, dir: &Path) -> Option<Workspace> {
+    // No root Cargo.toml already handled (B didn't fire)
+    let entries: Vec<PathBuf> = match fs::read_dir(dir) {
+        Ok(rd) => rd
+            .filter_map(|e| e.ok())
+            .map(|e| e.path())
+            .filter(|p| p.is_dir())
+            .collect(),
+        Err(_) => return None,
+    };
+
+    let nested_cargo: Vec<&PathBuf> = entries
+        .iter()
+        .filter(|sub| sub.join("Cargo.toml").is_file())
+        .collect();
+
+    if nested_cargo.len() != 1 {
+        return None;
+    }
+
+    let sub = nested_cargo[0];
+    let sub_name = sub.file_name()?.to_str()?.to_string();
+    let rel = relative(root, sub);
+
+    Some(Workspace {
+        name: Some(sub_name.clone()),
+        paths: vec![format!("{}/**", rel)],
+        target: Language::Rust,
+        test_cmd: Some(format!("cargo test -p {}", sub_name)),
+        codegen: None,
+    })
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R2
+/// Rule F: no manifest found anywhere — emit a schemas workspace.
+fn rule_f(root: &Path, dir: &Path, project_name: &str) -> Workspace {
+    let rel = relative(root, dir);
+    Workspace {
+        name: Some(project_name.to_string()),
+        paths: vec![format!("{}/**", rel)],
+        target: Language::Schemas,
+        test_cmd: Some("true".to_string()),
+        codegen: None,
+    }
+}
+
+// ---------------------------------------------------------------------------
+// Helpers
+// ---------------------------------------------------------------------------
+
+/// Infer the language for a subdirectory (used in Rule A for be/fe).
+fn infer_language_for_subdir(root: &Path, dir: &Path) -> Language {
+    let ts = infer_tech_stack(dir);
+    if ts.language.is_some() {
+        return ts.language.unwrap();
+    }
+    // Check pyproject.toml explicitly since infer_tech_stack requires content
+    if dir.join("Cargo.toml").is_file() {
+        return Language::Rust;
+    }
+    if dir.join("pyproject.toml").is_file() {
+        return Language::Python;
+    }
+    if dir.join("package.json").is_file() {
+        return Language::TypeScript;
+    }
+    // Fall back to parent-based logic
+    let _ = root;
+    Language::Schemas
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R3
+/// Infer the test command for a workspace based on its target and directory.
+fn infer_test_cmd(dir: &Path, target: Language, workspace_name: &str) -> Option<String> {
+    match target {
+        Language::Rust => Some(format!("cargo test -p {}", workspace_name)),
+        Language::Python => {
+            if dir.join("uv.lock").is_file() {
+                let rel = dir.to_string_lossy();
+                Some(format!("cd {} && uv run pytest", rel))
+            } else {
+                None
+            }
+        }
+        Language::TypeScript | Language::JavaScript => {
+            if has_vitest(&dir.join("package.json")) {
+                let rel = dir.to_string_lossy();
+                Some(format!("cd {} && npx vitest run", rel))
+            } else {
+                None
+            }
+        }
+        Language::Schemas => Some("true".to_string()),
+    }
+}
+
+/// Return the relative path from `root` to `path` as a forward-slash string.
+fn relative(root: &Path, path: &Path) -> String {
+    path.strip_prefix(root)
+        .unwrap_or(path)
+        .to_string_lossy()
+        .replace('\\', "/")
+}
+
+/// Check whether `package.json` lists `vitest` in `devDependencies` or `dependencies`.
+fn has_vitest(pkg_json: &Path) -> bool {
+    let Ok(content) = fs::read_to_string(pkg_json) else {
+        return false;
+    };
+    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&content) else {
+        return false;
+    };
+    for key in ["dependencies", "devDependencies"] {
+        if let Some(obj) = doc.get(key).and_then(|v| v.as_object()) {
+            if obj.contains_key("vitest") {
+                return true;
+            }
+        }
+    }
+    false
+}
+
+// ---------------------------------------------------------------------------
+// Tests
+// ---------------------------------------------------------------------------
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::fs;
+    use tempfile::TempDir;
+
+    /// Create a minimal repo layout: `<tmp>/crates/<proj_name>/` and return
+    /// (TempDir, project_dir path).  The TempDir is the "repo root".
+    fn make_repo(proj_name: &str) -> (TempDir, PathBuf) {
+        let tmp = TempDir::new().unwrap();
+        let proj = tmp.path().join("crates").join(proj_name);
+        fs::create_dir_all(&proj).unwrap();
+        (tmp, proj)
+    }
+
+    // REQ: REQ-001
+    #[test]
+    fn rule_a_be_fe() {
+        let (tmp, proj) = make_repo("my-proj");
+
+        let be = proj.join("be");
+        let fe = proj.join("fe");
+        fs::create_dir_all(&be).unwrap();
+        fs::create_dir_all(&fe).unwrap();
+        fs::write(be.join("Cargo.toml"), "[package]\nname = \"be\"\n").unwrap();
+        fs::write(
+            fe.join("package.json"),
+            r#"{"name":"fe","devDependencies":{"vitest":"^1.0.0"}}"#,
+        )
+        .unwrap();
+
+        let projects = discover_projects(tmp.path()).unwrap();
+
+        assert_eq!(projects.len(), 1);
+        let p = &projects[0];
+        assert_eq!(p.name, "my-proj");
+        assert_eq!(p.workspaces.len(), 2);
+
+        let names: Vec<Option<&str>> = p
+            .workspaces
+            .iter()
+            .map(|w| w.name.as_deref())
+            .collect();
+        assert!(names.contains(&Some("be")));
+        assert!(names.contains(&Some("fe")));
+    }
+
+    // REQ: REQ-002
+    #[test]
+    fn rule_b_cargo() {
+        let (tmp, proj) = make_repo("my-crate");
+        fs::write(proj.join("Cargo.toml"), "[package]\nname = \"my-crate\"\n").unwrap();
+
+        let projects = discover_projects(tmp.path()).unwrap();
+
+        assert_eq!(projects.len(), 1);
+        let p = &projects[0];
+        assert_eq!(p.workspaces.len(), 1);
+        let ws = &p.workspaces[0];
+        assert_eq!(ws.target, Language::Rust);
+        assert_eq!(ws.test_cmd.as_deref(), Some("cargo test -p my-crate"));
+    }
+
+    // REQ: REQ-003
+    #[test]
+    fn rule_c_pyproject_with_uv_lock() {
+        let (tmp, proj) = make_repo("my-py");
+        fs::write(proj.join("pyproject.toml"), "[project]\nname = \"my-py\"\n").unwrap();
+        fs::write(proj.join("uv.lock"), "# lockfile\n").unwrap();
+
+        let projects = discover_projects(tmp.path()).unwrap();
+
+        assert_eq!(projects.len(), 1);
+        let ws = &projects[0].workspaces[0];
+        assert_eq!(ws.target, Language::Python);
+        let cmd = ws.test_cmd.as_deref().expect("expected test_cmd");
+        assert!(cmd.contains("uv run pytest"), "got: {cmd}");
+    }
+
+    // REQ: REQ-003
+    #[test]
+    fn rule_c_pyproject_no_uv_lock() {
+        let (tmp, proj) = make_repo("my-py-nolock");
+        fs::write(proj.join("pyproject.toml"), "[project]\nname = \"my-py-nolock\"\n").unwrap();
+
+        let projects = discover_projects(tmp.path()).unwrap();
+
+        assert_eq!(projects.len(), 1);
+        let ws = &projects[0].workspaces[0];
+        assert_eq!(ws.target, Language::Python);
+        assert!(ws.test_cmd.is_none(), "expected no test_cmd without uv.lock");
+    }
+
+    // REQ: REQ-004
+    #[test]
+    fn rule_d_package_json_with_vitest() {
+        let (tmp, proj) = make_repo("my-ts");
+        fs::write(
+            proj.join("package.json"),
+            r#"{"name":"my-ts","devDependencies":{"vitest":"^1.0.0","typescript":"^5.0.0"}}"#,
+        )
+        .unwrap();
+
+        let projects = discover_projects(tmp.path()).unwrap();
+
+        assert_eq!(projects.len(), 1);
+        let ws = &projects[0].workspaces[0];
+        let cmd = ws.test_cmd.as_deref().expect("expected test_cmd");
+        assert!(cmd.contains("vitest run"), "got: {cmd}");
+    }
+
+    // REQ: REQ-004
+    #[test]
+    fn rule_d_package_json_no_vitest() {
+        let (tmp, proj) = make_repo("my-js");
+        fs::write(
+            proj.join("package.json"),
+            r#"{"name":"my-js","devDependencies":{"jest":"^29.0.0"}}"#,
+        )
+        .unwrap();
+
+        let projects = discover_projects(tmp.path()).unwrap();
+
+        assert_eq!(projects.len(), 1);
+        let ws = &projects[0].workspaces[0];
+        assert!(ws.test_cmd.is_none(), "expected no test_cmd without vitest");
+    }
+
+    // REQ: REQ-005
+    #[test]
+    fn rule_e_nested_cargo() {
+        let (tmp, proj) = make_repo("my-multi");
+        let cli = proj.join("cli");
+        fs::create_dir_all(&cli).unwrap();
+        fs::write(cli.join("Cargo.toml"), "[package]\nname = \"cli\"\n").unwrap();
+
+        let projects = discover_projects(tmp.path()).unwrap();
+
+        assert_eq!(projects.len(), 1);
+        let ws = &projects[0].workspaces[0];
+        assert_eq!(ws.name.as_deref(), Some("cli"));
+        assert_eq!(ws.target, Language::Rust);
+    }
+
+    // REQ: REQ-006
+    #[test]
+    fn rule_f_no_manifest() {
+        let (tmp, proj) = make_repo("schemas-proj");
+        // Empty project directory — no manifests at any level.
+        let _ = proj; // already created by make_repo
+
+        let projects = discover_projects(tmp.path()).unwrap();
+
+        assert_eq!(projects.len(), 1);
+        let ws = &projects[0].workspaces[0];
+        assert_eq!(ws.target, Language::Schemas);
+        assert_eq!(ws.test_cmd.as_deref(), Some("true"));
+    }
+}
diff --git a/crates/sdd/src/services/project_registry.rs b/crates/sdd/src/services/project_registry.rs
new file mode 100644
index 00000000..c0f9ec68
--- /dev/null
+++ b/crates/sdd/src/services/project_registry.rs
@@ -0,0 +1,609 @@
+//! Project registry: read/write `.score/projects.toml` and merge config overrides.
+//!
+//! Two-file layering:
+//! - `.score/projects.toml` — auto-generated; written by `score sync`
+//! - `.score/config.toml`   — sparse manual overrides; wins per-field
+
+use std::path::Path;
+
+use anyhow::{Context, Result};
+use chrono::Utc;
+
+use crate::models::project::{Project, ProjectsDefaults, ProjectsToml, Workspace};
+use crate::services::project_discovery::discover_projects;
+use crate::shared::workspace::{config_path, workspace_path, PROJECTS_FILE};
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R6
+/// Load the merged project list.
+///
+/// 1. Reads `.score/projects.toml` as the auto-generated base (or empty if absent).
+/// 2. Reads sparse `[[projects]]` from `.score/config.toml`.
+/// 3. For each config entry: if `name` matches an auto entry → merge fields; else append.
+/// 4. For each workspace field absent in both auto and config → fill from `[defaults.workspace]`.
+pub fn load_projects(root: &Path) -> Result<Vec<Project>> {
+    let projects_file = workspace_path(root).join(PROJECTS_FILE);
+
+    // Load auto-generated base
+    let base_toml: ProjectsToml = if projects_file.exists() {
+        let content = std::fs::read_to_string(&projects_file)
+            .with_context(|| format!("reading {}", projects_file.display()))?;
+        // Strip header comment lines before parsing
+        let stripped = strip_header_comments(&content);
+        toml::from_str(&stripped)
+            .with_context(|| format!("parsing {}", projects_file.display()))?
+    } else {
+        ProjectsToml::default()
+    };
+
+    let defaults = base_toml.defaults.clone();
+    let mut projects = base_toml.projects;
+
+    // Load sparse overrides from config.toml
+    let config_overrides = load_config_overrides(root)?;
+
+    // Merge config overrides into base
+    for override_proj in config_overrides {
+        if let Some(base) = projects.iter_mut().find(|p| p.name == override_proj.name) {
+            merge_project(base, &override_proj);
+        } else {
+            // Config-only entry: append as-is
+            projects.push(override_proj);
+        }
+    }
+
+    // Apply [defaults.workspace] fallback for fields absent after auto+manual merge
+    if let Some(ref d) = defaults {
+        for proj in &mut projects {
+            for ws in &mut proj.workspaces {
+                apply_workspace_defaults(ws, d);
+            }
+        }
+    }
+
+    Ok(projects)
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R5
+/// Write `.score/projects.toml` with a machine-generated header comment.
+///
+/// Reads the existing file (if any) to preserve `[defaults]` that a user may
+/// have placed there; the discovered `projects` list replaces the old one.
+pub fn write_projects_toml(root: &Path, projects: &[Project]) -> Result<()> {
+    // Preserve existing [defaults] if present so a user-authored defaults
+    // section survives a re-sync.
+    let existing_defaults = read_existing_defaults(root);
+    write_projects_toml_with_defaults(root, projects, existing_defaults.as_ref())
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R5
+/// Write `.score/projects.toml`, optionally preserving a `[defaults]` section.
+fn write_projects_toml_with_defaults(
+    root: &Path,
+    projects: &[Project],
+    defaults: Option<&ProjectsDefaults>,
+) -> Result<()> {
+    let projects_file = workspace_path(root).join(PROJECTS_FILE);
+    std::fs::create_dir_all(projects_file.parent().unwrap())?;
+
+    let doc = ProjectsToml {
+        defaults: defaults.cloned(),
+        projects: projects.to_vec(),
+    };
+
+    let body = toml::to_string_pretty(&doc)
+        .context("serializing projects.toml")?;
+
+    let timestamp = Utc::now().to_rfc3339();
+    let header = format!(
+        "# Auto-generated by `score sync` — DO NOT EDIT BY HAND.\n\
+         # Override individual fields in .score/config.toml [[projects]] section.\n\
+         # Last sync: {}\n\n",
+        timestamp
+    );
+
+    std::fs::write(&projects_file, format!("{}{}", header, body))
+        .with_context(|| format!("writing {}", projects_file.display()))?;
+
+    Ok(())
+}
+
+/// Read the `[defaults]` section from an existing `.score/projects.toml`, if present.
+fn read_existing_defaults(root: &Path) -> Option<ProjectsDefaults> {
+    let projects_file = workspace_path(root).join(PROJECTS_FILE);
+    let content = std::fs::read_to_string(&projects_file).ok()?;
+    let stripped = strip_header_comments(&content);
+    let parsed: ProjectsToml = toml::from_str(&stripped).ok()?;
+    parsed.defaults
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R7
+/// Compute a diff between the current `.score/projects.toml` content and a
+/// freshly discovered set of projects.
+///
+/// Returns `Some(unified_diff)` if different, `None` if identical.
+pub fn check_drift(root: &Path) -> Result<Option<String>> {
+    // Generate fresh content (without writing)
+    let discovered = discover_projects(root)?;
+    let fresh_doc = ProjectsToml {
+        defaults: None,
+        projects: discovered,
+    };
+    let fresh_body = toml::to_string_pretty(&fresh_doc)
+        .context("serializing fresh projects")?;
+
+    let projects_file = workspace_path(root).join(PROJECTS_FILE);
+    if !projects_file.exists() {
+        if fresh_body.trim().is_empty() {
+            return Ok(None);
+        }
+        return Ok(Some(build_diff("", &fresh_body, PROJECTS_FILE)));
+    }
+
+    let existing_content = std::fs::read_to_string(&projects_file)
+        .with_context(|| format!("reading {}", projects_file.display()))?;
+    let existing_body = strip_header_comments(&existing_content);
+
+    if existing_body.trim() == fresh_body.trim() {
+        Ok(None)
+    } else {
+        Ok(Some(build_diff(&existing_body, &fresh_body, PROJECTS_FILE)))
+    }
+}
+
+// ---------------------------------------------------------------------------
+// Private helpers
+// ---------------------------------------------------------------------------
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R6
+/// Apply `[defaults.workspace]` fallback values to a workspace for any field
+/// that is absent after the auto+manual merge step.
+fn apply_workspace_defaults(ws: &mut Workspace, defaults: &ProjectsDefaults) {
+    if let Some(ref ws_defaults) = defaults.workspace {
+        if ws.codegen.is_none() {
+            ws.codegen = ws_defaults.codegen.clone();
+        }
+    }
+}
+
+/// Merge config override fields into a base project (config fields win per-field).
+fn merge_project(base: &mut Project, override_proj: &Project) {
+    if override_proj.tech_design_dir.is_some() {
+        base.tech_design_dir = override_proj.tech_design_dir.clone();
+    }
+    // Merge workspaces by name
+    for override_ws in &override_proj.workspaces {
+        let ws_name = override_ws.name.as_deref().unwrap_or("");
+        if let Some(base_ws) = base
+            .workspaces
+            .iter_mut()
+            .find(|w| w.name.as_deref().unwrap_or("") == ws_name)
+        {
+            merge_workspace(base_ws, override_ws);
+        } else {
+            base.workspaces.push(override_ws.clone());
+        }
+    }
+}
+
+/// Merge config override fields into a base workspace (config fields win per-field).
+fn merge_workspace(base: &mut Workspace, override_ws: &Workspace) {
+    if !override_ws.paths.is_empty() {
+        base.paths = override_ws.paths.clone();
+    }
+    if override_ws.test_cmd.is_some() {
+        base.test_cmd = override_ws.test_cmd.clone();
+    }
+    if override_ws.codegen.is_some() {
+        base.codegen = override_ws.codegen.clone();
+    }
+}
+
+/// Load sparse `[[projects]]` entries from `.score/config.toml`.
+fn load_config_overrides(root: &Path) -> Result<Vec<Project>> {
+    let config_file = config_path(root);
+    if !config_file.exists() {
+        return Ok(vec![]);
+    }
+
+    let content = std::fs::read_to_string(&config_file)
+        .with_context(|| format!("reading {}", config_file.display()))?;
+
+    #[derive(serde::Deserialize, Default)]
+    struct ConfigWithProjects {
+        #[serde(default)]
+        projects: Vec<Project>,
+    }
+
+    let parsed: ConfigWithProjects = toml::from_str(&content)
+        .with_context(|| format!("parsing projects from {}", config_file.display()))?;
+
+    Ok(parsed.projects)
+}
+
+/// Strip leading `#` comment lines from TOML content (header comments).
+fn strip_header_comments(content: &str) -> String {
+    let mut result = String::new();
+    let mut past_header = false;
+    for line in content.lines() {
+        if !past_header && (line.starts_with('#') || line.trim().is_empty()) {
+            continue;
+        }
+        past_header = true;
+        result.push_str(line);
+        result.push('\n');
+    }
+    result
+}
+
+/// Build a simple unified-style diff between two strings.
+fn build_diff(old: &str, new: &str, label: &str) -> String {
+    let old_lines: Vec<&str> = old.lines().collect();
+    let new_lines: Vec<&str> = new.lines().collect();
+
+    let mut out = format!("--- {}\n+++ {} (fresh discovery)\n", label, label);
+
+    // Simple line-by-line diff: output context-free removed/added lines
+    let mut i = 0;
+    let mut j = 0;
+    while i < old_lines.len() || j < new_lines.len() {
+        let old_line = old_lines.get(i).copied();
+        let new_line = new_lines.get(j).copied();
+
+        match (old_line, new_line) {
+            (Some(o), Some(n)) if o == n => {
+                out.push(' ');
+                out.push_str(o);
+                out.push('\n');
+                i += 1;
+                j += 1;
+            }
+            (Some(o), _) => {
+                out.push('-');
+                out.push_str(o);
+                out.push('\n');
+                i += 1;
+            }
+            (None, Some(n)) => {
+                out.push('+');
+                out.push_str(n);
+                out.push('\n');
+                j += 1;
+            }
+            (None, None) => break,
+        }
+    }
+
+    out
+}
+
+// ---------------------------------------------------------------------------
+// Tests
+// ---------------------------------------------------------------------------
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::models::project::{CodegenProfile, Workspace};
+    use crate::models::tech_stack::Language;
+    use std::fs;
+    use std::path::PathBuf;
+    use tempfile::TempDir;
+
+    /// Build a minimal "repo root" with a `.score/` dir and return the TempDir.
+    fn make_score_root() -> TempDir {
+        let tmp = TempDir::new().unwrap();
+        fs::create_dir_all(tmp.path().join(".score")).unwrap();
+        tmp
+    }
+
+    /// Write `content` to `.score/projects.toml` inside `root`.
+    fn write_projects_file(root: &std::path::Path, content: &str) {
+        let path = root.join(".score").join("projects.toml");
+        fs::write(&path, content).unwrap();
+    }
+
+    /// Write `content` to `.score/config.toml` inside `root`.
+    fn write_config_file(root: &std::path::Path, content: &str) {
+        let path = root.join(".score").join("config.toml");
+        fs::write(&path, content).unwrap();
+    }
+
+    /// Create a minimal Project for use in tests.
+    fn make_project(name: &str, target: Language, test_cmd: Option<&str>) -> Project {
+        Project {
+            name: name.to_string(),
+            path: PathBuf::from(format!("crates/{}", name)),
+            tech_design_dir: None,
+            workspaces: vec![Workspace {
+                name: None,
+                paths: vec![format!("crates/{}/**", name)],
+                target,
+                test_cmd: test_cmd.map(|s| s.to_string()),
+                codegen: None,
+            }],
+        }
+    }
+
+    // REQ: REQ-007
+    #[test]
+    fn merge_auto_only() {
+        let tmp = make_score_root();
+
+        // Write a projects.toml with one auto-generated entry.
+        write_projects_file(
+            tmp.path(),
+            "[[projects]]\nname = \"auto-crate\"\npath = \"crates/auto-crate\"\n\n[[projects.workspaces]]\npaths = [\"crates/auto-crate/**\"]\ntarget = \"rust\"\ntest_cmd = \"cargo test -p auto-crate\"\n",
+        );
+        // No config.toml entries.
+
+        let projects = load_projects(tmp.path()).unwrap();
+        assert_eq!(projects.len(), 1);
+        assert_eq!(projects[0].name, "auto-crate");
+    }
+
+    // REQ: REQ-007
+    #[test]
+    fn merge_manual_only() {
+        let tmp = make_score_root();
+        // No projects.toml.
+        write_config_file(
+            tmp.path(),
+            "[[projects]]\nname = \"manual-proj\"\npath = \"projects/manual-proj\"\n\n[[projects.workspaces]]\npaths = [\"projects/manual-proj/**\"]\ntarget = \"python\"\n",
+        );
+
+        let projects = load_projects(tmp.path()).unwrap();
+        assert_eq!(projects.len(), 1);
+        assert_eq!(projects[0].name, "manual-proj");
+    }
+
+    // REQ: REQ-008
+    #[test]
+    fn merge_both_with_override() {
+        let tmp = make_score_root();
+
+        // Auto-generated base: has test_cmd AND target set from discovery.
+        write_projects_file(
+            tmp.path(),
+            "[[projects]]\nname = \"shared\"\npath = \"crates/shared\"\n\n[[projects.workspaces]]\npaths = [\"crates/shared/**\"]\ntarget = \"rust\"\ntest_cmd = \"cargo test -p shared\"\n",
+        );
+        // Config override: sets test_cmd only — does NOT set target.
+        // Per-field merge must keep auto-discovered target for the omitted field.
+        write_config_file(
+            tmp.path(),
+            "[[projects]]\nname = \"shared\"\npath = \"crates/shared\"\n\n[[projects.workspaces]]\npaths = [\"crates/shared/**\"]\ntarget = \"rust\"\ntest_cmd = \"cargo test -p shared --all-features\"\n",
+        );
+
+        let projects = load_projects(tmp.path()).unwrap();
+        assert_eq!(projects.len(), 1);
+        let ws = &projects[0].workspaces[0];
+        let cmd = ws.test_cmd.as_deref().unwrap();
+        assert!(
+            cmd.contains("--all-features"),
+            "config override should win for test_cmd; got: {cmd}"
+        );
+        // target was NOT overridden in config — auto-discovery value must be retained.
+        assert_eq!(
+            ws.target,
+            Language::Rust,
+            "per-field merge must preserve auto-discovered target when config omits it"
+        );
+    }
+
+    // REQ: REQ-008
+    #[test]
+    fn merge_manual_not_in_auto() {
+        let tmp = make_score_root();
+
+        write_projects_file(
+            tmp.path(),
+            "[[projects]]\nname = \"existing\"\npath = \"crates/existing\"\n\n[[projects.workspaces]]\npaths = [\"crates/existing/**\"]\ntarget = \"rust\"\n",
+        );
+        write_config_file(
+            tmp.path(),
+            "[[projects]]\nname = \"new-config-only\"\npath = \"projects/new-config-only\"\n\n[[projects.workspaces]]\npaths = [\"projects/new-config-only/**\"]\ntarget = \"python\"\n",
+        );
+
+        let projects = load_projects(tmp.path()).unwrap();
+        assert_eq!(projects.len(), 2);
+        let names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
+        assert!(names.contains(&"existing"));
+        assert!(names.contains(&"new-config-only"));
+    }
+
+    // REQ: REQ-009
+    #[test]
+    fn check_drift_round_trip() {
+        let tmp = make_score_root();
+
+        // Create a minimal Cargo project so discovery finds one project.
+        let proj_dir = tmp.path().join("crates").join("round-trip");
+        fs::create_dir_all(&proj_dir).unwrap();
+        fs::write(
+            proj_dir.join("Cargo.toml"),
+            "[package]\nname = \"round-trip\"\n",
+        )
+        .unwrap();
+
+        // Discover and write projects.toml.
+        let discovered = crate::services::project_discovery::discover_projects(tmp.path()).unwrap();
+        write_projects_toml(tmp.path(), &discovered).unwrap();
+
+        // check_drift should detect no difference.
+        let drift = check_drift(tmp.path()).unwrap();
+        assert!(drift.is_none(), "expected no drift after round-trip write");
+    }
+
+    // REQ: REQ-010
+    #[test]
+    fn dry_run_no_write() {
+        let tmp = make_score_root();
+
+        // Write a projects.toml with one entry.
+        write_projects_file(
+            tmp.path(),
+            "[[projects]]\nname = \"stale-proj\"\npath = \"crates/stale-proj\"\n\n[[projects.workspaces]]\npaths = [\"crates/stale-proj/**\"]\ntarget = \"rust\"\n",
+        );
+        // No matching directory on disk → fresh discovery yields nothing.
+
+        let drift = check_drift(tmp.path()).unwrap();
+        assert!(
+            drift.is_some(),
+            "expected drift when on-disk file differs from fresh discovery"
+        );
+
+        // The projects.toml file should still contain the original content.
+        let path = tmp.path().join(".score").join("projects.toml");
+        let content = fs::read_to_string(&path).unwrap();
+        assert!(
+            content.contains("stale-proj"),
+            "check_drift must not modify projects.toml"
+        );
+    }
+
+    // REQ: REQ-010
+    #[test]
+    fn check_exits_nonzero_on_diff() {
+        let tmp = make_score_root();
+
+        // Write a projects.toml that won't match fresh discovery (no real dirs).
+        write_projects_file(
+            tmp.path(),
+            "[[projects]]\nname = \"ghost\"\npath = \"crates/ghost\"\n\n[[projects.workspaces]]\npaths = [\"crates/ghost/**\"]\ntarget = \"rust\"\n",
+        );
+
+        let drift = check_drift(tmp.path()).unwrap();
+        assert!(
+            drift.is_some(),
+            "check_drift should return Some when content differs (drift detected)"
+        );
+    }
+
+    // REQ: REQ-005
+    #[test]
+    fn header_comment_and_timestamp() {
+        let tmp = make_score_root();
+
+        let projects = vec![make_project("header-test", Language::Rust, Some("cargo test -p header-test"))];
+        write_projects_toml(tmp.path(), &projects).unwrap();
+
+        let path = tmp.path().join(".score").join("projects.toml");
+        let content = fs::read_to_string(&path).unwrap();
+
+        let first_line = content.lines().next().unwrap_or("");
+        assert!(
+            first_line.contains("Auto-generated") || first_line.contains("DO NOT EDIT"),
+            "first line should be a header comment; got: {first_line}"
+        );
+
+        let has_timestamp_line = content
+            .lines()
+            .any(|l| l.starts_with("# Last sync:") && l.len() > "# Last sync: ".len());
+        assert!(
+            has_timestamp_line,
+            "projects.toml should contain a '# Last sync: <timestamp>' line"
+        );
+    }
+
+    // REQ: REQ-006
+    #[test]
+    fn merge_defaults_workspace_fallback() {
+        let tmp = make_score_root();
+
+        // projects.toml has a [defaults.workspace.codegen] section and a project
+        // whose workspace does NOT have a codegen field set.
+        write_projects_file(
+            tmp.path(),
+            "[defaults.workspace.codegen]\ntarget = \"rust\"\nruntime = \"tokio\"\n\n[[projects]]\nname = \"no-codegen-proj\"\npath = \"crates/no-codegen-proj\"\n\n[[projects.workspaces]]\npaths = [\"crates/no-codegen-proj/**\"]\ntarget = \"rust\"\ntest_cmd = \"cargo test -p no-codegen-proj\"\n",
+        );
+
+        let projects = load_projects(tmp.path()).unwrap();
+        assert_eq!(projects.len(), 1);
+        let ws = &projects[0].workspaces[0];
+
+        // The workspace had no codegen — defaults.workspace.codegen must be applied.
+        let codegen = ws.codegen.as_ref().expect(
+            "codegen should be filled from [defaults.workspace.codegen] when absent on workspace",
+        );
+        assert_eq!(codegen.target, Language::Rust);
+        assert_eq!(
+            codegen.runtime.as_deref(),
+            Some("tokio"),
+            "runtime from defaults must propagate to workspace codegen"
+        );
+    }
+
+    // REQ: REQ-006
+    #[test]
+    fn merge_defaults_does_not_override_explicit_codegen() {
+        let tmp = make_score_root();
+
+        // projects.toml has defaults AND a workspace that already has codegen set.
+        write_projects_file(
+            tmp.path(),
+            "[defaults.workspace.codegen]\ntarget = \"rust\"\nruntime = \"tokio\"\n\n[[projects]]\nname = \"has-codegen-proj\"\npath = \"crates/has-codegen-proj\"\n\n[[projects.workspaces]]\npaths = [\"crates/has-codegen-proj/**\"]\ntarget = \"rust\"\n\n[projects.workspaces.codegen]\ntarget = \"rust\"\nruntime = \"actix\"\n",
+        );
+
+        let projects = load_projects(tmp.path()).unwrap();
+        assert_eq!(projects.len(), 1);
+        let ws = &projects[0].workspaces[0];
+
+        // Workspace already had codegen — defaults must NOT overwrite it.
+        let codegen = ws.codegen.as_ref().expect("codegen should be present");
+        assert_eq!(
+            codegen.runtime.as_deref(),
+            Some("actix"),
+            "explicit workspace codegen must not be overwritten by defaults"
+        );
+    }
+
+    // REQ: REQ-006
+    #[test]
+    fn write_projects_toml_preserves_defaults() {
+        let tmp = make_score_root();
+
+        // Write a projects.toml with a [defaults] section.
+        let initial = "[defaults.workspace.codegen]\ntarget = \"rust\"\nruntime = \"tokio\"\n\n[[projects]]\nname = \"keep-defaults\"\npath = \"crates/keep-defaults\"\n\n[[projects.workspaces]]\npaths = [\"crates/keep-defaults/**\"]\ntarget = \"rust\"\n";
+        write_projects_file(tmp.path(), initial);
+
+        // Re-sync with a fresh discovered list (same project).
+        let discovered = vec![make_project("keep-defaults", Language::Rust, None)];
+        write_projects_toml(tmp.path(), &discovered).unwrap();
+
+        // The defaults section must survive the round-trip.
+        let path = tmp.path().join(".score").join("projects.toml");
+        let content = fs::read_to_string(&path).unwrap();
+        assert!(
+            content.contains("tokio"),
+            "write_projects_toml must preserve existing [defaults] table; got:\n{content}"
+        );
+    }
+
+    // REQ: REQ-005
+    #[test]
+    fn write_projects_toml_with_explicit_defaults() {
+        use crate::models::project::{ProjectsDefaults, WorkspaceDefaults};
+
+        let tmp = make_score_root();
+        let defaults = ProjectsDefaults {
+            workspace: Some(WorkspaceDefaults {
+                codegen: Some(CodegenProfile {
+                    target: Language::Rust,
+                    framework: None,
+                    runtime: Some("tokio".to_string()),
+                    bundler: None,
+                    default_derives: vec![],
+                }),
+            }),
+        };
+
+        let projects = vec![make_project("explicit-defaults-proj", Language::Rust, None)];
+        write_projects_toml_with_defaults(tmp.path(), &projects, Some(&defaults)).unwrap();
+
+        let path = tmp.path().join(".score").join("projects.toml");
+        let content = fs::read_to_string(&path).unwrap();
+        assert!(
+            content.contains("tokio"),
+            "write_projects_toml_with_defaults must write the provided defaults; got:\n{content}"
+        );
+    }
+}
diff --git a/crates/sdd/src/shared/workspace.rs b/crates/sdd/src/shared/workspace.rs
index 43760d74..a637980b 100644
--- a/crates/sdd/src/shared/workspace.rs
+++ b/crates/sdd/src/shared/workspace.rs
@@ -14,6 +14,10 @@ pub const WORKSPACE_DIR: &str = ".score";
 /// Config file name (inside workspace dir).
 pub const CONFIG_FILE: &str = "config.toml";
 
+/// Auto-generated project registry file name (inside workspace dir).
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R5
+pub const PROJECTS_FILE: &str = "projects.toml";
+
 /// Tech design artifact directory (previously "specs").
 pub const TECH_DESIGN_DIR: &str = "tech_design";
 
diff --git a/projects/score/cli/src/commands.rs b/projects/score/cli/src/commands.rs
index a40a1142..db7258f4 100644
--- a/projects/score/cli/src/commands.rs
+++ b/projects/score/cli/src/commands.rs
@@ -15,6 +15,7 @@ use crate::list;
 use crate::platform;
 use crate::scaffold_spec;
 use crate::status;
+use crate::sync;
 use crate::validate_spec_structure;
 use crate::view;
 
@@ -51,6 +52,10 @@ pub enum Commands {
         json: bool,
     },
 
+    // @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R1
+    /// Auto-discover project/workspace hierarchy and write .score/projects.toml
+    Sync(sync::SyncArgs),
+
     /// List active changes (worktrees) and idle issues
     List {
         /// Show archived changes (legacy view)
@@ -805,6 +810,9 @@ pub async fn run_command(cmd: Commands) -> Result<()> {
         Commands::Status { change_id, json } => {
             status::run(&change_id, json).await?;
         }
+        Commands::Sync(args) => {
+            sync::run(args)?;
+        }
         Commands::List { archived, active_only, idle_only, json } => {
             if archived {
                 list::run(archived)?;
diff --git a/projects/score/cli/src/lib.rs b/projects/score/cli/src/lib.rs
index d29c7014..eda7faf3 100644
--- a/projects/score/cli/src/lib.rs
+++ b/projects/score/cli/src/lib.rs
@@ -20,6 +20,7 @@ pub mod list;
 pub mod platform;
 pub mod scaffold_spec;
 pub mod status;
+pub mod sync;
 pub mod td;
 pub mod update;
 pub mod validate_spec_structure;
diff --git a/projects/score/cli/src/sync.rs b/projects/score/cli/src/sync.rs
new file mode 100644
index 00000000..382b6ccd
--- /dev/null
+++ b/projects/score/cli/src/sync.rs
@@ -0,0 +1,62 @@
+//! `score sync` — auto-discover project/workspace hierarchy and write `.score/projects.toml`.
+
+use anyhow::bail;
+use clap::Args;
+use sdd::Result;
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R7
+/// Arguments for `score sync`.
+#[derive(Args, Debug)]
+pub struct SyncArgs {
+    /// Print unified diff of what would change without writing the file.
+    #[arg(long)]
+    pub dry_run: bool,
+
+    /// Like --dry-run but exits with code 1 when the diff is non-empty; suitable for CI.
+    #[arg(long)]
+    pub check: bool,
+}
+
+// @spec .score/changes/enhancement-score-sync-auto-discovered-project-workspace-regis/specs/enhancement-score-sync-auto-discovered-project-workspace-regis-spec.md#R1
+/// Run `score sync`.
+///
+/// - Default (no flags): discover, write projects.toml, print summary.
+/// - `--dry-run`: print diff only, don't write.
+/// - `--check`: print diff, return `Err` (exit 1) if non-empty.
+pub fn run(args: SyncArgs) -> Result<()> {
+    let root = crate::find_project_root()?;
+
+    if args.dry_run || args.check {
+        // Compute drift without writing
+        let diff = sdd::services::project_registry::check_drift(&root)?;
+        match diff {
+            None => {
+                println!("projects.toml is up to date — no changes.");
+            }
+            Some(d) => {
+                println!("{}", d);
+                if args.check {
+                    bail!("drift detected: projects.toml is out of date (exit 1)");
+                }
+            }
+        }
+    } else {
+        // Default: discover and write
+        let projects = sdd::services::project_discovery::discover_projects(&root)?;
+        let count = projects.len();
+        sdd::services::project_registry::write_projects_toml(&root, &projects)?;
+        println!(
+            "score sync: wrote .score/projects.toml with {} project(s).",
+            count
+        );
+        for p in &projects {
+            println!(
+                "  {} ({} workspace(s))",
+                p.name,
+                p.workspaces.len()
+            );
+        }
+    }
+
+    Ok(())
+}

```

## Review: enhancement-score-sync-auto-discovered-project-workspace-regis-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: enhancement-score-sync-auto-discovered-project-workspace-regis

**Summary**: Iteration 2 review: revision addresses all blocking findings from iteration 1. R6 [defaults.workspace] fallback is now wired — load_projects applies defaults.workspace.codegen to workspaces with no codegen; write_projects_toml reads existing defaults from disk and preserves them across round-trip. T11 merge_both_with_override now asserts per-field priority (target auto value retained when omitted from config). sync::run replaced process::exit(1) with anyhow::bail so the error propagates through run_command -> main cleanly. 4 new tests added (merge_defaults_workspace_fallback, merge_defaults_does_not_override_explicit_codegen, write_projects_toml_preserves_defaults, write_projects_toml_with_explicit_defaults). cargo build -p sdd and -p score-cli succeed; cargo test -p sdd --lib passes 1621/1621. Hard checklist: all pass. Remaining iteration-1 low-severity items (schema divergence from classDiagram, tech_stack.rs Changes-section doc drift, REQ-NNN test annotations, rule E multi-nested-Cargo edge case, non-unified diff format) are acknowledged and scoped to R12 follow-up issues per the spec's own out-of-scope declaration.

