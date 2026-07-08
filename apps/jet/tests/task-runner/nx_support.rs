// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for Nx workspace detection and task execution.
//!
//! These tests exercise the public API surface of the Nx support layer:
//! workspace detection via `WorkspaceMode::detect`, `NxWorkspaceManager::discover`,
//! and project-graph topological ordering.

use jet::pkg_manager::nx::{
    NxDependency, NxGraphData, NxProject, NxProjectGraph, NxWorkspaceManager,
};
use jet::pkg_manager::workspace::WorkspaceMode;
use std::collections::HashMap;
use tempfile::tempdir;

// ------------------------------------------------------------------
// Helper: write arbitrary files inside a temp directory
// ------------------------------------------------------------------

fn create_file(base: &std::path::Path, rel_path: &str, content: &str) {
    let full = base.join(rel_path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(full, content).unwrap();
}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------

fn write_file(dir: &std::path::Path, name: &str, contents: &str) {
    std::fs::write(dir.join(name), contents).unwrap();
}

fn make_nx_graph(nodes: &[&str], edges: &[(&str, &str)]) -> NxProjectGraph {
    let node_map: HashMap<String, NxProject> = nodes
        .iter()
        .map(|&n| {
            (
                n.to_string(),
                NxProject {
                    name: n.to_string(),
                    project_type: None,
                    data: None,
                },
            )
        })
        .collect();

    let mut dep_map: HashMap<String, Vec<NxDependency>> = HashMap::new();
    for &(src, tgt) in edges {
        dep_map
            .entry(src.to_string())
            .or_default()
            .push(NxDependency {
                source: src.to_string(),
                target: tgt.to_string(),
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
// WorkspaceMode::detect
// ------------------------------------------------------------------

#[test]
fn test_detect_nx_workspace_when_nx_json_present() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "nx.json", r#"{"affected": {}}"#);

    let mode = WorkspaceMode::detect(dir.path()).unwrap();
    assert!(
        matches!(mode, WorkspaceMode::Nx(_)),
        "Expected WorkspaceMode::Nx when nx.json is present"
    );
}

#[test]
fn test_detect_jet_workspace_when_package_json_has_workspaces() {
    let dir = tempdir().unwrap();
    // No nx.json, but package.json declares workspaces.
    write_file(
        dir.path(),
        "package.json",
        r#"{"name":"root","version":"1.0.0","workspaces":["packages/*"]}"#,
    );

    let mode = WorkspaceMode::detect(dir.path()).unwrap();
    assert!(
        matches!(mode, WorkspaceMode::Jet(_)),
        "Expected WorkspaceMode::Jet when package.json workspaces is set"
    );
}

#[test]
fn test_detect_single_when_no_workspace_config() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "package.json",
        r#"{"name":"app","version":"1.0.0"}"#,
    );

    let mode = WorkspaceMode::detect(dir.path()).unwrap();
    assert!(
        matches!(mode, WorkspaceMode::Single),
        "Expected WorkspaceMode::Single when no workspace config present"
    );
}

#[test]
fn test_detect_nx_takes_priority_over_package_json_workspaces() {
    let dir = tempdir().unwrap();
    // Both nx.json and package.json workspaces present → Nx wins.
    write_file(dir.path(), "nx.json", r#"{}"#);
    write_file(
        dir.path(),
        "package.json",
        r#"{"name":"root","version":"1.0.0","workspaces":["packages/*"]}"#,
    );

    let mode = WorkspaceMode::detect(dir.path()).unwrap();
    assert!(
        matches!(mode, WorkspaceMode::Nx(_)),
        "Nx mode must take priority over Jet workspace mode"
    );
}

#[test]
fn test_detect_jet_workspace_from_yaml() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "jet-workspace.yaml",
        "packages:\n  - packages/*\n",
    );

    let mode = WorkspaceMode::detect(dir.path()).unwrap();
    assert!(
        matches!(mode, WorkspaceMode::Jet(_)),
        "Expected WorkspaceMode::Jet when jet-workspace.yaml is present"
    );
}

#[test]
fn test_detect_returns_error_for_malformed_nx_json() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "nx.json", "THIS_IS_NOT_JSON");

    let result = WorkspaceMode::detect(dir.path());
    assert!(
        result.is_err(),
        "Should propagate parse error for malformed nx.json"
    );
}

