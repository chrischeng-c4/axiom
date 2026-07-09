// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
//! Unit tests for NxProjectGraph and NxWorkspaceManager.
//!
//! Loaded by `nx.rs` under `#[cfg(test)]`.

use super::*;
use std::collections::HashMap;
use tempfile::tempdir;

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------

fn make_graph(nodes: &[&str], edges: &[(&str, &str)]) -> NxProjectGraph {
    let node_map: HashMap<String, NxProject> = nodes
        .iter()
        .map(|&name| {
            (
                name.to_string(),
                NxProject {
                    name: name.to_string(),
                    project_type: None,
                    data: None,
                },
            )
        })
        .collect();

    let mut dep_map: HashMap<String, Vec<NxDependency>> = HashMap::new();
    for &(source, target) in edges {
        dep_map
            .entry(source.to_string())
            .or_default()
            .push(NxDependency {
                source: source.to_string(),
                target: target.to_string(),
                dep_type: Some("static".to_string()),
            });
    }

    NxProjectGraph {
        graph: NxGraphData {
            nodes: node_map,
            dependencies: dep_map,
        },
    }
}

// ------------------------------------------------------------------
// NxProjectGraph tests
// ------------------------------------------------------------------

#[test]
fn test_topological_sort_no_deps() {
    let graph = make_graph(&["a", "b", "c"], &[]);
    let order = graph.topological_sort();
    // All three nodes must appear; order is alphabetical when no deps exist.
    assert_eq!(order.len(), 3);
    assert_eq!(order, vec!["a", "b", "c"]);
}

#[test]
fn test_topological_sort_linear_chain() {
    // c depends on b, b depends on a → build order: a, b, c
    let graph = make_graph(&["a", "b", "c"], &[("b", "a"), ("c", "b")]);
    let order = graph.topological_sort();
    assert_eq!(order, vec!["a", "b", "c"]);
}

#[test]
fn test_topological_sort_diamond() {
    // b and c both depend on a; d depends on both b and c.
    // Valid orders: a, b, c, d  or  a, c, b, d
    let graph = make_graph(
        &["a", "b", "c", "d"],
        &[("b", "a"), ("c", "a"), ("d", "b"), ("d", "c")],
    );
    let order = graph.topological_sort();
    assert_eq!(order.len(), 4);

    let pos = |name: &str| order.iter().position(|x| x == name).unwrap();
    assert!(pos("a") < pos("b"), "a must come before b");
    assert!(pos("a") < pos("c"), "a must come before c");
    assert!(pos("b") < pos("d"), "b must come before d");
    assert!(pos("c") < pos("d"), "c must come before d");
}

#[test]
fn test_topological_sort_independent_projects() {
    // Four independent projects – order is alphabetical.
    let graph = make_graph(&["delta", "alpha", "gamma", "beta"], &[]);
    let order = graph.topological_sort();
    assert_eq!(order, vec!["alpha", "beta", "delta", "gamma"]);
}

#[test]
fn test_project_names_sorted() {
    let graph = make_graph(&["z-lib", "a-app", "m-core"], &[]);
    let names = graph.project_names();
    assert_eq!(names, vec!["a-app", "m-core", "z-lib"]);
}

#[test]
fn test_project_root() {
    let mut nodes = HashMap::new();
    nodes.insert(
        "my-lib".to_string(),
        NxProject {
            name: "my-lib".to_string(),
            project_type: Some("lib".to_string()),
            data: Some(NxProjectData {
                root: Some("libs/my-lib".to_string()),
                targets: HashMap::new(),
            }),
        },
    );
    let graph = NxProjectGraph {
        graph: NxGraphData {
            nodes,
            dependencies: HashMap::new(),
        },
    };

    assert_eq!(graph.project_root("my-lib"), Some("libs/my-lib"));
    assert_eq!(graph.project_root("nonexistent"), None);
}

// ------------------------------------------------------------------
// NxProjectGraph JSON parsing
// ------------------------------------------------------------------

#[test]
fn test_nx_graph_json_parse_minimal() {
    let json = r#"
    {
        "graph": {
            "nodes": {
                "app": {
                    "name": "app",
                    "type": "app",
                    "data": {
                        "root": "apps/app",
                        "targets": {}
                    }
                }
            },
            "dependencies": {
                "app": []
            }
        }
    }
    "#;

    let graph: NxProjectGraph = serde_json::from_str(json).unwrap();
    assert_eq!(graph.graph.nodes.len(), 1);
    assert!(graph.graph.nodes.contains_key("app"));
    assert_eq!(
        graph.graph.nodes["app"].data.as_ref().unwrap().root,
        Some("apps/app".to_string())
    );
}

