// HANDWRITE-BEGIN gap="broken-generator:gen-code-stub-uncompilable" tracker="2092" reason="gen-code emitted lens dispatch as free-standing async fns with &self / undefined identifiers; replaced with a real LensView builder over the graph.jsonl store. Reopen codegen once the logic + rpc-api generators emit valid Rust into impl blocks."
//! Lens service: builds `LensView`s from a project's graph.jsonl per
//! `cgdb-v0-1.md#cgdb-v0-1-lens-dispatch`.
//!
//! v0.1 nodes carry an optional `level: u8`. When the indexer hasn't yet
//! assigned levels (`LEVEL_UNSPECIFIED == 255`), we infer level from
//! NodeType: Spec → 3 (section), Code → 3 (symbol). This is the pre-Phase-2
//! shape — once indexer.rs emits L0/L1 nodes (project/crate/spec-tree/dir),
//! the inference layer falls away.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::path::Path;

use anyhow::{anyhow, Result};
use cgdb_core::graph::{GraphPayload, NodeRecord, NodeType, LEVEL_UNSPECIFIED};
use cgdb_core::lens::{
    Breadcrumb, BreadcrumbEntry, LensEdge, LensEdgeSource, LensEdgeType, LensKind, LensNode,
    LensView, MAX_NODES,
};

struct GraphIndex {
    nodes: BTreeMap<String, NodeRecord>,
    out_edges: BTreeMap<String, Vec<String>>,
    in_edges: BTreeMap<String, Vec<String>>,
}

fn load(graph_path: &Path) -> Result<GraphIndex> {
    let records = cgdb_core::graph::scan(graph_path)?;
    let mut nodes = BTreeMap::new();
    let mut out_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut in_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in records {
        match r.payload {
            GraphPayload::Node(n) => {
                nodes.insert(n.id.clone(), n);
            }
            GraphPayload::Edge(e) => {
                out_edges.entry(e.from.clone()).or_default().push(e.to.clone());
                in_edges.entry(e.to.clone()).or_default().push(e.from.clone());
            }
        }
    }
    Ok(GraphIndex { nodes, out_edges, in_edges })
}

fn level_of(n: &NodeRecord) -> u8 {
    if n.level != LEVEL_UNSPECIFIED {
        return n.level;
    }
    match n.node_type {
        NodeType::Spec => 3,
        NodeType::Code => 3,
    }
}

fn kind_of(n: &NodeRecord) -> LensKind {
    match (n.node_type, level_of(n)) {
        (NodeType::Code, 0) => LensKind::Project,
        (NodeType::Code, 1) => LensKind::Crate,
        (NodeType::Code, 2) => LensKind::File,
        (NodeType::Code, _) => LensKind::Symbol,
        (NodeType::Spec, 0) => LensKind::ProjectSpecs,
        (NodeType::Spec, 1) => LensKind::Directory,
        (NodeType::Spec, 2) => LensKind::SpecFile,
        (NodeType::Spec, 4) => LensKind::Subsection,
        (NodeType::Spec, _) => LensKind::Section,
    }
}

fn label_of(n: &NodeRecord) -> String {
    match n.node_type {
        NodeType::Code => n.symbol.clone().unwrap_or_else(|| n.file.clone()),
        NodeType::Spec => {
            if n.anchor.is_empty() { n.file.clone() } else { n.anchor.clone() }
        }
    }
}

fn to_lens_node(n: &NodeRecord) -> LensNode {
    LensNode {
        id: n.id.clone(),
        level: level_of(n),
        kind: kind_of(n),
        label: label_of(n),
        section_type: None,
    }
}

fn explicit_edges(idx: &GraphIndex, keep: &BTreeSet<String>) -> Vec<LensEdge> {
    let mut edges = Vec::new();
    for (from, tos) in &idx.out_edges {
        if !keep.contains(from) {
            continue;
        }
        for to in tos {
            if !keep.contains(to) {
                continue;
            }
            edges.push(LensEdge {
                from: from.clone(),
                to: to.clone(),
                edge_type: LensEdgeType::SpecRef,
                source: LensEdgeSource::DocComment,
                weight: None,
            });
        }
    }
    edges
}

/// `lens.overview` — top-level (L0) aggregation. In the absence of indexer-assigned
/// L0 nodes, we synthesize one node per distinct `file` (L2) and bucket by directory.
pub fn overview(graph_path: &Path, project: &str) -> Result<LensView> {
    let idx = load(graph_path)?;
    let mut view = LensView::empty(project.to_string());

    // Group nodes by file → emit one L2 LensNode per file.
    let mut by_file: HashMap<String, (NodeType, u32)> = HashMap::new();
    for n in idx.nodes.values() {
        let entry = by_file.entry(n.file.clone()).or_insert((n.node_type, 0));
        entry.1 += 1;
    }
    for (file, (nt, _count)) in by_file {
        view.nodes.push(LensNode {
            id: file.clone(),
            level: 2,
            kind: match nt {
                NodeType::Code => LensKind::File,
                NodeType::Spec => LensKind::SpecFile,
            },
            label: file,
            section_type: None,
        });
    }
    // Synthesize Contains-style edges from edge endpoints' files.
    let mut seen_edges: BTreeSet<(String, String)> = BTreeSet::new();
    for (from, tos) in &idx.out_edges {
        if let Some(f_node) = idx.nodes.get(from) {
            for to in tos {
                if let Some(t_node) = idx.nodes.get(to) {
                    if f_node.file != t_node.file
                        && seen_edges.insert((f_node.file.clone(), t_node.file.clone()))
                    {
                        view.edges.push(LensEdge {
                            from: f_node.file.clone(),
                            to: t_node.file.clone(),
                            edge_type: LensEdgeType::SpecRef,
                            source: LensEdgeSource::DocComment,
                            weight: None,
                        });
                    }
                }
            }
        }
    }
    view.cap_at_max(project);
    Ok(view)
}

