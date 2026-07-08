// HANDWRITE-BEGIN gap="missing-generator:gen-code-stub-replaced" tracker="2092" reason="LensView types + Mermaid renderer for cgdb v0.1 lens.* API; gen-code stub was a uniform RPC scaffold that did not reflect the real type shape."
//! Lens view types and Mermaid renderer per `cgdb-v0-1.md#rpc-api`.
//!
//! The `LensView` is the canonical response shape for every `lens.*` RPC.
//! It carries up to 50 nodes (R3 hard cap) plus an optional `summary` when
//! truncation happens. Two output paths share the same struct:
//! - `to_json()` → canonical JSON for agents
//! - `to_mermaid()` → dagre-shaped flowchart for humans

use serde::{Deserialize, Serialize};

pub const MAX_NODES: usize = 50;

/// Node-kind enum mirroring the openrpc `LensView.nodes[].kind` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LensKind {
    Project,
    Crate,
    File,
    Symbol,
    ProjectSpecs,
    Directory,
    SpecFile,
    Section,
    Subsection,
}

impl LensKind {
    pub fn level(self) -> u8 {
        match self {
            LensKind::Project | LensKind::ProjectSpecs => 0,
            LensKind::Crate | LensKind::Directory => 1,
            LensKind::File | LensKind::SpecFile => 2,
            LensKind::Symbol | LensKind::Section => 3,
            LensKind::Subsection => 4,
        }
    }