#[test]
fn test_nx_graph_json_parse_with_deps() {
    let json = r#"
    {
        "graph": {
            "nodes": {
                "ui": { "name": "ui", "type": "lib", "data": null },
                "app": { "name": "app", "type": "app", "data": null }
            },
            "dependencies": {
                "app": [
                    { "source": "app", "target": "ui", "type": "static" }
                ]
            }
        }
    }
    "#;

    let graph: NxProjectGraph = serde_json::from_str(json).unwrap();
    let order = graph.topological_sort();
    let pos_ui = order.iter().position(|x| x == "ui").unwrap();
    let pos_app = order.iter().position(|x| x == "app").unwrap();
    assert!(pos_ui < pos_app, "ui must be built before app");
}

// ------------------------------------------------------------------
// NxWorkspaceManager discovery tests
// ------------------------------------------------------------------

#[test]
fn test_discover_no_nx_json() {
    let dir = tempdir().unwrap();
    let result = NxWorkspaceManager::discover(dir.path()).unwrap();
    assert!(
        result.is_none(),
        "Should return None when nx.json is absent"
    );
}

#[test]
fn test_discover_with_minimal_nx_json() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();

    let result = NxWorkspaceManager::discover(dir.path()).unwrap();
    assert!(result.is_some(), "Should return Some when nx.json exists");

    let mgr = result.unwrap();
    assert_eq!(mgr.root, dir.path());
}

#[test]
fn test_discover_with_full_nx_json() {
    let dir = tempdir().unwrap();
    let nx_json = r#"
    {
        "affected": { "defaultBase": "main" },
        "tasksRunnerOptions": {
            "default": {
                "runner": "nx/tasks-runners/default",
                "options": { "cacheableOperations": ["build", "test"] }
            }
        },
        "targetDefaults": {
            "build": { "dependsOn": ["^build"] }
        }
    }
    "#;
    std::fs::write(dir.path().join("nx.json"), nx_json).unwrap();

    let result = NxWorkspaceManager::discover(dir.path()).unwrap();
    assert!(result.is_some());

    let mgr = result.unwrap();
    assert!(
        mgr.config.target_defaults.contains_key("build"),
        "target_defaults should contain 'build'"
    );
}

#[test]
fn test_discover_malformed_nx_json_returns_error() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), "THIS IS NOT JSON").unwrap();

    let result = NxWorkspaceManager::discover(dir.path());
    assert!(
        result.is_err(),
        "Should return Err when nx.json is malformed"
    );
}

// ------------------------------------------------------------------
// NxWorkspaceManager::build_project_graph_from_files tests (R6)
// ------------------------------------------------------------------

fn write_project_json(
    root: &std::path::Path,
    rel_dir: &str,
    name: &str,
    project_type: &str,
    implicit_deps: &[&str],
) {
    let dir = root.join(rel_dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dep_json = serde_json::to_string(implicit_deps).unwrap();
    let content = format!(
        r#"{{
  "name": "{name}",
  "projectType": "{project_type}",
  "targets": {{}},
  "implicitDependencies": {dep_json}
}}"#,
        name = name,
        project_type = project_type,
        dep_json = dep_json,
    );
    std::fs::write(dir.join("project.json"), content).unwrap();
}

fn write_pkg_json(root: &std::path::Path, rel_dir: &str, pkg_json: &str) {
    let dir = root.join(rel_dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package.json"), pkg_json).unwrap();
}

/// Single project workspace: one project.json, no edges.
#[test]
fn test_build_graph_single_project() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "apps/web", "web", "application", &[]);

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    assert_eq!(graph.graph.nodes.len(), 1, "Expected 1 project");
    assert!(graph.graph.nodes.contains_key("web"));

    let node = &graph.graph.nodes["web"];
    assert_eq!(node.project_type.as_deref(), Some("application"));
    assert_eq!(
        node.data.as_ref().unwrap().root.as_deref(),
        Some("apps/web")
    );
}

