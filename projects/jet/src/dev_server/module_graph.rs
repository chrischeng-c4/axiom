// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
use std::collections::{HashMap, HashSet, VecDeque};

/// A node in the module dependency graph.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone)]
pub struct ModuleGraphNode {
    /// Module URL path (e.g. `/src/App.tsx`).
    pub url: String,
    /// Absolute filesystem path.
    pub file: String,
    /// URLs of modules this module imports.
    pub imports: HashSet<String>,
    /// URLs of modules that import this module.
    pub importers: HashSet<String>,
    /// Module called `import.meta.hot.accept()` with no deps (self-accepting).
    pub is_self_accepting: bool,
    /// URLs of deps accepted via `import.meta.hot.accept(deps, cb)`.
    pub accepted_deps: HashSet<String>,
    /// Module contains React Fast Refresh instrumentation.
    pub has_react_refresh: bool,
    /// Timestamp of the last transform.
    pub last_transform_timestamp: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl ModuleGraphNode {
    fn new(url: &str, file: &str) -> Self {
        Self {
            url: url.to_string(),
            file: file.to_string(),
            imports: HashSet::new(),
            importers: HashSet::new(),
            is_self_accepting: false,
            accepted_deps: HashSet::new(),
            has_react_refresh: false,
            last_transform_timestamp: 0,
        }
    }
}

/// Result of an HMR boundary search.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum HmrBoundaryResult {
    /// Hot-update targets found — re-import these modules.
    HotUpdate { targets: Vec<String> },
    /// No boundary found — full page reload required.
    FullReload { reason: String },
}