// ------------------------------------------------------------------
// NxWorkspaceManager discovery
// ------------------------------------------------------------------

#[test]
fn test_nx_manager_stores_root_path() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "nx.json", r#"{}"#);

    let mgr = NxWorkspaceManager::discover(dir.path())
        .unwrap()
        .expect("Should find NxWorkspaceManager");
    assert_eq!(mgr.root, dir.path());
}

#[test]
fn test_nx_manager_returns_none_without_nx_json() {
    let dir = tempdir().unwrap();
    let result = NxWorkspaceManager::discover(dir.path()).unwrap();
    assert!(result.is_none());
}

// ------------------------------------------------------------------
// NxProjectGraph: topological ordering via public API
// ------------------------------------------------------------------

#[test]
fn test_graph_topological_sort_respects_dependencies() {
    // app depends on shared; shared has no deps.
    let graph = make_nx_graph(&["shared", "app"], &[("app", "shared")]);
    let order = graph.topological_sort();

    let pos_shared = order.iter().position(|x| x == "shared").unwrap();
    let pos_app = order.iter().position(|x| x == "app").unwrap();
    assert!(
        pos_shared < pos_app,
        "shared must be built before app; order was {:?}",
        order
    );
}

#[test]
fn test_graph_includes_all_projects_in_sort() {
    let graph = make_nx_graph(&["alpha", "beta", "gamma", "delta"], &[]);
    let order = graph.topological_sort();
    assert_eq!(order.len(), 4, "All four projects must appear in the sort");
}

#[test]
fn test_graph_project_names_returns_sorted_list() {
    let graph = make_nx_graph(&["z-pkg", "a-pkg", "m-pkg"], &[]);
    assert_eq!(graph.project_names(), vec!["a-pkg", "m-pkg", "z-pkg"]);
}

#[test]
fn test_graph_json_roundtrip() {
    let graph = make_nx_graph(&["core", "api"], &[("api", "core")]);
    let json = serde_json::to_string(&graph).unwrap();
    let parsed: NxProjectGraph = serde_json::from_str(&json).unwrap();

    let order = parsed.topological_sort();
    let pos_core = order.iter().position(|x| x == "core").unwrap();
    let pos_api = order.iter().position(|x| x == "api").unwrap();
    assert!(pos_core < pos_api);
}

// ------------------------------------------------------------------
// R6: Direct workspace parsing — no nx CLI invocation (integration)
// ------------------------------------------------------------------