/// Two projects with an implicit dependency edge.
#[test]
fn test_build_graph_implicit_dependency_edge() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "libs/shared", "shared", "library", &[]);
    write_project_json(dir.path(), "apps/web", "web", "application", &["shared"]);

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    assert_eq!(graph.graph.nodes.len(), 2);

    let web_deps = graph.graph.dependencies.get("web").unwrap();
    assert!(
        web_deps.iter().any(|d| d.target == "shared"),
        "web should depend on shared; deps: {:?}",
        web_deps
    );

    // Topological order: shared before web.
    let order = graph.topological_sort();
    let pos_shared = order.iter().position(|x| x == "shared").unwrap();
    let pos_web = order.iter().position(|x| x == "web").unwrap();
    assert!(pos_shared < pos_web, "shared must come before web");
}

/// Three projects: app → ui-lib → core.  Linear implicit dep chain.
#[test]
fn test_build_graph_three_project_chain() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "libs/core", "core", "library", &[]);
    write_project_json(dir.path(), "libs/ui-lib", "ui-lib", "library", &["core"]);
    write_project_json(dir.path(), "apps/app", "app", "application", &["ui-lib"]);

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();
    assert_eq!(graph.graph.nodes.len(), 3);

    let order = graph.topological_sort();
    let pos = |name: &str| order.iter().position(|x| x == name).unwrap();
    assert!(pos("core") < pos("ui-lib"), "core before ui-lib");
    assert!(pos("ui-lib") < pos("app"), "ui-lib before app");
}

/// Workspace-protocol dependency in package.json creates an edge.
#[test]
fn test_build_graph_workspace_protocol_edge() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "libs/utils", "utils", "library", &[]);
    write_project_json(dir.path(), "apps/api", "api", "application", &[]);

    // api/package.json references utils via workspace: protocol.
    write_pkg_json(
        dir.path(),
        "apps/api",
        r#"{"name":"api","version":"1.0.0","dependencies":{"utils":"workspace:*"}}"#,
    );

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    let api_deps = graph.graph.dependencies.get("api").unwrap();
    assert!(
        api_deps.iter().any(|d| d.target == "utils"),
        "api should depend on utils via workspace: protocol; deps: {:?}",
        api_deps
    );
}

/// project.json without explicit `name` field falls back to directory name.
#[test]
fn test_build_graph_project_name_fallback_to_dirname() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();

    // Write project.json without a `name` field.
    let proj_dir = dir.path().join("packages").join("my-lib");
    std::fs::create_dir_all(&proj_dir).unwrap();
    std::fs::write(
        proj_dir.join("project.json"),
        r#"{"projectType":"library","targets":{}}"#,
    )
    .unwrap();

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    assert_eq!(graph.graph.nodes.len(), 1);
    // Falls back to the directory name "my-lib".
    assert!(
        graph.graph.nodes.contains_key("my-lib"),
        "Expected project named 'my-lib' from directory; got {:?}",
        graph.graph.nodes.keys().collect::<Vec<_>>()
    );
}

/// node_modules directories are not traversed.
#[test]
fn test_build_graph_skips_node_modules() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "apps/real-app", "real-app", "application", &[]);

    // Fake project.json inside node_modules (must not be picked up).
    let nm_dir = dir.path().join("node_modules").join("some-pkg");
    std::fs::create_dir_all(&nm_dir).unwrap();
    std::fs::write(
        nm_dir.join("project.json"),
        r#"{"name":"phantom","projectType":"library"}"#,
    )
    .unwrap();

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    assert!(
        !graph.graph.nodes.contains_key("phantom"),
        "node_modules/project.json must not be included"
    );
    assert!(graph.graph.nodes.contains_key("real-app"));
}

/// Malformed project.json returns an error.
#[test]
fn test_build_graph_malformed_project_json_returns_error() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();

    let proj_dir = dir.path().join("apps/broken");
    std::fs::create_dir_all(&proj_dir).unwrap();
    std::fs::write(proj_dir.join("project.json"), "THIS IS NOT JSON").unwrap();

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let result = mgr.build_project_graph_from_files();
    assert!(
        result.is_err(),
        "Should return error for malformed project.json"
    );
}

/// project_names() returns sorted list from file-based graph.
#[test]
fn test_build_graph_project_names_sorted() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "z/z-pkg", "z-pkg", "library", &[]);
    write_project_json(dir.path(), "a/a-pkg", "a-pkg", "library", &[]);
    write_project_json(dir.path(), "m/m-pkg", "m-pkg", "library", &[]);

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    assert_eq!(graph.project_names(), vec!["a-pkg", "m-pkg", "z-pkg"]);
}