    pub fn plural_label(self) -> &'static str {
        match self {
            LensKind::Project => "projects",
            LensKind::Crate => "crates",
            LensKind::File => "files",
            LensKind::Symbol => "symbols",
            LensKind::ProjectSpecs => "spec-trees",
            LensKind::Directory => "directories",
            LensKind::SpecFile => "spec-files",
            LensKind::Section => "sections",
            LensKind::Subsection => "subsections",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LensNode {
    pub id: String,
    pub level: u8,
    pub kind: LensKind,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum LensEdgeType {
    SpecRef,
    SemanticSpecRef,
    Contains,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum LensEdgeSource {
    DocComment,
    CodegenBlock,
    HandwriteBlock,
    Inferred,
    Hierarchical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensEdge {
    pub from: String,
    pub to: String,
    pub edge_type: LensEdgeType,
    pub source: LensEdgeSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,
}

/// Overflow descriptor emitted when the view would exceed `MAX_NODES`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LensSummary {
    pub collapsed_count: u32,
    /// Format: "+{count} {kind} in {parent}" per config.lens.summary_format.
    pub collapsed_summary: String,
}

impl LensSummary {
    pub fn format(count: u32, kind: LensKind, parent: &str) -> Self {
        Self {
            collapsed_count: count,
            collapsed_summary: format!("+{} {} in {}", count, kind.plural_label(), parent),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensView {
    /// node_id at the focus of this view (project name for `lens.overview`).
    pub center: String,
    pub nodes: Vec<LensNode>,
    pub edges: Vec<LensEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<LensSummary>,
}

impl LensView {
    pub fn empty(center: impl Into<String>) -> Self {
        Self {
            center: center.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
            summary: None,
        }
    }

    /// Enforce R3: cap at `MAX_NODES`, keep highest-degree nodes (the center
    /// node is always kept), and synthesize a `LensSummary`.
    ///
    /// `parent_label` is the human-readable name shown after "in …" in the
    /// summary string; pick the center node's label or the project name.
    pub fn cap_at_max(&mut self, parent_label: &str) {
        if self.nodes.len() <= MAX_NODES {
            return;
        }
        let original = self.nodes.len();
        let mut degree: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        for e in &self.edges {
            *degree.entry(e.from.as_str()).or_insert(0) += 1;
            *degree.entry(e.to.as_str()).or_insert(0) += 1;
        }
        let center = self.center.clone();
        let mut idx: Vec<usize> = (0..self.nodes.len()).collect();
        idx.sort_by(|&a, &b| {
            let an = self.nodes[a].id.as_str();
            let bn = self.nodes[b].id.as_str();
            let ad = degree.get(an).copied().unwrap_or(0);
            let bd = degree.get(bn).copied().unwrap_or(0);
            let ac = if an == center { u32::MAX } else { ad };
            let bc = if bn == center { u32::MAX } else { bd };
            bc.cmp(&ac).then(an.cmp(bn))
        });
        let keep: std::collections::HashSet<String> =
            idx.iter().take(MAX_NODES).map(|&i| self.nodes[i].id.clone()).collect();
        self.nodes.retain(|n| keep.contains(&n.id));
        self.edges.retain(|e| keep.contains(&e.from) && keep.contains(&e.to));
        let collapsed = original.saturating_sub(MAX_NODES) as u32;
        let kind = self.nodes.first().map(|n| n.kind).unwrap_or(LensKind::Symbol);
        self.summary = Some(LensSummary::format(collapsed, kind, parent_label));
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("LensView is always serializable")
    }

    /// Render as Mermaid dagre flowchart. Sanitises node IDs (Mermaid forbids
    /// dots, hashes, slashes in identifiers) by hashing-on-the-fly.
    pub fn to_mermaid(&self) -> String {
        let mut out = String::new();
        out.push_str("```mermaid\n");
        out.push_str("---\n");
        out.push_str(&format!("title: cgdb lens — {}\n", self.center));
        out.push_str("---\n");
        out.push_str("flowchart LR\n");
        for n in &self.nodes {
            let sid = sanitize(&n.id);
            let label = escape_label(&n.label);
            let shape_l = match n.kind {
                LensKind::Project | LensKind::ProjectSpecs => '(',
                LensKind::Crate | LensKind::Directory => '[',
                LensKind::Section | LensKind::Subsection => '>',
                _ => '[',
            };
            let shape_r = match shape_l {
                '(' => ')',
                '[' => ']',
                '>' => ']',
                _ => ']',
            };
            out.push_str(&format!("  {}{}{}\"{}\"{}\n", sid, shape_l, '"', label, shape_r));
        }
        for e in &self.edges {
            let from = sanitize(&e.from);
            let to = sanitize(&e.to);
            let arrow = match e.edge_type {
                LensEdgeType::SpecRef => "-->",
                LensEdgeType::SemanticSpecRef => "-.->",
                LensEdgeType::Contains => "===",
            };
            out.push_str(&format!("  {} {} {}\n", from, arrow, to));
        }
        if let Some(s) = &self.summary {
            out.push_str(&format!("  summary[\"{}\"]\n", escape_label(&s.collapsed_summary)));
        }
        out.push_str("```\n");
        out
    }
}

fn sanitize(id: &str) -> String {
    let mut s = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            s.push(ch);
        } else {
            s.push('_');
        }
    }
    if s.is_empty() || !s.chars().next().unwrap().is_ascii_alphabetic() {
        s.insert(0, 'n');
    }
    s
}

fn escape_label(s: &str) -> String {
    s.replace('"', "\\\"")
}

/// Breadcrumb response struct (lens.breadcrumb).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreadcrumbEntry {
    pub node_id: String,
    pub level: u8,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Breadcrumb {
    pub path: Vec<BreadcrumbEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(id: &str, kind: LensKind) -> LensNode {
        LensNode { id: id.into(), level: kind.level(), kind, label: id.into(), section_type: None }
    }

    fn e(from: &str, to: &str) -> LensEdge {
        LensEdge {
            from: from.into(),
            to: to.into(),
            edge_type: LensEdgeType::SpecRef,
            source: LensEdgeSource::DocComment,
            weight: None,
        }
    }

    #[test]
    fn cap_at_max_truncates_to_50_and_keeps_center() {
        let mut v = LensView {
            center: "ctr".into(),
            nodes: (0..80)
                .map(|i| n(&format!("nd{}", i), LensKind::Symbol))
                .chain(std::iter::once(n("ctr", LensKind::Symbol)))
                .collect(),
            edges: vec![e("ctr", "nd0")],
            summary: None,
        };
        v.cap_at_max("project-x");
        assert_eq!(v.nodes.len(), MAX_NODES);
        assert!(v.nodes.iter().any(|n| n.id == "ctr"), "center always kept");
        assert!(v.summary.is_some());
        let s = v.summary.as_ref().unwrap();
        assert!(s.collapsed_summary.starts_with("+"));
        assert!(s.collapsed_summary.contains("symbols"));
        assert!(s.collapsed_summary.contains("in project-x"));
    }

    #[test]
    fn to_mermaid_emits_dagre_flowchart() {
        let v = LensView {
            center: "p".into(),
            nodes: vec![n("p", LensKind::Project), n("c", LensKind::Crate)],
            edges: vec![e("c", "p")],
            summary: None,
        };
        let m = v.to_mermaid();
        assert!(m.contains("flowchart LR"));
        assert!(m.contains("\"p\""));
        assert!(m.contains("-->"));
    }

    #[test]
    fn json_round_trip_preserves_shape() {
        let v = LensView {
            center: "x".into(),
            nodes: vec![n("x", LensKind::Symbol)],
            edges: vec![],
            summary: None,
        };
        let j = v.to_json();
        let back: LensView = serde_json::from_value(j).unwrap();
        assert_eq!(back.center, "x");
        assert_eq!(back.nodes.len(), 1);
    }
}
// HANDWRITE-END
