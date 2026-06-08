// HANDWRITE-BEGIN gap="missing-generator:hand-written:8f3753d9" tracker="2087" reason="GraphRecord JSONL schema + append-only writer (single write() per record) + scan iterator."
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum NodeType {
    Spec,
    Code,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EdgeType {
    SpecRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RegionKind {
    Codegen,
    Handwrite,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeSource {
    DocComment,
    SpecRefMarker,
    CodegenPayload,
    HandwritePayload,
}

pub const LEVEL_UNSPECIFIED: u8 = 255;
fn default_level() -> u8 { LEVEL_UNSPECIFIED }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub file: String,
    pub anchor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_kind: Option<RegionKind>,
    #[serde(default = "default_level", skip_serializing_if = "is_unspecified_level")]
    pub level: u8,
}

fn is_unspecified_level(v: &u8) -> bool { *v == LEVEL_UNSPECIFIED }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRecord {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
    pub source: EdgeSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum GraphPayload {
    Node(NodeRecord),
    Edge(EdgeRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRecord {
    pub v: u32,
    #[serde(flatten)]
    pub payload: GraphPayload,
}

impl GraphRecord {
    pub fn node(n: NodeRecord) -> Self {
        Self { v: 1, payload: GraphPayload::Node(n) }
    }
    pub fn edge(e: EdgeRecord) -> Self {
        Self { v: 1, payload: GraphPayload::Edge(e) }
    }
}

pub struct GraphAppender {
    path: PathBuf,
    file: File,
}

impl GraphAppender {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("open graph file {}", path.display()))?;
        Ok(Self { path: path.to_path_buf(), file })
    }

    pub fn truncate(path: &Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Append one whole record as a single write() syscall (crash-safe atomic unit).
    pub fn append(&mut self, record: &GraphRecord) -> Result<()> {
        let mut line = serde_json::to_vec(record)?;
        line.push(b'\n');
        self.file.write_all(&line)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn scan(path: &Path) -> Result<Vec<GraphRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let rec: GraphRecord = serde_json::from_str(&line)
            .with_context(|| format!("parse graph record: {}", line))?;
        out.push(rec);
    }
    Ok(out)
}
// HANDWRITE-END
// SPEC-MANAGED: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#changes

// HANDWRITE-BEGIN gap="broken-generator:gen-code-stub-uncompilable" tracker="2092" reason="gen-code emitted lens dispatch as free-standing async fns with &self (no impl block), undefined identifiers (nodes/format), empty `#[derive(Subcommand)]` enum without the clap import, and illegal `else if … else if` chains. Replaced with a HANDWRITE block that keeps only the SyncPipeline state machine (cgdb-v0-1 sync pipeline). Lens dispatch + RPC methods now live in cgdb-daemon. Reopen codegen once the logic generator emits valid Rust and the rpc-api generator targets an impl block."

/// Sync pipeline state machine per cgdb-v0-1.md#cgdb-v0-1-sync-pipeline.
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
        matches!(self, Self::Failed | Self::Done)
    }

    pub fn can_transition_to(&self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Classifying, Self::EmbeddingCpu)
                | (Self::Classifying, Self::EmbeddingGpu)
                | (Self::Cosine, Self::Writing)
                | (Self::EmbeddingCpu, Self::Cosine)
                | (Self::EmbeddingCpu, Self::Failed)
                | (Self::EmbeddingGpu, Self::Cosine)
                | (Self::EmbeddingGpu, Self::Failed)
                | (Self::Idle, Self::Walking)
                | (Self::Walking, Self::Classifying)
                | (Self::Writing, Self::Done)
                | (Self::Writing, Self::Failed)
        )
    }
}
// HANDWRITE-END
