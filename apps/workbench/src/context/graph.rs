// HANDWRITE-BEGIN gap="missing-generator:logic:a8e2b94e" tracker="pending-tracker" reason="Define the repository-owned v1 graph payload, bounded read source, node/edge validation, provenance mapping, escaped graph rendering, and provider-failure surface."
use std::{
    collections::{BTreeSet, HashSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;

use super::{
    escape_html,
    provenance::{
        ContextProvenanceItem, ProvenanceClassification, ProvenanceView, ProviderIdentity,
        SourceLocation, SourceStatus,
    },
    ContextDocument, ContextDocumentKind, ContextNavigation, ContextProvenance, ContextRenderer,
    ContextRequest, ContextTarget, RendererError, RendererSupport,
};

pub const GRAPH_PAYLOAD_RELATIVE_PATH: &str = "graphify-out/workbench-graph.json";
pub const MAX_GRAPH_PAYLOAD_BYTES: usize = 1024 * 1024;
const MAX_GRAPH_NODES: usize = 2_048;
const MAX_GRAPH_EDGES: usize = 4_096;
const MAX_SOURCES_PER_RECORD: usize = 16;
const MAX_ID_BYTES: usize = 128;
const MAX_LABEL_BYTES: usize = 512;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphPayload {
    schema_version: String,
    provider: ProviderIdentity,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Clone, Debug, Deserialize)]
struct GraphNode {
    id: String,
    label: String,
    #[serde(default)]
    kind: String,
    classification: ProvenanceClassification,
    sources: Vec<SourceLocation>,
}

#[derive(Clone, Debug, Deserialize)]
struct GraphEdge {
    id: String,
    from: String,
    to: String,
    label: String,
    classification: ProvenanceClassification,
    sources: Vec<SourceLocation>,
}

pub trait GraphPayloadSource: Send + Sync {
    fn read(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, RendererError>;
}

#[derive(Debug, Default)]
pub struct FileGraphPayloadSource;

impl GraphPayloadSource for FileGraphPayloadSource {
    fn read(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, RendererError> {
        let metadata = fs::metadata(path).map_err(|error| {
            RendererError::new(format!("could not inspect graph payload: {error}"))
        })?;
        if !metadata.is_file() {
            return Err(RendererError::new("graph payload is not a regular file"));
        }
        if metadata.len() > max_bytes as u64 {
            return Err(RendererError::new(format!(
                "graph payload exceeds the {max_bytes}-byte limit"
            )));
        }

        let file = File::open(path)
            .map_err(|error| RendererError::new(format!("could not open graph payload: {error}")))?;
        let mut bytes = Vec::new();
        file.take(max_bytes as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| RendererError::new(format!("could not read graph payload: {error}")))?;
        if bytes.len() > max_bytes {
            return Err(RendererError::new(format!(
                "graph payload exceeds the {max_bytes}-byte limit"
            )));
        }
        Ok(bytes)
    }
}

#[derive(Clone)]
pub struct GraphContextRenderer {
    source: Arc<dyn GraphPayloadSource>,
}

impl std::fmt::Debug for GraphContextRenderer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GraphContextRenderer")
            .finish_non_exhaustive()
    }
}

impl Default for GraphContextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphContextRenderer {
    pub fn new() -> Self {
        Self::with_source(FileGraphPayloadSource)
    }

    pub fn with_source(source: impl GraphPayloadSource + 'static) -> Self {
        Self {
            source: Arc::new(source),
        }
    }

    fn payload_path(request: &ContextRequest) -> PathBuf {
        request.root().join(GRAPH_PAYLOAD_RELATIVE_PATH)
    }
}

impl ContextRenderer for GraphContextRenderer {
    fn id(&self) -> &'static str {
        "graph-context"
    }

    fn priority(&self) -> i32 {
        350
    }

    fn supports(&self, request: &ContextRequest) -> RendererSupport {
        if !matches!(request.target(), ContextTarget::Workspace) {
            return RendererSupport::Unsupported;
        }
        if Self::payload_path(request).is_file() {
            RendererSupport::Supported
        } else {
            RendererSupport::Unsupported
        }
    }

    fn render(&self, request: &ContextRequest) -> Result<ContextDocument, RendererError> {
        if !matches!(request.target(), ContextTarget::Workspace) {
            return Err(RendererError::new("graph context supports only workspace targets"));
        }
        let requested_path = Self::payload_path(request);
        let payload_path = fs::canonicalize(&requested_path).map_err(|error| {
            RendererError::new(format!("could not resolve graph payload: {error}"))
        })?;
        if !payload_path.starts_with(request.root()) || !payload_path.is_file() {
            return Err(RendererError::new(
                "graph payload must be a regular file confined to the selected root",
            ));
        }

        let bytes = self.source.read(&payload_path, MAX_GRAPH_PAYLOAD_BYTES)?;
        let payload: GraphPayload = serde_json::from_slice(&bytes)
            .map_err(|error| RendererError::new(format!("malformed graph payload: {error}")))?;
        validate_payload(&payload, request.root())?;
        render_payload(payload, request, payload_path)
    }
}

