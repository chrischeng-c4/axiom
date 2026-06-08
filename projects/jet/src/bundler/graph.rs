// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::path::PathBuf;

/// Module identifier (graph node index)
pub type ModuleId = NodeIndex;

/// Module dependency graph
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub struct ModuleGraph {
    graph: DiGraph<ModuleNode, EdgeKind>,
    path_to_id: HashMap<PathBuf, ModuleId>,
}

/// Node in the module graph
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub path: PathBuf,
    pub kind: ModuleKind,
    pub size: u64,
}

/// Type of module
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    Script,
    Css,
    Json,
    Asset,
    Wasm,
}

/// Edge type in the dependency graph
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Import,
    DynamicImport,
    CssImport,
    WasmImport,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            path_to_id: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, path: PathBuf, kind: ModuleKind, size: u64) -> ModuleId {
        if let Some(&id) = self.path_to_id.get(&path) {
            return id;
        }

        let node = ModuleNode {
            path: path.clone(),
            kind,
            size,
        };
        let id = self.graph.add_node(node);
        self.path_to_id.insert(path, id);
        id
    }

    pub fn add_dependency(&mut self, from: ModuleId, to: ModuleId, kind: EdgeKind) {
        self.graph.add_edge(from, to, kind);
    }

    pub fn get_module(&self, path: &PathBuf) -> Option<ModuleId> {
        self.path_to_id.get(path).copied()
    }

    pub fn get_node(&self, id: ModuleId) -> Option<&ModuleNode> {
        self.graph.node_weight(id)
    }

    pub fn topological_sort(&self) -> Result<Vec<ModuleId>, Vec<PathBuf>> {
        use petgraph::algo::toposort;

        match toposort(&self.graph, None) {
            Ok(order) => Ok(order),
            Err(cycle) => {
                let cycle_node = cycle.node_id();
                let cycle_path = self
                    .graph
                    .node_weight(cycle_node)
                    .map(|n| n.path.clone())
                    .unwrap_or_default();

                tracing::warn!("Cycle detected in module graph at: {:?}", cycle_path);

                let cycle_paths = self.find_cycle_from(cycle_node);
                Err(cycle_paths)
            }
        }
    }

    pub fn find_cycle_from(&self, start: ModuleId) -> Vec<PathBuf> {
        use petgraph::Direction;

        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![start];
        let mut path = Vec::new();

        while let Some(node) = stack.last().copied() {
            if visited.contains(&node) {
                if let Some(pos) = path.iter().position(|&n| n == node) {
                    return path[pos..]
                        .iter()
                        .filter_map(|&id| self.graph.node_weight(id).map(|n| n.path.clone()))
                        .collect();
                }
                stack.pop();
                continue;
            }

            visited.insert(node);
            path.push(node);

            let mut has_unvisited = false;
            for neighbor in self.graph.neighbors_directed(node, Direction::Outgoing) {
                if !visited.contains(&neighbor) {
                    stack.push(neighbor);
                    has_unvisited = true;
                    break;
                }
            }

            if !has_unvisited {
                stack.pop();
            }
        }

        Vec::new()
    }

    pub fn has_cycle(&self) -> bool {
        use petgraph::algo::toposort;
        toposort(&self.graph, None).is_err()
    }

    pub fn dependencies(&self, id: ModuleId) -> Vec<(ModuleId, EdgeKind)> {
        use petgraph::Direction;

        self.graph
            .neighbors_directed(id, Direction::Outgoing)
            .map(|dep_id| {
                let edge = self.graph.find_edge(id, dep_id).unwrap();
                let kind = *self.graph.edge_weight(edge).unwrap();
                (dep_id, kind)
            })
            .collect()
    }

    /// Modules that import (directly or transitively) the given path.
    /// BFS walk over reverse `Import` / `CssImport` / `WasmImport`
    /// edges. **`DynamicImport` edges stop the walk** — every
    /// `import()` boundary is already an HMR root in the runtime
    /// client, so pushing invalidation past it just discards work the
    /// browser will re-fetch lazily. This is the rule that takes a
    /// 500-module-graph rebuild from "1.5 s" to "well under 100 ms"
    /// for the typical leaf-module change.
    ///
    /// The returned list does not include `path` itself. Returns
    /// empty if the path is not in the graph.
    ///
    /// @spec `.aw/tech-design/projects/jet/logic/bundler-incremental-rebuild.md`
    ///   §"Invalidation graph contract".
    /// @issue #1250 Slice 4 — R2.
    pub fn dependents_of(&self, path: &std::path::Path) -> Vec<PathBuf> {
        use petgraph::Direction;
        use std::collections::{HashSet, VecDeque};

        let Some(start) = self.path_to_id.get(&path.to_path_buf()).copied() else {
            return Vec::new();
        };

        let mut seen: HashSet<ModuleId> = HashSet::new();
        let mut queue: VecDeque<ModuleId> = VecDeque::new();
        queue.push_back(start);
        seen.insert(start);

        let mut out: Vec<PathBuf> = Vec::new();

        while let Some(node) = queue.pop_front() {
            for edge_ref in self.graph.edges_directed(node, Direction::Incoming) {
                let parent = petgraph::visit::EdgeRef::source(&edge_ref);
                let kind = *petgraph::visit::EdgeRef::weight(&edge_ref);
                if matches!(kind, EdgeKind::DynamicImport) {
                    continue;
                }
                if seen.insert(parent) {
                    if let Some(node_data) = self.graph.node_weight(parent) {
                        out.push(node_data.path.clone());
                    }
                    queue.push_back(parent);
                }
            }
        }

        out
    }

    pub fn module_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get all module IDs in graph insertion order.
    ///
    /// Used as a fallback ordering when topological sort fails due to cycles.
    /// Insertion order guarantees the entry point (first module added) is at
    /// index 0, which is required by `generate_bundle_with_runtime`.
    pub fn all_node_ids(&self) -> Vec<ModuleId> {
        self.graph.node_indices().collect()
    }

    pub fn clear(&mut self) {
        self.graph.clear();
        self.path_to_id.clear();
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
impl Default for ModuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_module() {
        let mut graph = ModuleGraph::new();
        let path = PathBuf::from("test.js");
        let id = graph.add_module(path.clone(), ModuleKind::Script, 100);

        assert_eq!(graph.module_count(), 1);
        assert_eq!(graph.get_module(&path), Some(id));
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = ModuleGraph::new();
        let id1 = graph.add_module(PathBuf::from("a.js"), ModuleKind::Script, 100);
        let id2 = graph.add_module(PathBuf::from("b.js"), ModuleKind::Script, 200);

        graph.add_dependency(id1, id2, EdgeKind::Import);

        let deps = graph.dependencies(id1);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, id2);
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = ModuleGraph::new();
        let id_a = graph.add_module(PathBuf::from("a.js"), ModuleKind::Script, 100);
        let id_b = graph.add_module(PathBuf::from("b.js"), ModuleKind::Script, 100);
        let id_c = graph.add_module(PathBuf::from("c.js"), ModuleKind::Script, 100);

        graph.add_dependency(id_a, id_b, EdgeKind::Import);
        graph.add_dependency(id_b, id_c, EdgeKind::Import);

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);

        let pos_a = sorted.iter().position(|&id| id == id_a).unwrap();
        let pos_b = sorted.iter().position(|&id| id == id_b).unwrap();
        let pos_c = sorted.iter().position(|&id| id == id_c).unwrap();

        println!("Sorted order: c={}, b={}, a={}", pos_c, pos_b, pos_a);

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = ModuleGraph::new();
        let id_a = graph.add_module(PathBuf::from("a.js"), ModuleKind::Script, 100);
        let id_b = graph.add_module(PathBuf::from("b.js"), ModuleKind::Script, 100);
        let id_c = graph.add_module(PathBuf::from("c.js"), ModuleKind::Script, 100);

        graph.add_dependency(id_a, id_b, EdgeKind::Import);
        graph.add_dependency(id_b, id_c, EdgeKind::Import);
        graph.add_dependency(id_c, id_a, EdgeKind::Import);

        assert!(graph.has_cycle());
        assert!(graph.topological_sort().is_err());
    }

    #[test]
    fn dependents_of_walks_reverse_chain() {
        // a → b → c (a imports b imports c).
        // dependents_of(c) should yield [b, a] (BFS order).
        let mut graph = ModuleGraph::new();
        let id_a = graph.add_module(PathBuf::from("a.js"), ModuleKind::Script, 100);
        let id_b = graph.add_module(PathBuf::from("b.js"), ModuleKind::Script, 100);
        let id_c = graph.add_module(PathBuf::from("c.js"), ModuleKind::Script, 100);

        graph.add_dependency(id_a, id_b, EdgeKind::Import);
        graph.add_dependency(id_b, id_c, EdgeKind::Import);

        let deps = graph.dependents_of(std::path::Path::new("c.js"));
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&PathBuf::from("a.js")));
        assert!(deps.contains(&PathBuf::from("b.js")));
    }

    #[test]
    fn dependents_of_excludes_self() {
        let mut graph = ModuleGraph::new();
        let id_a = graph.add_module(PathBuf::from("a.js"), ModuleKind::Script, 100);
        let id_b = graph.add_module(PathBuf::from("b.js"), ModuleKind::Script, 100);
        graph.add_dependency(id_a, id_b, EdgeKind::Import);

        let deps = graph.dependents_of(std::path::Path::new("b.js"));
        assert_eq!(deps, vec![PathBuf::from("a.js")]);
    }

    #[test]
    fn dependents_of_unknown_path_is_empty() {
        let graph = ModuleGraph::new();
        let deps = graph.dependents_of(std::path::Path::new("ghost.js"));
        assert!(deps.is_empty());
    }

    #[test]
    fn dependents_of_walks_barrel_reexport_cascade() {
        // entry → barrel → leaf; barrel re-exports a name from leaf.
        // Editing leaf must invalidate barrel AND entry — the spec's
        // "high-leverage barrel cascade" case.
        let mut graph = ModuleGraph::new();
        let entry = graph.add_module(PathBuf::from("entry.ts"), ModuleKind::Script, 100);
        let barrel = graph.add_module(PathBuf::from("barrel.ts"), ModuleKind::Script, 100);
        let leaf = graph.add_module(PathBuf::from("leaf.ts"), ModuleKind::Script, 100);
        graph.add_dependency(entry, barrel, EdgeKind::Import);
        graph.add_dependency(barrel, leaf, EdgeKind::Import);

        let deps = graph.dependents_of(std::path::Path::new("leaf.ts"));
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&PathBuf::from("barrel.ts")));
        assert!(deps.contains(&PathBuf::from("entry.ts")));
    }

    #[test]
    fn dependents_of_stops_at_dynamic_import_boundary() {
        // entry --DynamicImport--> lazy --Import--> shared.
        // dependents_of(shared) yields [lazy] but NOT entry — the
        // dynamic boundary halts the walk because every import() is
        // already an HMR root in the runtime client.
        let mut graph = ModuleGraph::new();
        let entry = graph.add_module(PathBuf::from("entry.js"), ModuleKind::Script, 100);
        let lazy = graph.add_module(PathBuf::from("lazy.js"), ModuleKind::Script, 100);
        let shared = graph.add_module(PathBuf::from("shared.js"), ModuleKind::Script, 100);
        graph.add_dependency(entry, lazy, EdgeKind::DynamicImport);
        graph.add_dependency(lazy, shared, EdgeKind::Import);

        let deps = graph.dependents_of(std::path::Path::new("shared.js"));
        assert_eq!(deps, vec![PathBuf::from("lazy.js")]);
    }

    #[test]
    fn dependents_of_handles_diamond_without_duplicates() {
        // a imports b and c; both b and c import d.
        // dependents_of(d) yields {a, b, c} once each.
        let mut graph = ModuleGraph::new();
        let id_a = graph.add_module(PathBuf::from("a.js"), ModuleKind::Script, 100);
        let id_b = graph.add_module(PathBuf::from("b.js"), ModuleKind::Script, 100);
        let id_c = graph.add_module(PathBuf::from("c.js"), ModuleKind::Script, 100);
        let id_d = graph.add_module(PathBuf::from("d.js"), ModuleKind::Script, 100);
        graph.add_dependency(id_a, id_b, EdgeKind::Import);
        graph.add_dependency(id_a, id_c, EdgeKind::Import);
        graph.add_dependency(id_b, id_d, EdgeKind::Import);
        graph.add_dependency(id_c, id_d, EdgeKind::Import);

        let mut deps = graph.dependents_of(std::path::Path::new("d.js"));
        deps.sort();
        assert_eq!(
            deps,
            vec![
                PathBuf::from("a.js"),
                PathBuf::from("b.js"),
                PathBuf::from("c.js"),
            ]
        );
    }

    #[test]
    fn dependents_of_walks_css_and_wasm_edges() {
        // Both CssImport and WasmImport are static dependencies and
        // must be walked (only DynamicImport stops).
        let mut graph = ModuleGraph::new();
        let entry = graph.add_module(PathBuf::from("entry.js"), ModuleKind::Script, 100);
        let style = graph.add_module(PathBuf::from("style.css"), ModuleKind::Css, 50);
        let blob = graph.add_module(PathBuf::from("mod.wasm"), ModuleKind::Wasm, 50);
        graph.add_dependency(entry, style, EdgeKind::CssImport);
        graph.add_dependency(entry, blob, EdgeKind::WasmImport);

        let css_deps = graph.dependents_of(std::path::Path::new("style.css"));
        assert_eq!(css_deps, vec![PathBuf::from("entry.js")]);
        let wasm_deps = graph.dependents_of(std::path::Path::new("mod.wasm"));
        assert_eq!(wasm_deps, vec![PathBuf::from("entry.js")]);
    }

    #[test]
    fn test_complex_graph() {
        let mut graph = ModuleGraph::new();
        let id_a = graph.add_module(PathBuf::from("a.js"), ModuleKind::Script, 100);
        let id_b = graph.add_module(PathBuf::from("b.js"), ModuleKind::Script, 100);
        let id_c = graph.add_module(PathBuf::from("c.js"), ModuleKind::Script, 100);
        let id_d = graph.add_module(PathBuf::from("d.js"), ModuleKind::Script, 100);

        graph.add_dependency(id_a, id_b, EdgeKind::Import);
        graph.add_dependency(id_a, id_c, EdgeKind::Import);
        graph.add_dependency(id_b, id_d, EdgeKind::Import);
        graph.add_dependency(id_c, id_d, EdgeKind::Import);

        assert!(!graph.has_cycle());

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 4);

        let pos_a = sorted.iter().position(|&id| id == id_a).unwrap();
        let pos_b = sorted.iter().position(|&id| id == id_b).unwrap();
        let pos_c = sorted.iter().position(|&id| id == id_c).unwrap();
        let pos_d = sorted.iter().position(|&id| id == id_d).unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }
}
// CODEGEN-END
