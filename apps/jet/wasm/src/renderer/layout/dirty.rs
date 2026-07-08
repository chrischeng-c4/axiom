// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
// CODEGEN-BEGIN
//! Dirty-subtree marking algorithm.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/layout-runtime.md#logic
//!
//! Pure helper functions used by the impure dirty-marking phase
//! (R3) that runs BEFORE `layout()`. The algorithm is straightforward:
//!
//! 1. Caller marks one or more leaf-level nodes dirty (via
//!    `LayoutTree::mark_dirty`).
//! 2. Before calling `layout()`, run `propagate_ancestors` to walk
//!    each dirty node's parent chain, adding ancestors to the dirty
//!    set so taffy receives `set_style`/`set_children` updates from
//!    leaves up to the root.
//! 3. `layout()` then applies the accumulated dirty work and clears
//!    the set.
//!
//! Per spec: viewport changes force a full recompute, which is
//! handled inside `layout()` (it calls `taffy::mark_dirty` on the
//! root node when the viewport differs from the last call). This
//! module is responsible only for the per-node dirty-set propagation.

use std::collections::{HashMap, HashSet};

use super::{LayoutNodeId, LayoutTree};

/// Walk each dirty node's parent chain in `tree` and add every
/// ancestor to the dirty set. Idempotent — re-running with the same
/// input produces the same set.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
pub fn propagate_ancestors(tree: &mut LayoutTree) {
    let parent_map = tree.parent_map();
    let mut to_add: HashSet<LayoutNodeId> = HashSet::new();
    for id in &tree.dirty_nodes {
        let mut cursor = id.clone();
        while let Some(parent) = parent_map.get(&cursor) {
            if !tree.dirty_nodes.contains(parent) && !to_add.contains(parent) {
                to_add.insert(parent.clone());
            }
            cursor = parent.clone();
        }
    }
    for p in to_add {
        tree.dirty_nodes.insert(p);
    }
}

/// Pure helper: given a parent map and a set of leaf dirty IDs,
/// return the closure of dirty IDs (leaves + every ancestor).
///
/// Useful for unit-testing the algorithm without standing up a
/// full `LayoutTree`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
pub fn closure_with_ancestors(
    parent_map: &HashMap<LayoutNodeId, LayoutNodeId>,
    seed: &HashSet<LayoutNodeId>,
) -> HashSet<LayoutNodeId> {
    let mut out = seed.clone();
    for id in seed {
        let mut cursor = id.clone();
        while let Some(parent) = parent_map.get(&cursor) {
            if !out.insert(parent.clone()) {
                break;
            }
            cursor = parent.clone();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> LayoutNodeId {
        LayoutNodeId::new(s)
    }

    #[test]
    fn closure_walks_to_root() {
        // Tree:
        //   root → mid → leaf
        let mut parent_map: HashMap<LayoutNodeId, LayoutNodeId> = HashMap::new();
        parent_map.insert(id("leaf"), id("mid"));
        parent_map.insert(id("mid"), id("root"));

        let mut seed: HashSet<LayoutNodeId> = HashSet::new();
        seed.insert(id("leaf"));

        let out = closure_with_ancestors(&parent_map, &seed);
        assert!(out.contains(&id("leaf")));
        assert!(out.contains(&id("mid")));
        assert!(out.contains(&id("root")));
    }

    #[test]
    fn closure_idempotent_on_root() {
        let parent_map: HashMap<LayoutNodeId, LayoutNodeId> = HashMap::new();
        let mut seed: HashSet<LayoutNodeId> = HashSet::new();
        seed.insert(id("root"));
        let out = closure_with_ancestors(&parent_map, &seed);
        assert_eq!(out.len(), 1);
        assert!(out.contains(&id("root")));
    }

    #[test]
    fn closure_handles_two_branches() {
        // Tree:
        //   root → a → leafA
        //         → b → leafB
        let mut parent_map: HashMap<LayoutNodeId, LayoutNodeId> = HashMap::new();
        parent_map.insert(id("leafA"), id("a"));
        parent_map.insert(id("a"), id("root"));
        parent_map.insert(id("leafB"), id("b"));
        parent_map.insert(id("b"), id("root"));

        let mut seed: HashSet<LayoutNodeId> = HashSet::new();
        seed.insert(id("leafA"));
        seed.insert(id("leafB"));

        let out = closure_with_ancestors(&parent_map, &seed);
        assert_eq!(out.len(), 5);
        assert!(out.contains(&id("a")));
        assert!(out.contains(&id("b")));
        assert!(out.contains(&id("root")));
    }
}
// CODEGEN-END