fn validate_payload(payload: &GraphPayload, root: &Path) -> Result<(), RendererError> {
    if payload.schema_version != "workbench.graph-context.v1" {
        return Err(RendererError::new(format!(
            "unsupported graph schema {:?}",
            payload.schema_version
        )));
    }
    validate_text("provider id", &payload.provider.id, MAX_ID_BYTES)?;
    validate_text("provider label", &payload.provider.label, MAX_LABEL_BYTES)?;
    if payload.nodes.len() > MAX_GRAPH_NODES || payload.edges.len() > MAX_GRAPH_EDGES {
        return Err(RendererError::new(format!(
            "graph exceeds node/edge limits ({MAX_GRAPH_NODES}/{MAX_GRAPH_EDGES})"
        )));
    }

    let mut all_ids = HashSet::new();
    let mut node_ids = HashSet::new();
    for node in &payload.nodes {
        validate_text("node id", &node.id, MAX_ID_BYTES)?;
        validate_text("node label", &node.label, MAX_LABEL_BYTES)?;
        if !node.kind.is_empty() {
            validate_text("node kind", &node.kind, MAX_ID_BYTES)?;
        }
        if !all_ids.insert(node.id.as_str()) {
            return Err(RendererError::new(format!("duplicate graph id {:?}", node.id)));
        }
        node_ids.insert(node.id.as_str());
        validate_sources(node.classification, &node.sources, root)?;
    }
    for edge in &payload.edges {
        validate_text("edge id", &edge.id, MAX_ID_BYTES)?;
        validate_text("edge label", &edge.label, MAX_LABEL_BYTES)?;
        if !all_ids.insert(edge.id.as_str()) {
            return Err(RendererError::new(format!("duplicate graph id {:?}", edge.id)));
        }
        if !node_ids.contains(edge.from.as_str()) || !node_ids.contains(edge.to.as_str()) {
            return Err(RendererError::new(format!(
                "edge {:?} references an unknown endpoint",
                edge.id
            )));
        }
        validate_sources(edge.classification, &edge.sources, root)?;
    }
    Ok(())
}

fn validate_text(field: &str, value: &str, max_bytes: usize) -> Result<(), RendererError> {
    if value.trim().is_empty() || value.len() > max_bytes {
        return Err(RendererError::new(format!(
            "{field} must contain 1..={max_bytes} bytes"
        )));
    }
    Ok(())
}

fn validate_sources(
    classification: ProvenanceClassification,
    sources: &[SourceLocation],
    root: &Path,
) -> Result<(), RendererError> {
    if sources.is_empty() || sources.len() > MAX_SOURCES_PER_RECORD {
        return Err(RendererError::new(format!(
            "every graph record needs 1..={MAX_SOURCES_PER_RECORD} source inputs"
        )));
    }
    if classification == ProvenanceClassification::Extracted && sources.len() != 1 {
        return Err(RendererError::new(
            "an extracted graph record must identify exactly one source",
        ));
    }
    let view = provenance_item(
        ProviderIdentity::new("validation", "Graph validation"),
        classification,
        sources.to_vec(),
    )
    .resolve(root);
    if view
        .sources
        .iter()
        .any(|source| matches!(source.status, SourceStatus::Invalid { .. }))
    {
        return Err(RendererError::new(
            "graph source paths and spans must be confined, relative, and one-based",
        ));
    }
    Ok(())
}

