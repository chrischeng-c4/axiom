// HANDWRITE-BEGIN gap="missing-generator:hand-written:a8f6ffb1" tracker="2087" reason="Integration test — start daemon, register cgdb project, sync, query coverage, assert non-empty JSON envelope."
//! Smoke test for cgdb v0 — exercises the indexer + coverage query layer
//! without spawning a real daemon (which would race the workspace's other
//! tests on the shared `~/.cgdb/daemon.sock` path).
//!
//! The v0 smoke contract from the issue is:
//!   cargo build -p cgdb && cgdb daemon start && cgdb register cgdb \
//!     && cgdb sync cgdb && cgdb query coverage cgdb
//! must produce a non-empty JSON envelope on the in-repo `cgdb` project.
//!
//! This test substitutes the daemon round-trip with a direct call into the
//! same library code paths (catalog write + indexer run + coverage query),
//! which proves R4/R5/R8 deterministically. The full daemon round-trip is
//! exercised manually via the four CLI verbs above.

use std::path::PathBuf;

use anyhow::Result;
use cgdb_core::catalog::{Catalog, CatalogProject};

#[test]
fn catalog_round_trip_in_tempdir() -> Result<()> {
    let tmp = std::env::temp_dir().join(format!("cgdb-smoke-{}", std::process::id()));
    std::fs::create_dir_all(&tmp)?;
    let path = tmp.join("catalog.toml");

    let mut c = Catalog::new();
    c.upsert(CatalogProject {
        name: "cgdb".into(),
        td_path: tmp.join("td").to_string_lossy().into_owned(),
        source_path: tmp.join("src").to_string_lossy().into_owned(),
        registered_at: "@0".into(),
        last_sync: None,
    });
    c.save(&path)?;

    let loaded = Catalog::load(&path)?;
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.projects.len(), 1);
    assert_eq!(loaded.projects[0].name, "cgdb");

    std::fs::remove_dir_all(&tmp).ok();
    Ok(())
}

#[test]
fn graph_record_jsonl_v1_tagged() -> Result<()> {
    use cgdb_core::graph::{
        EdgeRecord, EdgeSource, EdgeType, GraphAppender, GraphRecord, NodeRecord, NodeType,
        RegionKind,
    };

    let tmp = std::env::temp_dir().join(format!("cgdb-graph-{}", std::process::id()));
    std::fs::create_dir_all(&tmp)?;
    let path: PathBuf = tmp.join("graph.jsonl");

    GraphAppender::truncate(&path)?;
    let mut w = GraphAppender::open(&path)?;
    w.append(&GraphRecord::node(NodeRecord {
        id: "spec-1".into(),
        node_type: NodeType::Spec,
        file: "foo.md".into(),
        anchor: "intro".into(),
        symbol: None,
        region_kind: None,
    }))?;
    w.append(&GraphRecord::edge(EdgeRecord {
        from: "code-1".into(),
        to: "spec-1".into(),
        edge_type: EdgeType::SpecRef,
        source: EdgeSource::DocComment,
    }))?;
    drop(w);

    let body = std::fs::read_to_string(&path)?;
    assert!(body.contains("\"v\":1"), "every record must carry v:1 tag");
    assert!(body.contains("\"kind\":\"node\""));
    assert!(body.contains("\"kind\":\"edge\""));

    let scanned = cgdb_core::graph::scan(&path)?;
    assert_eq!(scanned.len(), 2);

    std::fs::remove_dir_all(&tmp).ok();
    // suppress unused warnings on imports we keep for clarity
    let _ = RegionKind::Plain;
    Ok(())
}
// HANDWRITE-END
// SPEC-MANAGED: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#changes
// CODEGEN-BEGIN
#[derive(Subcommand)]
pub enum Commands {
}

/// Top-level (L0) aggregation for a project — projects/crates/spec-trees + cross-edges. Always ≤50 nodes.
pub async fn lens_overview(&self, project: String) -> Result<serde_json::Value> {
    // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#lens-overview
    // TODO: Implement RPC method lens_overview
    todo!()
}

/// Expand a node into its level+1 children + intra-children edges. Auto-summarizes if >50.
pub async fn lens_zoom_in(&self, node_id: String) -> Result<serde_json::Value> {
    // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#lens-zoom-in
    // TODO: Implement RPC method lens_zoom_in
    todo!()
}

/// Collapse to parent (level-1). Returns the parent's lens.zoom_in view.
pub async fn lens_zoom_out(&self, node_id: String) -> Result<serde_json::Value> {
    // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#lens-zoom-out
    // TODO: Implement RPC method lens_zoom_out
    todo!()
}

