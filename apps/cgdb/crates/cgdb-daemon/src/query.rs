// HANDWRITE-BEGIN gap="missing-generator:hand-written:90096f0c" tracker="2087" reason="coverage (bidirectional orphans) + impact (reverse-SpecRef BFS) query algorithms."
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

use anyhow::Result;
use cgdb_core::graph::{GraphPayload, NodeRecord, NodeType, RegionKind};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SpecOrphan {
    pub file: String,
    pub section: String,
}

#[derive(Debug, Serialize)]
pub struct CodeOrphan {
    pub file: String,
    pub symbol: String,
    pub region_kind: RegionKind,
}

#[derive(Debug, Serialize)]
pub struct CoverageResult {
    pub spec_total: usize,
    pub code_total: usize,
    pub spec_orphans: Vec<SpecOrphan>,
    pub code_orphans: Vec<CodeOrphan>,
}

#[derive(Debug, Serialize)]
pub struct AffectedNode {
    pub file: String,
    pub symbol: String,
    pub region_kind: RegionKind,
}

#[derive(Debug, Serialize)]
pub struct ImpactResult {
    pub spec_section: String,
    pub affected: Vec<AffectedNode>,
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

pub fn coverage(graph_path: &Path) -> Result<CoverageResult> {
    let records = cgdb_core::graph::scan(graph_path)?;
    let mut spec_nodes: BTreeMap<String, NodeRecord> = BTreeMap::new();
    let mut code_nodes: BTreeMap<String, NodeRecord> = BTreeMap::new();
    let mut incoming: BTreeSet<String> = BTreeSet::new();
    let mut outgoing: BTreeSet<String> = BTreeSet::new();
    for r in records {
        match r.payload {
            GraphPayload::Node(n) => match n.node_type {
                NodeType::Spec => {
                    spec_nodes.insert(n.id.clone(), n);
                }
                NodeType::Code => {
                    code_nodes.insert(n.id.clone(), n);
                }
            },
            GraphPayload::Edge(e) => {
                outgoing.insert(e.from);
                incoming.insert(e.to);
            }
        }
    }

    let mut spec_orphans: Vec<SpecOrphan> = spec_nodes
        .values()
        .filter(|n| !incoming.contains(&n.id))
        .map(|n| SpecOrphan { file: n.file.clone(), section: n.anchor.clone() })
        .collect();
    spec_orphans.sort_by(|a, b| (a.file.as_str(), a.section.as_str()).cmp(&(b.file.as_str(), b.section.as_str())));

    let mut code_orphans: Vec<CodeOrphan> = code_nodes
        .values()
        .filter(|n| !outgoing.contains(&n.id))
        .map(|n| CodeOrphan {
            file: n.file.clone(),
            symbol: n.symbol.clone().unwrap_or_default(),
            region_kind: n.region_kind.unwrap_or(RegionKind::Plain),
        })
        .collect();
    code_orphans.sort_by(|a, b| (a.file.as_str(), a.symbol.as_str()).cmp(&(b.file.as_str(), b.symbol.as_str())));

    Ok(CoverageResult {
        spec_total: spec_nodes.len(),
        code_total: code_nodes.len(),
        spec_orphans,
        code_orphans,
    })
}

pub fn impact(graph_path: &Path, selector: &str) -> Result<ImpactResult> {
    let records = cgdb_core::graph::scan(graph_path)?;
    let mut spec_id_for_selector: Option<String> = None;
    let (file, section) = match selector.split_once('#') {
        Some((f, s)) => (f, s),
        None => ("", selector),
    };
    let target_anchor_slug = slugify(section);

    let mut nodes: BTreeMap<String, NodeRecord> = BTreeMap::new();
    let mut reverse: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in records {
        match r.payload {
            GraphPayload::Node(n) => {
                if matches!(n.node_type, NodeType::Spec)
                    && (file.is_empty() || n.file == file)
                    && n.anchor == target_anchor_slug
                {
                    spec_id_for_selector = Some(n.id.clone());
                }
                nodes.insert(n.id.clone(), n);
            }
            GraphPayload::Edge(e) => {
                reverse.entry(e.to.clone()).or_default().push(e.from.clone());
            }
        }
    }

    let mut affected: Vec<AffectedNode> = Vec::new();
    if let Some(start) = spec_id_for_selector {
        let mut q: VecDeque<String> = VecDeque::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        q.push_back(start);
        while let Some(id) = q.pop_front() {
            if !seen.insert(id.clone()) {
                continue;
            }
            if let Some(from_ids) = reverse.get(&id) {
                for from in from_ids {
                    if seen.contains(from) {
                        continue;
                    }
                    if let Some(n) = nodes.get(from) {
                        if matches!(n.node_type, NodeType::Code) {
                            affected.push(AffectedNode {
                                file: n.file.clone(),
                                symbol: n.symbol.clone().unwrap_or_default(),
                                region_kind: n.region_kind.unwrap_or(RegionKind::Plain),
                            });
                        }
                    }
                    q.push_back(from.clone());
                }
            }
        }
    }
    affected.sort_by(|a, b| (a.file.as_str(), a.symbol.as_str()).cmp(&(b.file.as_str(), b.symbol.as_str())));
    affected.dedup_by(|a, b| a.file == b.file && a.symbol == b.symbol);

    Ok(ImpactResult { spec_section: selector.to_string(), affected })
}
// HANDWRITE-END