/// Build a realistic Nx monorepo fixture with project.json files and assert
/// that `build_project_graph_from_files` identifies all projects correctly
/// without spawning any external `nx` process.
#[test]
fn test_workspace_discovery_reads_project_json_files_directly() {
    let dir = tempdir().unwrap();

    // nx.json — marks this as an Nx workspace.
    create_file(dir.path(), "nx.json", r#"{"affected":{}}"#);

    // Two libraries and one application.
    create_file(
        dir.path(),
        "libs/core/project.json",
        r#"{"name":"core","projectType":"library","targets":{},"implicitDependencies":[]}"#,
    );
    create_file(
        dir.path(),
        "libs/ui/project.json",
        r#"{"name":"ui","projectType":"library","targets":{},"implicitDependencies":["core"]}"#,
    );
    create_file(
        dir.path(),
        "apps/web/project.json",
        r#"{"name":"web","projectType":"application","targets":{},"implicitDependencies":["core","ui"]}"#,
    );

    let mgr = NxWorkspaceManager::discover(dir.path())
        .unwrap()
        .expect("NxWorkspaceManager must be found");

    // File-based graph discovery — no nx process is spawned.
    let graph = mgr
        .build_project_graph_from_files()
        .expect("build_project_graph_from_files must succeed");

    // All three projects discovered.
    assert_eq!(graph.graph.nodes.len(), 3, "Expected 3 projects");
    for name in &["core", "ui", "web"] {
        assert!(
            graph.graph.nodes.contains_key(*name),
            "Project '{}' not found in graph",
            name
        );
    }

    // Dependency edges are correct.
    let web_deps = &graph.graph.dependencies["web"];
    assert!(
        web_deps.iter().any(|d| d.target == "core"),
        "web must depend on core"
    );
    assert!(
        web_deps.iter().any(|d| d.target == "ui"),
        "web must depend on ui"
    );

    let ui_deps = &graph.graph.dependencies["ui"];
    assert!(
        ui_deps.iter().any(|d| d.target == "core"),
        "ui must depend on core"
    );
}

/// Workspace-protocol edges in package.json are picked up by file parser.
#[test]
fn test_workspace_discovery_resolves_workspace_protocol_deps() {
    let dir = tempdir().unwrap();

    create_file(dir.path(), "nx.json", r#"{}"#);
    create_file(
        dir.path(),
        "libs/shared/project.json",
        r#"{"name":"shared","projectType":"library","targets":{}}"#,
    );
    create_file(
        dir.path(),
        "apps/server/project.json",
        r#"{"name":"server","projectType":"application","targets":{}}"#,
    );
    // server/package.json lists shared via workspace: protocol.
    create_file(
        dir.path(),
        "apps/server/package.json",
        r#"{"name":"server","version":"0.0.1","dependencies":{"shared":"workspace:*"}}"#,
    );

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    let server_deps = graph.graph.dependencies.get("server").unwrap();
    assert!(
        server_deps.iter().any(|d| d.target == "shared"),
        "server must depend on shared via workspace: protocol"
    );
}

/// node_modules directories inside the workspace are not traversed.
#[test]
fn test_workspace_discovery_ignores_node_modules() {
    let dir = tempdir().unwrap();

    create_file(dir.path(), "nx.json", r#"{}"#);
    create_file(
        dir.path(),
        "apps/real/project.json",
        r#"{"name":"real","projectType":"application","targets":{}}"#,
    );
    // A fake project.json inside node_modules must not be discovered.
    create_file(
        dir.path(),
        "node_modules/phantom/project.json",
        r#"{"name":"phantom","projectType":"library"}"#,
    );

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    assert!(
        !graph.graph.nodes.contains_key("phantom"),
        "phantom from node_modules must not appear in the graph"
    );
    assert!(graph.graph.nodes.contains_key("real"));
}

/// Topological order produced by file-based graph respects dep edges.
#[test]
fn test_workspace_discovery_topological_order() {
    let dir = tempdir().unwrap();

    create_file(dir.path(), "nx.json", r#"{}"#);
    create_file(
        dir.path(),
        "libs/base/project.json",
        r#"{"name":"base","projectType":"library","targets":{}}"#,
    );
    create_file(
        dir.path(),
        "libs/mid/project.json",
        r#"{"name":"mid","projectType":"library","targets":{},"implicitDependencies":["base"]}"#,
    );
    create_file(
        dir.path(),
        "apps/top/project.json",
        r#"{"name":"top","projectType":"application","targets":{},"implicitDependencies":["mid"]}"#,
    );

    let mgr = NxWorkspaceManager::discover(dir.path()).unwrap().unwrap();
    let graph = mgr.build_project_graph_from_files().unwrap();

    let order = graph.topological_sort();
    let pos = |name: &str| order.iter().position(|x| x == name).unwrap();

    assert!(pos("base") < pos("mid"), "base must precede mid");
    assert!(pos("mid") < pos("top"), "mid must precede top");
}
// CODEGEN-END