/// `lens.zoom_in` — expand a node to its level+1 children + intra-children edges.
pub fn zoom_in(graph_path: &Path, node_id: &str) -> Result<LensView> {
    let idx = load(graph_path)?;
    let center = idx
        .nodes
        .get(node_id)
        .ok_or_else(|| anyhow!("unknown node_id: {}", node_id))?
        .clone();
    let mut view = LensView::empty(node_id.to_string());
    view.nodes.push(to_lens_node(&center));

    // Children = nodes whose file starts with this node's file (best-effort
    // in absence of explicit Contains edges) or nodes directly referenced.
    let mut keep: BTreeSet<String> = BTreeSet::new();
    keep.insert(center.id.clone());
    if let Some(children) = idx.out_edges.get(&center.id) {
        for c in children {
            if let Some(n) = idx.nodes.get(c) {
                view.nodes.push(to_lens_node(n));
                keep.insert(n.id.clone());
            }
        }
    }
    // Pull intra-children edges from the explicit edge set.
    view.edges = explicit_edges(&idx, &keep);
    view.cap_at_max(&label_of(&center));
    Ok(view)
}

/// `lens.zoom_out` — collapse to parent. We approximate parent as the centre's file.
pub fn zoom_out(graph_path: &Path, node_id: &str) -> Result<LensView> {
    let idx = load(graph_path)?;
    let center = idx
        .nodes
        .get(node_id)
        .ok_or_else(|| anyhow!("unknown node_id: {}", node_id))?
        .clone();
    let parent_id = center.file.clone();
    let mut view = LensView::empty(parent_id.clone());
    view.nodes.push(LensNode {
        id: parent_id.clone(),
        level: level_of(&center).saturating_sub(1),
        kind: match center.node_type {
            NodeType::Code => LensKind::File,
            NodeType::Spec => LensKind::SpecFile,
        },
        label: parent_id.clone(),
        section_type: None,
    });
    // Siblings = nodes sharing the parent file.
    let mut keep: BTreeSet<String> = BTreeSet::new();
    keep.insert(parent_id.clone());
    for n in idx.nodes.values() {
        if n.file == parent_id {
            view.nodes.push(to_lens_node(n));
            keep.insert(n.id.clone());
        }
    }
    view.edges = explicit_edges(&idx, &keep);
    view.cap_at_max(&parent_id);
    Ok(view)
}

/// `lens.focus` — fixed-depth neighborhood BFS over explicit (and optionally
/// semantic) edges. Depth ∈ [1,3], capped to MAX_NODES.
pub fn focus(
    graph_path: &Path,
    node_id: &str,
    depth: u8,
    _include_semantic: bool,
) -> Result<LensView> {
    let idx = load(graph_path)?;
    let center = idx
        .nodes
        .get(node_id)
        .ok_or_else(|| anyhow!("unknown node_id: {}", node_id))?
        .clone();
    let mut view = LensView::empty(node_id.to_string());

    let mut keep: BTreeSet<String> = BTreeSet::new();
    let mut q: VecDeque<(String, u8)> = VecDeque::new();
    keep.insert(center.id.clone());
    q.push_back((center.id.clone(), 0));
    while let Some((cur, d)) = q.pop_front() {
        if d >= depth || keep.len() >= MAX_NODES {
            continue;
        }
        let mut neighbours: Vec<String> = Vec::new();
        if let Some(outs) = idx.out_edges.get(&cur) {
            neighbours.extend(outs.iter().cloned());
        }
        if let Some(ins) = idx.in_edges.get(&cur) {
            neighbours.extend(ins.iter().cloned());
        }
        for nbr in neighbours {
            if keep.insert(nbr.clone()) {
                q.push_back((nbr, d + 1));
            }
        }
    }
    for id in &keep {
        if let Some(n) = idx.nodes.get(id) {
            view.nodes.push(to_lens_node(n));
        }
    }
    view.edges = explicit_edges(&idx, &keep);
    view.cap_at_max(&label_of(&center));
    Ok(view)
}

/// `lens.breadcrumb` — walk parent chain to L0. v0.1 approximation walks
/// file→directory→project; deeper hierarchies follow once indexer.rs assigns
/// the real `level` field.
pub fn breadcrumb(graph_path: &Path, node_id: &str) -> Result<Breadcrumb> {
    let idx = load(graph_path)?;
    let node = idx
        .nodes
        .get(node_id)
        .ok_or_else(|| anyhow!("unknown node_id: {}", node_id))?;
    let mut path = Vec::new();
    // L0 — project (file root's first path segment).
    let root = node.file.split('/').next().unwrap_or("").to_string();
    if !root.is_empty() {
        path.push(BreadcrumbEntry { node_id: root.clone(), level: 0, label: root });
    }
    // L2 — file.
    path.push(BreadcrumbEntry {
        node_id: node.file.clone(),
        level: 2,
        label: node.file.clone(),
    });
    // L3+ — the node itself.
    path.push(BreadcrumbEntry {
        node_id: node.id.clone(),
        level: level_of(node),
        label: label_of(node),
    });
    Ok(Breadcrumb { path })
}
// HANDWRITE-END