fn provenance_item(
    provider: ProviderIdentity,
    classification: ProvenanceClassification,
    sources: Vec<SourceLocation>,
) -> ContextProvenanceItem {
    match classification {
        ProvenanceClassification::Extracted => {
            ContextProvenanceItem::extracted(provider, sources[0].clone())
        }
        ProvenanceClassification::Inferred => ContextProvenanceItem::inferred(provider, sources),
        ProvenanceClassification::Ambiguous => ContextProvenanceItem::ambiguous(provider, sources),
    }
}

fn render_payload(
    payload: GraphPayload,
    request: &ContextRequest,
    payload_path: PathBuf,
) -> Result<ContextDocument, RendererError> {
    let mut navigation = Vec::new();
    let mut canonical_sources = BTreeSet::new();
    let mut nodes_html = String::new();
    let mut edges_html = String::new();

    for node in payload.nodes {
        let view = provenance_item(
            payload.provider.clone(),
            node.classification,
            node.sources,
        )
        .resolve(request.root());
        nodes_html.push_str(&render_record(
            "node",
            &node.id,
            &node.label,
            if node.kind.is_empty() { None } else { Some(&node.kind) },
            &view,
            request.root(),
            &mut navigation,
            &mut canonical_sources,
        ));
    }
    for edge in payload.edges {
        let view = provenance_item(
            payload.provider.clone(),
            edge.classification,
            edge.sources,
        )
        .resolve(request.root());
        edges_html.push_str(&render_record(
            "edge",
            &edge.id,
            &edge.label,
            Some(&format!("{} → {}", edge.from, edge.to)),
            &view,
            request.root(),
            &mut navigation,
            &mut canonical_sources,
        ));
    }
    canonical_sources.insert(payload_path);

    Ok(ContextDocument {
        renderer_id: "graph-context".to_owned(),
        kind: ContextDocumentKind::Graph,
        title: format!("{} graph context", payload.provider.label),
        body_html: format!(
            "<section class=\"graph-context\"><p class=\"authority\">Derived compatibility view from {}</p><h2>Nodes</h2>{}<h2>Edges</h2>{}</section>",
            escape_html(&payload.provider.label),
            nodes_html,
            edges_html
        ),
        navigation,
        warnings: Vec::new(),
        provenance: ContextProvenance {
            root: request.root().to_path_buf(),
            sources: canonical_sources.into_iter().collect(),
        },
    })
}

#[allow(clippy::too_many_arguments)]
fn render_record(
    record_kind: &str,
    id: &str,
    label: &str,
    detail: Option<&str>,
    view: &ProvenanceView,
    root: &Path,
    navigation: &mut Vec<ContextNavigation>,
    canonical_sources: &mut BTreeSet<PathBuf>,
) -> String {
    let mut sources_html = String::new();
    for source in &view.sources {
        let requested = source.requested.relative_path.display().to_string();
        let status = match &source.status {
            SourceStatus::Canonical => "canonical",
            SourceStatus::Missing => "missing",
            SourceStatus::Invalid { .. } => "invalid",
        };
        sources_html.push_str(&format!(
            "<li data-source-status=\"{}\">{}</li>",
            status,
            escape_html(&requested)
        ));
        if let Some(link) = &source.navigation {
            let line = link.span.map(|span| span.start.line);
            let link_label = line.map_or_else(
                || format!("{record_kind} {id}: {}", link.relative_path.display()),
                |line| format!(
                    "{record_kind} {id}: {}:{line}",
                    link.relative_path.display()
                ),
            );
            navigation.push(ContextNavigation {
                label: link_label,
                path: link.relative_path.clone(),
            });
            if let Ok(canonical) = fs::canonicalize(root.join(&link.relative_path)) {
                canonical_sources.insert(canonical);
            }
        }
    }
    let detail_html = detail.map_or_else(String::new, |detail| {
        format!("<p class=\"detail\">{}</p>", escape_html(detail))
    });
    format!(
        "<article class=\"graph-record\" data-record-kind=\"{}\" data-record-id=\"{}\"><h3>{}</h3>{}<p class=\"provenance-badge\">{}</p><ul>{}</ul></article>",
        escape_html(record_kind),
        escape_html(id),
        escape_html(label),
        detail_html,
        escape_html(&view.badge),
        sources_html
    )
}
// HANDWRITE-END