// ------------------------------------------------------------------
// GH #3261 — workspace:* edge handling when package.json is missing
// or malformed must not silently drop edges from sibling projects.
// ------------------------------------------------------------------

/// Source project has no package.json: no panic, no workspace edge
/// from that project. Sibling projects' edges still resolve.
#[test]
fn test_build_graph_workspace_edge_missing_pkg_json_is_silent() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "libs/utils", "utils", "library", &[]);
    write_project_json(dir.path(), "apps/api", "api", "application", &[]);
    write_project_json(dir.path(), "apps/web", "web", "application", &[]);

    // Only `api` has a package.json; `web` and `utils` do not.
    write_pkg_json(
        dir.path(),
        "apps/api",
        r#"{"name":"api","version":"1.0.0","dependencies":{"utils":"workspace:*"}}"#,
    );

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    // `api` -> `utils` edge resolves.
    let api_deps = graph.graph.dependencies.get("api").unwrap();
    assert!(
        api_deps.iter().any(|d| d.target == "utils"),
        "api should still depend on utils; deps: {:?}",
        api_deps
    );
    // `web` has no package.json: no edges, no panic.
    assert!(
        graph.graph.dependencies.get("web").is_none()
            || graph.graph.dependencies.get("web").unwrap().is_empty(),
        "web should have no edges; deps: {:?}",
        graph.graph.dependencies.get("web"),
    );
}

/// Source project has malformed package.json: graph still builds,
/// that project's workspace edges are absent (and a warn fires —
/// we don't capture tracing in unit tests). Other projects'
/// workspace edges resolve normally.
#[test]
fn test_build_graph_workspace_edge_malformed_pkg_json_surfaces_warn() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "libs/utils", "utils", "library", &[]);
    write_project_json(dir.path(), "apps/api", "api", "application", &[]);
    write_project_json(dir.path(), "apps/broken", "broken", "application", &[]);

    // Healthy package.json on `api`.
    write_pkg_json(
        dir.path(),
        "apps/api",
        r#"{"name":"api","version":"1.0.0","dependencies":{"utils":"workspace:*"}}"#,
    );
    // Malformed package.json on `broken`. Pre-#3261 this was
    // `unwrap_or_default()` + `if let Ok(...)` so the edge would
    // silently disappear; post-fix the read+parse failure logs warn
    // and skips just this project.
    write_pkg_json(dir.path(), "apps/broken", "THIS IS NOT JSON");

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr
        .build_project_graph_from_files()
        .expect("malformed package.json must not abort graph build");

    // `api` -> `utils` edge resolves (sanity-check: unaffected projects keep edges).
    let api_deps = graph.graph.dependencies.get("api").unwrap();
    assert!(
        api_deps.iter().any(|d| d.target == "utils"),
        "api should still depend on utils; deps: {:?}",
        api_deps
    );
    // `broken` has no resolvable workspace edges.
    assert!(
        graph.graph.dependencies.get("broken").is_none()
            || graph.graph.dependencies.get("broken").unwrap().is_empty(),
        "broken's workspace edges should be absent; deps: {:?}",
        graph.graph.dependencies.get("broken"),
    );
}

/// Workspace-protocol edges via `devDependencies` and `peerDependencies`
/// also resolve. Confirms the fix didn't regress non-`dependencies` fields.
#[test]
fn test_build_graph_workspace_edge_via_dev_and_peer_deps() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "libs/utils", "utils", "library", &[]);
    write_project_json(dir.path(), "libs/types", "types", "library", &[]);
    write_project_json(dir.path(), "apps/api", "api", "application", &[]);

    write_pkg_json(
        dir.path(),
        "apps/api",
        r#"{
  "name":"api",
  "version":"1.0.0",
  "devDependencies":{"utils":"workspace:*"},
  "peerDependencies":{"types":"workspace:*"}
}"#,
    );

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    let api_deps = graph.graph.dependencies.get("api").unwrap();
    assert!(
        api_deps.iter().any(|d| d.target == "utils"),
        "devDependencies workspace edge missing; deps: {:?}",
        api_deps
    );
    assert!(
        api_deps.iter().any(|d| d.target == "types"),
        "peerDependencies workspace edge missing; deps: {:?}",
        api_deps
    );
}