/// Server-side module dependency graph.
///
/// Tracks import relationships as a directed graph and supports invalidation
/// walks to determine HMR update boundaries.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub struct ModuleGraph {
    nodes: HashMap<String, ModuleGraphNode>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Add or replace a module in the graph, establishing import edges.
    ///
    /// Returns the list of removed imports and newly added imports.
    pub fn add_module(
        &mut self,
        url: &str,
        file: &str,
        imports: &[String],
    ) -> (Vec<String>, Vec<String>) {
        let old_imports = if let Some(existing) = self.nodes.get(url) {
            existing.imports.clone()
        } else {
            HashSet::new()
        };

        let new_imports: HashSet<String> = imports.iter().cloned().collect();

        // Compute diff
        let removed: Vec<String> = old_imports.difference(&new_imports).cloned().collect();
        let added: Vec<String> = new_imports.difference(&old_imports).cloned().collect();

        // Remove stale importer edges
        for dep_url in &removed {
            if let Some(dep_node) = self.nodes.get_mut(dep_url) {
                dep_node.importers.remove(url);
            }
        }

        // Add new importer edges (ensure dep nodes exist)
        for dep_url in &added {
            let dep_node = self
                .nodes
                .entry(dep_url.clone())
                .or_insert_with(|| ModuleGraphNode::new(dep_url, ""));
            dep_node.importers.insert(url.to_string());
        }

        // Also ensure existing (unchanged) deps have importer edges
        for dep_url in new_imports.intersection(&old_imports) {
            if let Some(dep_node) = self.nodes.get_mut(dep_url) {
                dep_node.importers.insert(url.to_string());
            }
        }

        // Upsert the module node itself
        let node = self
            .nodes
            .entry(url.to_string())
            .or_insert_with(|| ModuleGraphNode::new(url, file));
        node.file = file.to_string();
        node.imports = imports.iter().cloned().collect();

        (removed, added)
    }

    /// Update a module's imports after re-transform, returning (removed, added).
    pub fn update_module(
        &mut self,
        url: &str,
        file: &str,
        new_imports: &[String],
    ) -> (Vec<String>, Vec<String>) {
        self.add_module(url, file, new_imports)
    }

    /// Mark a module as self-accepting (`import.meta.hot.accept()`).
    pub fn set_self_accepting(&mut self, url: &str, accepting: bool) {
        if let Some(node) = self.nodes.get_mut(url) {
            node.is_self_accepting = accepting;
        }
    }

    /// Set accepted deps for a module (`import.meta.hot.accept(deps, cb)`).
    pub fn set_accepted_deps(&mut self, url: &str, deps: HashSet<String>) {
        if let Some(node) = self.nodes.get_mut(url) {
            node.accepted_deps = deps;
        }
    }

    /// Mark a module as having React Fast Refresh instrumentation.
    pub fn set_has_react_refresh(&mut self, url: &str, has_refresh: bool) {
        if let Some(node) = self.nodes.get_mut(url) {
            node.has_react_refresh = has_refresh;
        }
    }

    /// Set the last transform timestamp on a module.
    pub fn set_timestamp(&mut self, url: &str, timestamp: u64) {
        if let Some(node) = self.nodes.get_mut(url) {
            node.last_transform_timestamp = timestamp;
        }
    }

    /// Remove a module from the graph and clean up edges.
    ///
    /// Returns the list of orphaned modules (imported by no one after removal).
    pub fn remove_module(&mut self, url: &str) -> Vec<String> {
        let mut orphans = Vec::new();

        if let Some(removed) = self.nodes.remove(url) {
            // Remove this module from importers lists of its imports
            for dep_url in &removed.imports {
                if let Some(dep_node) = self.nodes.get_mut(dep_url) {
                    dep_node.importers.remove(url);
                    if dep_node.importers.is_empty() {
                        orphans.push(dep_url.clone());
                    }
                }
            }

            // Remove this module from imports lists of its importers
            for importer_url in &removed.importers {
                if let Some(importer_node) = self.nodes.get_mut(importer_url) {
                    importer_node.imports.remove(url);
                }
            }
        }

        orphans
    }

    /// Walk the module graph upward from `changed_url` to find HMR boundaries.
    ///
    /// A boundary is:
    /// - A self-accepting module (`import.meta.hot.accept()`)
    /// - A module with React Fast Refresh instrumentation
    /// - A parent that accepts the changed dep (`import.meta.hot.accept([dep], cb)`)
    ///
    /// If no boundary is found before reaching the entry point, returns `FullReload`.
    pub fn find_hmr_boundary(&self, changed_url: &str) -> HmrBoundaryResult {
        let node = match self.nodes.get(changed_url) {
            Some(n) => n,
            None => {
                return HmrBoundaryResult::FullReload {
                    reason: format!("module not in graph: {}", changed_url),
                }
            }
        };

        // Case 1: Changed module is self-accepting
        if node.is_self_accepting {
            return HmrBoundaryResult::HotUpdate {
                targets: vec![changed_url.to_string()],
            };
        }

        // Case 2: Changed module has React Fast Refresh
        if node.has_react_refresh {
            return HmrBoundaryResult::HotUpdate {
                targets: vec![changed_url.to_string()],
            };
        }

        // Case 3: Walk upward through importers using BFS
        let mut targets = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Seed queue with (module_url, dep_that_changed_url)
        // The "dep" from the perspective of the parent is the module we came from.
        for importer_url in &node.importers {
            queue.push_back((importer_url.clone(), changed_url.to_string()));
        }
        visited.insert(changed_url.to_string());

        // If the module has no importers at all, it's likely an entry — full reload.
        if node.importers.is_empty() {
            return HmrBoundaryResult::FullReload {
                reason: format!("no HMR boundary for {}", changed_url),
            };
        }

        while let Some((current_url, dep_url)) = queue.pop_front() {
            if visited.contains(&current_url) {
                continue;
            }
            visited.insert(current_url.clone());

            let current = match self.nodes.get(&current_url) {
                Some(n) => n,
                None => continue,
            };

            // Check if this parent accepts the dep that changed
            if current.accepted_deps.contains(&dep_url) {
                targets.push(current_url.clone());
                continue;
            }

            // Check if parent is self-accepting
            if current.is_self_accepting {
                targets.push(current_url.clone());
                continue;
            }

            // Check if parent has React Fast Refresh
            if current.has_react_refresh {
                targets.push(current_url.clone());
                continue;
            }

            // No boundary here — keep walking up
            if current.importers.is_empty() {
                // Reached an entry point with no boundary
                return HmrBoundaryResult::FullReload {
                    reason: format!("no HMR boundary for {}", changed_url),
                };
            }

            for parent_url in &current.importers {
                queue.push_back((parent_url.clone(), current_url.clone()));
            }
        }

        if targets.is_empty() {
            HmrBoundaryResult::FullReload {
                reason: format!("no HMR boundary for {}", changed_url),
            }
        } else {
            HmrBoundaryResult::HotUpdate { targets }
        }
    }

    /// Transitive importers of `changed_url` — every module whose
    /// transform output may need to be invalidated when this URL's
    /// source changes. BFS over reverse `importers` edges.
    ///
    /// Distinct from `find_hmr_boundary`, which stops at the first
    /// boundary that can accept the update; this returns the full
    /// invalidation set the bundler-side cache layer must drop.
    /// The result excludes `changed_url` itself.
    ///
    /// All edges in `dev_server::module_graph` are static imports
    /// (the graph does not track `import()` boundaries here — those
    /// live on the client-runtime side), so there is no
    /// dynamic-import stop rule. The bundler-side
    /// `bundler::graph::ModuleGraph::dependents_of` enforces that
    /// stop separately.
    ///
    /// @spec `.aw/tech-design/projects/jet/logic/bundler-incremental-rebuild.md`
    ///   §"Invalidation graph contract" (dev-server half).
    /// @issue #1250 Slice 4b — R2 (dev-server reverse walker).
    pub fn dependents_of(&self, changed_url: &str) -> Vec<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut out: Vec<String> = Vec::new();

        seen.insert(changed_url.to_string());
        queue.push_back(changed_url.to_string());

        while let Some(url) = queue.pop_front() {
            let Some(node) = self.nodes.get(&url) else {
                continue;
            };
            for importer in &node.importers {
                if seen.insert(importer.clone()) {
                    out.push(importer.clone());
                    queue.push_back(importer.clone());
                }
            }
        }

        out
    }

    /// Get a reference to a module node.
    pub fn get(&self, url: &str) -> Option<&ModuleGraphNode> {
        self.nodes.get(url)
    }

    /// Get all module URLs in the graph.
    pub fn urls(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl Default for ModuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── T5: Graph Built From Imports ────────────────────────────────────────
    #[test]
    fn t5_graph_built_from_imports() {
        let mut graph = ModuleGraph::new();
        let imports = vec!["/src/B.tsx".to_string(), "/src/C.tsx".to_string()];
        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &imports);

        // A's imports contain B and C
        let node_a = graph.get("/src/A.tsx").expect("A must exist");
        assert!(node_a.imports.contains("/src/B.tsx"));
        assert!(node_a.imports.contains("/src/C.tsx"));

        // B's importers contain A
        let node_b = graph.get("/src/B.tsx").expect("B must exist");
        assert!(
            node_b.importers.contains("/src/A.tsx"),
            "B.importers must contain A"
        );

        // C's importers contain A
        let node_c = graph.get("/src/C.tsx").expect("C must exist");
        assert!(
            node_c.importers.contains("/src/A.tsx"),
            "C.importers must contain A"
        );
    }

    // ── T6: Graph Update Removes Stale Edges ────────────────────────────────
    #[test]
    fn t6_graph_update_removes_stale_edges() {
        let mut graph = ModuleGraph::new();

        // Initial: A imports B and C
        let imports_v1 = vec!["/src/B.tsx".to_string(), "/src/C.tsx".to_string()];
        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &imports_v1);

        // Update: A now imports C and D (removed B, added D)
        let imports_v2 = vec!["/src/C.tsx".to_string(), "/src/D.tsx".to_string()];
        let (removed, added) = graph.update_module("/src/A.tsx", "/abs/src/A.tsx", &imports_v2);

        // Check diff return values
        assert!(
            removed.contains(&"/src/B.tsx".to_string()),
            "B must be in removed list"
        );
        assert!(
            added.contains(&"/src/D.tsx".to_string()),
            "D must be in added list"
        );

        // B's importers no longer contains A
        let node_b = graph.get("/src/B.tsx").expect("B must still exist as node");
        assert!(
            !node_b.importers.contains("/src/A.tsx"),
            "B.importers must NOT contain A after update"
        );

        // D's importers contains A
        let node_d = graph.get("/src/D.tsx").expect("D must exist");
        assert!(
            node_d.importers.contains("/src/A.tsx"),
            "D.importers must contain A"
        );

        // C's importers still contains A (unchanged)
        let node_c = graph.get("/src/C.tsx").expect("C must exist");
        assert!(
            node_c.importers.contains("/src/A.tsx"),
            "C.importers must still contain A"
        );

        // A's imports are now C and D
        let node_a = graph.get("/src/A.tsx").expect("A must exist");
        assert!(node_a.imports.contains("/src/C.tsx"));
        assert!(node_a.imports.contains("/src/D.tsx"));
        assert!(!node_a.imports.contains("/src/B.tsx"));
    }

    // ── T7: Invalidation Walk Finds Self-Accept Boundary ────────────────────
    #[test]
    fn t7_invalidation_walk_finds_self_accept_boundary() {
        let mut graph = ModuleGraph::new();

        // A imports B
        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &[]);

        // B is self-accepting, A is not
        graph.set_self_accepting("/src/B.tsx", true);

        let result = graph.find_hmr_boundary("/src/B.tsx");
        match result {
            HmrBoundaryResult::HotUpdate { targets } => {
                assert_eq!(targets, vec!["/src/B.tsx".to_string()]);
            }
            HmrBoundaryResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    // ── T8: Invalidation Walk Propagates to Parent ──────────────────────────
    #[test]
    fn t8_invalidation_walk_propagates_to_parent() {
        let mut graph = ModuleGraph::new();

        // B imports C
        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &["/src/C.tsx".to_string()]);
        graph.add_module("/src/C.tsx", "/abs/src/C.tsx", &[]);

        // C has no accept handler, no React components
        // B accepts C as a dependency
        let mut deps = HashSet::new();
        deps.insert("/src/C.tsx".to_string());
        graph.set_accepted_deps("/src/B.tsx", deps);

        let result = graph.find_hmr_boundary("/src/C.tsx");
        match result {
            HmrBoundaryResult::HotUpdate { targets } => {
                assert!(
                    targets.contains(&"/src/B.tsx".to_string()),
                    "B must be in targets as the parent that accepts C: {:?}",
                    targets
                );
            }
            HmrBoundaryResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    // ── T9: Invalidation Walk Returns FullReload ────────────────────────────
    #[test]
    fn t9_invalidation_walk_returns_full_reload() {
        let mut graph = ModuleGraph::new();

        // Chain: entry → A → B → C, none have accept handlers or React components
        graph.add_module(
            "/src/entry.tsx",
            "/abs/src/entry.tsx",
            &["/src/A.tsx".to_string()],
        );
        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &["/src/C.tsx".to_string()]);
        graph.add_module("/src/C.tsx", "/abs/src/C.tsx", &[]);

        let result = graph.find_hmr_boundary("/src/C.tsx");
        match result {
            HmrBoundaryResult::FullReload { reason } => {
                assert!(
                    reason.contains("no HMR boundary"),
                    "reason must mention 'no HMR boundary': {}",
                    reason
                );
            }
            HmrBoundaryResult::HotUpdate { targets } => {
                panic!("Expected FullReload but got HotUpdate: {:?}", targets);
            }
        }
    }

    // ── Additional: React Fast Refresh boundary ─────────────────────────────
    #[test]
    fn invalidation_walk_finds_react_refresh_boundary() {
        let mut graph = ModuleGraph::new();

        // A imports B
        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &[]);

        // B has React Fast Refresh (it's a React component)
        graph.set_has_react_refresh("/src/B.tsx", true);

        let result = graph.find_hmr_boundary("/src/B.tsx");
        match result {
            HmrBoundaryResult::HotUpdate { targets } => {
                assert_eq!(targets, vec!["/src/B.tsx".to_string()]);
            }
            HmrBoundaryResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    // ── Additional: Module removal returns orphans ──────────────────────────
    #[test]
    fn remove_module_returns_orphans() {
        let mut graph = ModuleGraph::new();

        // A imports B (B has no other importers)
        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &[]);

        let orphans = graph.remove_module("/src/A.tsx");
        assert!(
            orphans.contains(&"/src/B.tsx".to_string()),
            "B should be orphaned after A is removed"
        );
    }

    // ── Additional: Unknown module returns FullReload ────────────────────────
    #[test]
    fn find_boundary_unknown_module_returns_full_reload() {
        let graph = ModuleGraph::new();
        let result = graph.find_hmr_boundary("/src/unknown.tsx");
        match result {
            HmrBoundaryResult::FullReload { reason } => {
                assert!(reason.contains("not in graph"), "reason: {}", reason);
            }
            HmrBoundaryResult::HotUpdate { .. } => {
                panic!("Expected FullReload for unknown module");
            }
        }
    }

    // ── Additional: Self-accepting parent acts as boundary ──────────────────
    #[test]
    fn self_accepting_parent_acts_as_boundary() {
        let mut graph = ModuleGraph::new();

        // Parent imports child, parent is self-accepting
        graph.add_module(
            "/src/Parent.tsx",
            "/abs/src/Parent.tsx",
            &["/src/Child.tsx".to_string()],
        );
        graph.add_module("/src/Child.tsx", "/abs/src/Child.tsx", &[]);
        graph.set_self_accepting("/src/Parent.tsx", true);

        let result = graph.find_hmr_boundary("/src/Child.tsx");
        match result {
            HmrBoundaryResult::HotUpdate { targets } => {
                assert!(
                    targets.contains(&"/src/Parent.tsx".to_string()),
                    "Parent should be the boundary target: {:?}",
                    targets
                );
            }
            HmrBoundaryResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    // ── TR1/S1: Circular Import Chain — BFS Termination ─────────────────────
    #[test]
    fn test_circular_import_chain_bfs_termination() {
        let mut graph = ModuleGraph::new();

        // Build cycle: A→B (static), B→C (static), C→A (static)
        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &["/src/C.tsx".to_string()]);
        graph.add_module("/src/C.tsx", "/abs/src/C.tsx", &["/src/A.tsx".to_string()]);

        // None are self-accepting, no React Refresh
        let result = graph.find_hmr_boundary("/src/C.tsx");
        match result {
            HmrBoundaryResult::FullReload { reason } => {
                assert!(
                    reason.contains("no HMR boundary"),
                    "reason must mention 'no HMR boundary': {}",
                    reason
                );
            }
            HmrBoundaryResult::HotUpdate { targets } => {
                panic!("Expected FullReload but got HotUpdate: {:?}", targets);
            }
        }
    }

    // ── TR2/S2: Circular Import Chain with Boundary ─────────────────────────
    #[test]
    fn test_circular_import_chain_with_boundary() {
        let mut graph = ModuleGraph::new();

        // Build cycle: A→B (static), B→C (static), C→A (static)
        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &["/src/C.tsx".to_string()]);
        graph.add_module("/src/C.tsx", "/abs/src/C.tsx", &["/src/A.tsx".to_string()]);

        // B is self-accepting
        graph.set_self_accepting("/src/B.tsx", true);

        // Changing C: C's importer is B (cycle goes C→A but A imports B).
        // BFS from C: importer is A (C→A edge means A imports C? No.)
        // Wait — let me re-read the graph edges:
        //   A imports B → A.imports={B}, B.importers={A}
        //   B imports C → B.imports={C}, C.importers={B}
        //   C imports A → C.imports={A}, A.importers={C}
        // So C's importers = {B}. BFS visits B, which is self-accepting → target.
        let result = graph.find_hmr_boundary("/src/C.tsx");
        match result {
            HmrBoundaryResult::HotUpdate { targets } => {
                assert!(
                    targets.contains(&"/src/B.tsx".to_string()),
                    "B must be in targets as boundary: {:?}",
                    targets
                );
                assert!(
                    !targets.contains(&"/src/A.tsx".to_string()),
                    "A must NOT be in targets (B is the boundary, walk stops): {:?}",
                    targets
                );
            }
            HmrBoundaryResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    // ── TR3/S3: Diamond Graph Collects Multiple Targets ─────────────────────
    #[test]
    fn test_diamond_graph_multiple_targets() {
        let mut graph = ModuleGraph::new();

        // Diamond: entry→A, entry→B, A→util, B→util
        graph.add_module(
            "/src/entry.tsx",
            "/abs/src/entry.tsx",
            &["/src/A.tsx".to_string(), "/src/B.tsx".to_string()],
        );
        graph.add_module(
            "/src/A.tsx",
            "/abs/src/A.tsx",
            &["/src/util.ts".to_string()],
        );
        graph.add_module(
            "/src/B.tsx",
            "/abs/src/B.tsx",
            &["/src/util.ts".to_string()],
        );
        graph.add_module("/src/util.ts", "/abs/src/util.ts", &[]);

        // A and B are both self-accepting
        graph.set_self_accepting("/src/A.tsx", true);
        graph.set_self_accepting("/src/B.tsx", true);

        let result = graph.find_hmr_boundary("/src/util.ts");
        match result {
            HmrBoundaryResult::HotUpdate { targets } => {
                assert!(
                    targets.contains(&"/src/A.tsx".to_string()),
                    "A must be in targets: {:?}",
                    targets
                );
                assert!(
                    targets.contains(&"/src/B.tsx".to_string()),
                    "B must be in targets: {:?}",
                    targets
                );
                assert!(
                    !targets.contains(&"/src/entry.tsx".to_string()),
                    "entry must NOT be in targets: {:?}",
                    targets
                );
            }
            HmrBoundaryResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    // ── TR4/S4: Deep Chain BFS Traverses to Distant Boundary ────────────────
    #[test]
    fn test_deep_chain_bfs_traversal() {
        let mut graph = ModuleGraph::new();

        // Chain: entry→m1→m2→m3→m4→leaf
        graph.add_module(
            "/src/entry.tsx",
            "/abs/src/entry.tsx",
            &["/src/m1.tsx".to_string()],
        );
        graph.add_module(
            "/src/m1.tsx",
            "/abs/src/m1.tsx",
            &["/src/m2.tsx".to_string()],
        );
        graph.add_module(
            "/src/m2.tsx",
            "/abs/src/m2.tsx",
            &["/src/m3.tsx".to_string()],
        );
        graph.add_module(
            "/src/m3.tsx",
            "/abs/src/m3.tsx",
            &["/src/m4.tsx".to_string()],
        );
        graph.add_module(
            "/src/m4.tsx",
            "/abs/src/m4.tsx",
            &["/src/leaf.ts".to_string()],
        );
        graph.add_module("/src/leaf.ts", "/abs/src/leaf.ts", &[]);

        // Only m1 is self-accepting
        graph.set_self_accepting("/src/m1.tsx", true);

        let result = graph.find_hmr_boundary("/src/leaf.ts");
        match result {
            HmrBoundaryResult::HotUpdate { targets } => {
                assert!(
                    targets.contains(&"/src/m1.tsx".to_string()),
                    "m1 must be in targets: {:?}",
                    targets
                );
                assert!(
                    !targets.contains(&"/src/m2.tsx".to_string()),
                    "m2 must NOT be in targets: {:?}",
                    targets
                );
                assert!(
                    !targets.contains(&"/src/m3.tsx".to_string()),
                    "m3 must NOT be in targets: {:?}",
                    targets
                );
                assert!(
                    !targets.contains(&"/src/m4.tsx".to_string()),
                    "m4 must NOT be in targets: {:?}",
                    targets
                );
            }
            HmrBoundaryResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    // ── #1250 Slice 4b: dev-server reverse-dep walker ─────────────────────
    #[test]
    fn dependents_of_unknown_url_is_empty() {
        let graph = ModuleGraph::new();
        let deps = graph.dependents_of("/src/ghost.tsx");
        assert!(deps.is_empty());
    }

    #[test]
    fn dependents_of_excludes_self() {
        // entry imports leaf; dependents_of(leaf) returns [entry]
        // and never the leaf itself.
        let mut graph = ModuleGraph::new();
        graph.add_module(
            "/src/entry.tsx",
            "/abs/entry.tsx",
            &["/src/leaf.ts".to_string()],
        );
        graph.add_module("/src/leaf.ts", "/abs/leaf.ts", &[]);
        let deps = graph.dependents_of("/src/leaf.ts");
        assert_eq!(deps, vec!["/src/entry.tsx".to_string()]);
    }

    #[test]
    fn dependents_of_walks_chain_transitively() {
        // a → b → c. dependents_of(c) should yield {a, b}.
        let mut graph = ModuleGraph::new();
        graph.add_module("/a.ts", "/abs/a.ts", &["/b.ts".to_string()]);
        graph.add_module("/b.ts", "/abs/b.ts", &["/c.ts".to_string()]);
        graph.add_module("/c.ts", "/abs/c.ts", &[]);
        let mut deps = graph.dependents_of("/c.ts");
        deps.sort();
        assert_eq!(deps, vec!["/a.ts".to_string(), "/b.ts".to_string()]);
    }

    #[test]
    fn dependents_of_dedups_diamond() {
        // a imports b and c; both b and c import d.
        // dependents_of(d) yields {a, b, c} once each.
        let mut graph = ModuleGraph::new();
        graph.add_module(
            "/a.ts",
            "/abs/a.ts",
            &["/b.ts".to_string(), "/c.ts".to_string()],
        );
        graph.add_module("/b.ts", "/abs/b.ts", &["/d.ts".to_string()]);
        graph.add_module("/c.ts", "/abs/c.ts", &["/d.ts".to_string()]);
        graph.add_module("/d.ts", "/abs/d.ts", &[]);
        let mut deps = graph.dependents_of("/d.ts");
        deps.sort();
        assert_eq!(
            deps,
            vec![
                "/a.ts".to_string(),
                "/b.ts".to_string(),
                "/c.ts".to_string()
            ]
        );
    }

    #[test]
    fn dependents_of_walks_barrel_reexport_cascade() {
        // entry → barrel → leaf. Editing leaf must invalidate barrel
        // AND entry — the high-leverage barrel cascade.
        let mut graph = ModuleGraph::new();
        graph.add_module(
            "/src/entry.tsx",
            "/abs/entry.tsx",
            &["/src/barrel.ts".to_string()],
        );
        graph.add_module(
            "/src/barrel.ts",
            "/abs/barrel.ts",
            &["/src/leaf.ts".to_string()],
        );
        graph.add_module("/src/leaf.ts", "/abs/leaf.ts", &[]);
        let mut deps = graph.dependents_of("/src/leaf.ts");
        deps.sort();
        assert_eq!(
            deps,
            vec!["/src/barrel.ts".to_string(), "/src/entry.tsx".to_string()]
        );
    }
}
// CODEGEN-END