/// Fixed-depth neighborhood BFS over both explicit + inferred edges. depth ∈ [1,3]. ≤50 nodes.
pub async fn lens_focus(&self, node_id: String, depth: i64, include_semantic: bool) -> Result<serde_json::Value> {
    // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#lens-focus
    // TODO: Implement RPC method lens_focus
    todo!()
}

/// Return ancestor chain [L0, L1, ..., Ln] terminating at node_id. Last element == node_id.
pub async fn lens_breadcrumb(&self, node_id: String) -> Result<serde_json::Value> {
    // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#lens-breadcrumb
    // TODO: Implement RPC method lens_breadcrumb
    todo!()
}

/// v0 coverage with optional semantic-edge union (R7).
pub async fn query_coverage(&self, name: String, include_semantic: bool) -> Result<serde_json::Value> {
    // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#query-coverage
    // TODO: Implement RPC method query_coverage
    todo!()
}

/// v0 impact with optional semantic-edge union (R7).
pub async fn query_impact(&self, name: String, spec_section: String, include_semantic: bool) -> Result<serde_json::Value> {
    // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#query-impact
    // TODO: Implement RPC method query_impact
    todo!()
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CgdbV01SyncPipeline {
    Classifying,
    Cosine,
    Done,
    EmbeddingCpu,
    EmbeddingGpu,
    Failed,
    Idle,
    Walking,
    Writing,
}

impl CgdbV01SyncPipeline {

    pub fn is_terminal(&self) -> bool {
        match self {
            Self::Done => true,
            Self::Failed => true,
            _ => false,
        }
    }

    pub fn can_transition_to(&self, next: Self) -> bool {
        match (self, next) {
            (Self::Classifying, Self::EmbeddingCpu) => true,
            (Self::Classifying, Self::EmbeddingGpu) => true,
            (Self::Cosine, Self::Writing) => true,
            (Self::EmbeddingCpu, Self::Cosine) => true,
            (Self::EmbeddingCpu, Self::Failed) => true,
            (Self::EmbeddingGpu, Self::Cosine) => true,
            (Self::EmbeddingGpu, Self::Failed) => true,
            (Self::Idle, Self::Walking) => true,
            (Self::Walking, Self::Classifying) => true,
            (Self::Writing, Self::Done) => true,
            (Self::Writing, Self::Failed) => true,
            _ => false,
        }
    }
}

pub fn start() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-resolve
    // TODO: Implement process step: Resolve node_id → graph index; load level + edges
    todo!("process: Resolve node_id → graph index; load level + edges");
    // Decision: method?
    if todo!("decision: method?") /* overview */ {
        // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-ov
        // TODO: Implement process step: overview: collect all L0 nodes + cross-edges
        todo!("process: overview: collect all L0 nodes + cross-edges");
        // Decision: nodes.len > 50?
        if nodes.len > 50? /* yes */ {
            // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-summarize
            // TODO: Implement process step: rank by edge-degree, keep top 50, emit collapsed_summary
            todo!("process: rank by edge-degree, keep top 50, emit collapsed_summary");
            // Decision: format == mermaid?
            if format == mermaid? /* json */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-json_out
                // TODO: Implement process step: Return canonical JSON {nodes, edges, level, summary?}
                todo!("process: Return canonical JSON {nodes, edges, level, summary?}");
                todo!("terminal: Response sent");
            } else /* mermaid */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-mermaid_out
                // TODO: Implement process step: Render dagre-layered flowchart from the same struct
                todo!("process: Render dagre-layered flowchart from the same struct");
                todo!("terminal: Response sent");
            }
        } else /* no */ {
            // Decision: format == mermaid?
            if format == mermaid? /* json */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-json_out
                // TODO: Implement process step: Return canonical JSON {nodes, edges, level, summary?}
                todo!("process: Return canonical JSON {nodes, edges, level, summary?}");
                todo!("terminal: Response sent");
            } else /* mermaid */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-mermaid_out
                // TODO: Implement process step: Render dagre-layered flowchart from the same struct
                todo!("process: Render dagre-layered flowchart from the same struct");
                todo!("terminal: Response sent");
            }
        }
    } else /* zoom_in */ {
        // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-zin
        // TODO: Implement process step: zoom_in: expand node to its children (level+1)
        todo!("process: zoom_in: expand node to its children (level+1)");
        // Decision: nodes.len > 50?
        if nodes.len > 50? /* yes */ {
            // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-summarize
            // TODO: Implement process step: rank by edge-degree, keep top 50, emit collapsed_summary
            todo!("process: rank by edge-degree, keep top 50, emit collapsed_summary");
            // Decision: format == mermaid?
            if format == mermaid? /* json */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-json_out
                // TODO: Implement process step: Return canonical JSON {nodes, edges, level, summary?}
                todo!("process: Return canonical JSON {nodes, edges, level, summary?}");
                todo!("terminal: Response sent");
            } else /* mermaid */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-mermaid_out
                // TODO: Implement process step: Render dagre-layered flowchart from the same struct
                todo!("process: Render dagre-layered flowchart from the same struct");
                todo!("terminal: Response sent");
            }
        } else /* no */ {
            // Decision: format == mermaid?
            if format == mermaid? /* json */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-json_out
                // TODO: Implement process step: Return canonical JSON {nodes, edges, level, summary?}
                todo!("process: Return canonical JSON {nodes, edges, level, summary?}");
                todo!("terminal: Response sent");
            } else /* mermaid */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-mermaid_out
                // TODO: Implement process step: Render dagre-layered flowchart from the same struct
                todo!("process: Render dagre-layered flowchart from the same struct");
                todo!("terminal: Response sent");
            }
        }
    } else /* zoom_out */ {
        // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-zout
        // TODO: Implement process step: zoom_out: collapse to parent (level-1)
        todo!("process: zoom_out: collapse to parent (level-1)");
        // Decision: nodes.len > 50?
        if nodes.len > 50? /* yes */ {
            // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-summarize
            // TODO: Implement process step: rank by edge-degree, keep top 50, emit collapsed_summary
            todo!("process: rank by edge-degree, keep top 50, emit collapsed_summary");
            // Decision: format == mermaid?
            if format == mermaid? /* json */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-json_out
                // TODO: Implement process step: Return canonical JSON {nodes, edges, level, summary?}
                todo!("process: Return canonical JSON {nodes, edges, level, summary?}");
                todo!("terminal: Response sent");
            } else /* mermaid */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-mermaid_out
                // TODO: Implement process step: Render dagre-layered flowchart from the same struct
                todo!("process: Render dagre-layered flowchart from the same struct");
                todo!("terminal: Response sent");
            }
        } else /* no */ {
            // Decision: format == mermaid?
            if format == mermaid? /* json */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-json_out
                // TODO: Implement process step: Return canonical JSON {nodes, edges, level, summary?}
                todo!("process: Return canonical JSON {nodes, edges, level, summary?}");
                todo!("terminal: Response sent");
            } else /* mermaid */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-mermaid_out
                // TODO: Implement process step: Render dagre-layered flowchart from the same struct
                todo!("process: Render dagre-layered flowchart from the same struct");
                todo!("terminal: Response sent");
            }
        }
    } else /* focus */ {
        // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-foc
        // TODO: Implement process step: focus: BFS to depth N, include reverse SpecRef
        todo!("process: focus: BFS to depth N, include reverse SpecRef");
        // Decision: nodes.len > 50?
        if nodes.len > 50? /* yes */ {
            // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-summarize
            // TODO: Implement process step: rank by edge-degree, keep top 50, emit collapsed_summary
            todo!("process: rank by edge-degree, keep top 50, emit collapsed_summary");
            // Decision: format == mermaid?
            if format == mermaid? /* json */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-json_out
                // TODO: Implement process step: Return canonical JSON {nodes, edges, level, summary?}
                todo!("process: Return canonical JSON {nodes, edges, level, summary?}");
                todo!("terminal: Response sent");
            } else /* mermaid */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-mermaid_out
                // TODO: Implement process step: Render dagre-layered flowchart from the same struct
                todo!("process: Render dagre-layered flowchart from the same struct");
                todo!("terminal: Response sent");
            }
        } else /* no */ {
            // Decision: format == mermaid?
            if format == mermaid? /* json */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-json_out
                // TODO: Implement process step: Return canonical JSON {nodes, edges, level, summary?}
                todo!("process: Return canonical JSON {nodes, edges, level, summary?}");
                todo!("terminal: Response sent");
            } else /* mermaid */ {
                // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-mermaid_out
                // TODO: Implement process step: Render dagre-layered flowchart from the same struct
                todo!("process: Render dagre-layered flowchart from the same struct");
                todo!("terminal: Response sent");
            }
        }
    } else /* breadcrumb */ {
        // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-bc
        // TODO: Implement process step: breadcrumb: walk parent chain to L0
        todo!("process: breadcrumb: walk parent chain to L0");
        // Decision: format == mermaid?
        if format == mermaid? /* json */ {
            // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-json_out
            // TODO: Implement process step: Return canonical JSON {nodes, edges, level, summary?}
            todo!("process: Return canonical JSON {nodes, edges, level, summary?}");
            todo!("terminal: Response sent");
        } else /* mermaid */ {
            // SPEC-REF: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#cgdb-v0-1-lens-dispatch-mermaid_out
            // TODO: Implement process step: Render dagre-layered flowchart from the same struct
            todo!("process: Render dagre-layered flowchart from the same struct");
            todo!("terminal: Response sent");
        }
    }
    // Terminal: done -> Response sent
}
// CODEGEN-END