// ----- GH #3292 regression tests for walk_for_project_json dirent errors -----

/// Happy path: nested project.json files at multiple depths are all
/// discovered. Pins that the new `tracing::warn!` arm did not regress
/// the success path.
#[test]
fn test_walk_for_project_json_discovers_nested_projects() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "apps/web", "web", "application", &[]);
    write_project_json(dir.path(), "libs/core", "core", "library", &[]);
    write_project_json(
        dir.path(),
        "libs/ui/components",
        "ui-components",
        "library",
        &[],
    );

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    assert_eq!(graph.graph.nodes.len(), 3, "expected 3 projects discovered");
    assert!(graph.graph.nodes.contains_key("web"));
    assert!(graph.graph.nodes.contains_key("core"));
    assert!(graph.graph.nodes.contains_key("ui-components"));
}

/// GH #3292 — an unreadable subdirectory must not abort discovery and
/// must not silently swallow sibling subtrees. The walk continues past
/// the offending dirent and still discovers neighbouring project.json
/// files.
#[cfg(unix)]
#[test]
fn test_walk_for_project_json_unreadable_subdir_keeps_siblings() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("nx.json"), r#"{}"#).unwrap();
    write_project_json(dir.path(), "apps/web", "web", "application", &[]);
    write_project_json(dir.path(), "libs/core", "core", "library", &[]);

    // A subdir whose contents (its project.json + sub-subtree) become
    // unreadable. The directory entry itself is still listed by the
    // parent's read_dir, but `path.is_dir()` may fail; either way the
    // walk must not abort.
    let locked = dir.path().join("locked");
    std::fs::create_dir_all(&locked).unwrap();
    write_project_json(dir.path(), "locked/inside", "inside", "library", &[]);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

    // Skip cleanly when running as root (chmod is a no-op).
    if std::fs::read_dir(&locked).is_ok() {
        let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));
        return;
    }

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph_result = mgr.build_project_graph_from_files();

    // Restore perms so tempdir cleanup works.
    let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));

    let graph = graph_result.expect("walk must not abort on a single unreadable subtree");
    assert!(
        graph.graph.nodes.contains_key("web"),
        "healthy sibling 'web' must still be discovered: {:?}",
        graph.graph.nodes.keys().collect::<Vec<_>>()
    );
    assert!(
        graph.graph.nodes.contains_key("core"),
        "healthy sibling 'core' must still be discovered: {:?}",
        graph.graph.nodes.keys().collect::<Vec<_>>()
    );
    assert!(
        !graph.graph.nodes.contains_key("inside"),
        "project beneath an unreadable subtree must not appear (only siblings survive)"
    );
}

// ─── GH #3765 derive_rel_root branching ─────────────────────────────────

/// GH #3765 — `project.json` at the workspace root yields `""` (the
/// legitimate "this is the workspace root" signal). Stays silent.
#[test]
fn gh3765_rel_root_at_workspace_root_returns_empty() {
    use std::path::Path;
    let workspace = Path::new("/tmp/ws");
    let project_json = Path::new("/tmp/ws/project.json");
    let rel = derive_rel_root(project_json, workspace);
    assert_eq!(rel, "");
}

/// GH #3765 — nested project yields the strip-prefix result.
#[test]
fn gh3765_rel_root_nested_project_returns_relative_path() {
    use std::path::Path;
    let workspace = Path::new("/tmp/ws");
    let project_json = Path::new("/tmp/ws/apps/web/project.json");
    let rel = derive_rel_root(project_json, workspace);
    assert_eq!(rel, "apps/web");
}

/// GH #3765 — a path outside the workspace returns `""` and would warn
/// (test the empty fallback; warn capture would require a tracing
/// subscriber).
#[test]
fn gh3765_rel_root_outside_workspace_returns_empty() {
    use std::path::Path;
    let workspace = Path::new("/tmp/ws");
    let project_json = Path::new("/tmp/elsewhere/project.json");
    let rel = derive_rel_root(project_json, workspace);
    assert_eq!(rel, "");
}

/// GH #3765 — non-UTF-8 rel_root recovers via lossy decode rather than
/// dropping to `""`. Test on Linux only because APFS rejects non-UTF-8
/// at the FS layer; we drive `Path` directly to avoid filesystem
/// creation.
#[cfg(unix)]
#[test]
fn gh3765_rel_root_non_utf8_recovers_via_lossy() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::{Path, PathBuf};

    let workspace = Path::new("/tmp/ws");
    let mut bad = PathBuf::from("/tmp/ws/apps/");
    bad.push(OsStr::from_bytes(b"bad\xFFname"));
    bad.push("project.json");

    let rel = derive_rel_root(&bad, workspace);
    // U+FFFD substitution → still produces a path that includes "bad" + "name"
    assert!(rel.starts_with("apps/"));
    assert!(rel.contains("bad"));
    assert!(rel.contains("name"));
}

/// GH #3765 — all three helper messages include the issue tag for grep
/// discoverability.
#[test]
fn gh3765_helpers_include_issue_tag() {
    use std::path::Path;
    let p = Path::new("/tmp/x/project.json");
    let ws = Path::new("/tmp/ws");
    assert!(format_pkg_manager_nx_rel_root_no_parent_warn(p).contains("GH #3765"));
    assert!(format_pkg_manager_nx_rel_root_outside_workspace_warn(p, ws).contains("GH #3765"));
    assert!(format_pkg_manager_nx_rel_root_non_utf8_warn(p, "lossy").contains("GH #3765"));
}

/// GH #3765 — sibling-distinctness vs. prior nx warns (#3261 read warn,
/// #3292 dirent warn, #3751 dep-version warn): each branch's message
/// is distinguishable by the failure-mode wording.
#[test]
fn gh3765_helpers_distinct_from_prior_nx_warns() {
    use std::path::Path;
    let p = Path::new("/tmp/x/project.json");
    let ws = Path::new("/tmp/ws");
    let no_parent = format_pkg_manager_nx_rel_root_no_parent_warn(p);
    let outside = format_pkg_manager_nx_rel_root_outside_workspace_warn(p, ws);
    let non_utf8 = format_pkg_manager_nx_rel_root_non_utf8_warn(p, "lossy");

    // Each helper's message uniquely identifies its branch.
    assert!(no_parent.contains("has no parent directory"));
    assert!(outside.contains("not under workspace root"));
    assert!(non_utf8.contains("non-UTF-8 relative path"));

    // None of them collide with the read-failure or dirent warns.
    for msg in [&no_parent, &outside, &non_utf8] {
        assert!(!msg.contains("GH #3261"));
        assert!(!msg.contains("GH #3292"));
        assert!(!msg.contains("GH #3751"));
    }
}

/// GH #3765 — the no_parent message records the offending path so the
/// operator can chase it without re-running with debug logging.
#[test]
fn gh3765_no_parent_message_records_path() {
    use std::path::Path;
    let p = Path::new("/oddly/rooted/project.json");
    let msg = format_pkg_manager_nx_rel_root_no_parent_warn(p);
    assert!(msg.contains("/oddly/rooted/project.json"));
}

/// GH #3765 — the outside-workspace message records BOTH the offending
/// project.json path and the configured workspace root so the operator
/// can spot relative-vs-absolute mismatches.
#[test]
fn gh3765_outside_workspace_message_records_both_paths() {
    use std::path::Path;
    let p = Path::new("/elsewhere/project.json");
    let ws = Path::new("/intended/ws");
    let msg = format_pkg_manager_nx_rel_root_outside_workspace_warn(p, ws);
    assert!(msg.contains("/elsewhere/project.json"));
    assert!(msg.contains("/intended/ws"));
}

/// GH #3765 — the non-UTF-8 message includes the lossy recovery so
/// operators see the U+FFFD substitution that landed in the graph.
#[test]
fn gh3765_non_utf8_message_records_lossy() {
    use std::path::Path;
    let p = Path::new("/tmp/ws/apps/bad/project.json");
    let msg = format_pkg_manager_nx_rel_root_non_utf8_warn(p, "apps/\u{FFFD}name");
    assert!(msg.contains("apps/\u{FFFD}name"));
}

/// GH #3765 — helper naming convention. All three follow
/// `format_pkg_manager_nx_rel_root_*_warn` so the warn family is
/// grep-able as a set.
#[test]
fn gh3765_helper_naming_convention_discoverable() {
    use std::path::Path;
    let p = Path::new("/tmp/project.json");
    let ws = Path::new("/tmp");
    // If any of these helpers were renamed, this file would no longer
    // compile — the test asserts the convention via use-site.
    let _ = format_pkg_manager_nx_rel_root_no_parent_warn(p);
    let _ = format_pkg_manager_nx_rel_root_outside_workspace_warn(p, ws);
    let _ = format_pkg_manager_nx_rel_root_non_utf8_warn(p, "");
}

// ─── GH #3772 derive_project_name_from_dir branching ───────────────────

/// GH #3772 — UTF-8 directory name passes through silently
/// (preserves current behavior).
#[test]
fn gh3772_utf8_dir_name_returns_dir_name() {
    use std::path::Path;
    let p = Path::new("/tmp/ws/apps/web/project.json");
    let name = derive_project_name_from_dir(p);
    assert_eq!(name.as_deref(), Some("web"));
}

/// GH #3772 — no parent (project.json at root) returns None so the
/// caller can fall back further.
#[test]
fn gh3772_no_parent_returns_none() {
    use std::path::Path;
    let p = Path::new("project.json");
    // "project.json".parent() == Some("") which has no file_name.
    let name = derive_project_name_from_dir(p);
    assert!(name.is_none());
}

/// GH #3772 — non-UTF-8 directory name recovers via lossy decode and
/// would emit a warn (test the lossy fallback; warn capture would
/// require a tracing subscriber).
#[cfg(unix)]
#[test]
fn gh3772_non_utf8_dir_name_recovers_via_lossy() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::PathBuf;

    let mut bad = PathBuf::from("/tmp/ws/apps/");
    bad.push(OsStr::from_bytes(b"bad\xFFname"));
    bad.push("project.json");

    let name = derive_project_name_from_dir(&bad).expect("lossy fallback must produce a name");
    assert!(name.contains("bad"));
    assert!(name.contains("name"));
}

/// GH #3772 — issue-tag discoverability.
#[test]
fn gh3772_helper_includes_issue_tag() {
    use std::path::Path;
    let p = Path::new("/tmp/x/project.json");
    assert!(format_pkg_manager_nx_project_name_non_utf8_warn(p, "lossy").contains("GH #3772"));
}

/// GH #3772 — sibling-distinctness vs. GH #3765 rel_root family. Both
/// concern the same file and the same lossy fallback, but the messages
/// must let operators grep separately for "which field collided".
#[test]
fn gh3772_warn_distinct_from_gh3765_rel_root_family() {
    use std::path::Path;
    let p = Path::new("/tmp/x/project.json");
    let rel_root_msg = format_pkg_manager_nx_rel_root_non_utf8_warn(p, "lossy");
    let name_msg = format_pkg_manager_nx_project_name_non_utf8_warn(p, "lossy");

    assert!(rel_root_msg.contains("GH #3765"));
    assert!(name_msg.contains("GH #3772"));
    // Distinct subject in the wording.
    assert!(rel_root_msg.contains("non-UTF-8 relative path"));
    assert!(name_msg.contains("parent directory contains non-UTF-8"));
}

/// GH #3772 — message records both the offending project.json and the
/// lossy recovery.
#[test]
fn gh3772_message_records_path_and_lossy() {
    use std::path::Path;
    let p = Path::new("/elsewhere/project.json");
    let msg = format_pkg_manager_nx_project_name_non_utf8_warn(p, "bad\u{FFFD}name");
    assert!(msg.contains("/elsewhere/project.json"));
    assert!(msg.contains("bad\u{FFFD}name"));
}

/// GH #3772 — message names the collision risk so operators
/// understand the operational impact, not just the symptom.
#[test]
fn gh3772_message_names_collision_risk() {
    use std::path::Path;
    let p = Path::new("/tmp/project.json");
    let msg = format_pkg_manager_nx_project_name_non_utf8_warn(p, "x");
    assert!(msg.contains("collide"));
    assert!(msg.contains("shadow"));
}

/// GH #3772 — helper-name convention is discoverable. If the helper is
/// ever renamed, this file would fail to compile — the test asserts
/// the convention via use-site.
#[test]
fn gh3772_helper_naming_convention_discoverable() {
    use std::path::Path;
    let p = Path::new("/tmp/project.json");
    let _ = derive_project_name_from_dir(p);
    let _ = format_pkg_manager_nx_project_name_non_utf8_warn(p, "x");
}
// CODEGEN-END
