// HANDWRITE-BEGIN gap="missing-generator:logic:c21410db" tracker="pending-tracker" reason="Define the repository-owned v1 derived-page payload, bounded byte source, section and citation validation, safe Markdown rendering, freshness labels, and provider-failure surface."
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
    markdown::safe_markdown_html,
    provenance::{
        ContextProvenanceItem, ProvenanceClassification, ProvenanceView, ProviderIdentity,
        SourceLocation, SourceStatus,
    },
    ContextDocument, ContextDocumentKind, ContextNavigation, ContextProvenance, ContextRenderer,
    ContextRequest, ContextTarget, RendererError, RendererSupport,
};

pub const DERIVED_PAGE_PAYLOAD_RELATIVE_PATH: &str =
    "llm-wiki-out/workbench-pages.json";
pub const MAX_DERIVED_PAGE_PAYLOAD_BYTES: usize = 1024 * 1024;
const MAX_SECTIONS: usize = 512;
const MAX_CITATIONS_PER_SECTION: usize = 16;
const MAX_ID_BYTES: usize = 128;
const MAX_HEADING_BYTES: usize = 512;
const MAX_SECTION_MARKDOWN_BYTES: usize = 64 * 1024;
const MAX_FRESHNESS_NOTE_BYTES: usize = 1024;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DerivedPagePayload {
    schema_version: String,
    provider: ProviderIdentity,
    page: DerivedPage,
}

#[derive(Clone, Debug, Deserialize)]
struct DerivedPage {
    id: String,
    title: String,
    sections: Vec<DerivedSection>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DerivedSection {
    id: String,
    heading: String,
    body_markdown: String,
    classification: ProvenanceClassification,
    citations: Vec<SourceLocation>,
    freshness: PageFreshness,
    #[serde(default)]
    freshness_note: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PageFreshness {
    Current,
    Stale,
    Unknown,
}

impl PageFreshness {
    fn label(self) -> &'static str {
        match self {
            Self::Current => "Current",
            Self::Stale => "Stale",
            Self::Unknown => "Unknown",
        }
    }
}

pub trait DerivedPagePayloadSource: Send + Sync {
    fn read(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, RendererError>;
}

#[derive(Debug, Default)]
pub struct FileDerivedPagePayloadSource;

impl DerivedPagePayloadSource for FileDerivedPagePayloadSource {
    fn read(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, RendererError> {
        let metadata = fs::metadata(path).map_err(|error| {
            RendererError::new(format!("could not inspect derived-page payload: {error}"))
        })?;
        if !metadata.is_file() {
            return Err(RendererError::new(
                "derived-page payload is not a regular file",
            ));
        }
        if metadata.len() > max_bytes as u64 {
            return Err(RendererError::new(format!(
                "derived-page payload exceeds the {max_bytes}-byte limit"
            )));
        }

        let file = File::open(path).map_err(|error| {
            RendererError::new(format!("could not open derived-page payload: {error}"))
        })?;
        let mut bytes = Vec::new();
        file.take(max_bytes as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| {
                RendererError::new(format!("could not read derived-page payload: {error}"))
            })?;
        if bytes.len() > max_bytes {
            return Err(RendererError::new(format!(
                "derived-page payload exceeds the {max_bytes}-byte limit"
            )));
        }
        Ok(bytes)
    }
}

#[derive(Clone)]
pub struct DerivedPageContextRenderer {
    source: Arc<dyn DerivedPagePayloadSource>,
}

impl std::fmt::Debug for DerivedPageContextRenderer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DerivedPageContextRenderer")
            .finish_non_exhaustive()
    }
}

impl Default for DerivedPageContextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl DerivedPageContextRenderer {
    pub fn new() -> Self {
        Self::with_source(FileDerivedPagePayloadSource)
    }

    pub fn with_source(source: impl DerivedPagePayloadSource + 'static) -> Self {
        Self {
            source: Arc::new(source),
        }
    }

    fn payload_path(request: &ContextRequest) -> PathBuf {
        request.root().join(DERIVED_PAGE_PAYLOAD_RELATIVE_PATH)
    }
}

impl ContextRenderer for DerivedPageContextRenderer {
    fn id(&self) -> &'static str {
        "derived-page-context"
    }

    fn priority(&self) -> i32 {
        340
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
            return Err(RendererError::new(
                "derived-page context supports only workspace targets",
            ));
        }
        let requested_path = Self::payload_path(request);
        let payload_path = fs::canonicalize(&requested_path).map_err(|error| {
            RendererError::new(format!("could not resolve derived-page payload: {error}"))
        })?;
        if !payload_path.starts_with(request.root()) || !payload_path.is_file() {
            return Err(RendererError::new(
                "derived-page payload must be a regular file confined to the selected root",
            ));
        }

        let bytes = self
            .source
            .read(&payload_path, MAX_DERIVED_PAGE_PAYLOAD_BYTES)?;
        let payload: DerivedPagePayload = serde_json::from_slice(&bytes).map_err(|error| {
            RendererError::new(format!("malformed derived-page payload: {error}"))
        })?;
        validate_payload(&payload, request.root())?;
        render_payload(payload, request, payload_path)
    }
}

fn validate_payload(payload: &DerivedPagePayload, root: &Path) -> Result<(), RendererError> {
    if payload.schema_version != "workbench.derived-page-context.v1" {
        return Err(RendererError::new(format!(
            "unsupported derived-page schema {:?}",
            payload.schema_version
        )));
    }
    validate_text("provider id", &payload.provider.id, MAX_ID_BYTES)?;
    validate_text("provider label", &payload.provider.label, MAX_HEADING_BYTES)?;
    validate_text("page id", &payload.page.id, MAX_ID_BYTES)?;
    validate_text("page title", &payload.page.title, MAX_HEADING_BYTES)?;
    if payload.page.sections.is_empty() || payload.page.sections.len() > MAX_SECTIONS {
        return Err(RendererError::new(format!(
            "derived page needs 1..={MAX_SECTIONS} sections"
        )));
    }

    let mut section_ids = HashSet::new();
    for section in &payload.page.sections {
        validate_text("section id", &section.id, MAX_ID_BYTES)?;
        validate_text("section heading", &section.heading, MAX_HEADING_BYTES)?;
        validate_text(
            "section Markdown",
            &section.body_markdown,
            MAX_SECTION_MARKDOWN_BYTES,
        )?;
        if !section_ids.insert(section.id.as_str()) {
            return Err(RendererError::new(format!(
                "duplicate derived section id {:?}",
                section.id
            )));
        }
        if section.citations.is_empty()
            || section.citations.len() > MAX_CITATIONS_PER_SECTION
        {
            return Err(RendererError::new(format!(
                "each section needs 1..={MAX_CITATIONS_PER_SECTION} citations"
            )));
        }
        if section.classification == ProvenanceClassification::Extracted
            && section.citations.len() != 1
        {
            return Err(RendererError::new(
                "an extracted section must identify exactly one citation",
            ));
        }
        if matches!(section.freshness, PageFreshness::Stale | PageFreshness::Unknown)
            && section
                .freshness_note
                .as_deref()
                .is_none_or(|note| note.trim().is_empty())
        {
            return Err(RendererError::new(
                "stale and unknown freshness need a visible explanation",
            ));
        }
        if let Some(note) = &section.freshness_note {
            validate_text("freshness note", note, MAX_FRESHNESS_NOTE_BYTES)?;
        }

        let view = provenance_item(
            ProviderIdentity::new("validation", "Derived page validation"),
            section.classification,
            section.citations.clone(),
        )
        .resolve(root);
        if view
            .sources
            .iter()
            .any(|source| matches!(source.status, SourceStatus::Invalid { .. }))
        {
            return Err(RendererError::new(
                "citation paths and spans must be confined, relative, and one-based",
            ));
        }
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

fn provenance_item(
    provider: ProviderIdentity,
    classification: ProvenanceClassification,
    citations: Vec<SourceLocation>,
) -> ContextProvenanceItem {
    match classification {
        ProvenanceClassification::Extracted => {
            ContextProvenanceItem::extracted(provider, citations[0].clone())
        }
        ProvenanceClassification::Inferred => {
            ContextProvenanceItem::inferred(provider, citations)
        }
        ProvenanceClassification::Ambiguous => {
            ContextProvenanceItem::ambiguous(provider, citations)
        }
    }
}

fn render_payload(
    payload: DerivedPagePayload,
    request: &ContextRequest,
    payload_path: PathBuf,
) -> Result<ContextDocument, RendererError> {
    let mut navigation = Vec::new();
    let mut canonical_sources = BTreeSet::new();
    let mut sections_html = String::new();

    for section in payload.page.sections {
        let view = provenance_item(
            payload.provider.clone(),
            section.classification,
            section.citations,
        )
        .resolve(request.root());
        sections_html.push_str(&render_section(
            &section,
            &view,
            request.root(),
            &mut navigation,
            &mut canonical_sources,
        ));
    }
    canonical_sources.insert(payload_path);

    Ok(ContextDocument {
        renderer_id: "derived-page-context".to_owned(),
        kind: ContextDocumentKind::DerivedPage,
        title: payload.page.title.clone(),
        body_html: format!(
            "<section class=\"derived-page\" data-page-id=\"{}\"><p class=\"authority\">Derived compatibility view from {}. Raw repository sources remain authoritative.</p>{}</section>",
            escape_html(&payload.page.id),
            escape_html(&payload.provider.label),
            sections_html
        ),
        navigation,
        warnings: Vec::new(),
        provenance: ContextProvenance {
            root: request.root().to_path_buf(),
            sources: canonical_sources.into_iter().collect(),
        },
    })
}

fn render_section(
    section: &DerivedSection,
    view: &ProvenanceView,
    root: &Path,
    navigation: &mut Vec<ContextNavigation>,
    canonical_sources: &mut BTreeSet<PathBuf>,
) -> String {
    let freshness_note = section.freshness_note.as_deref().unwrap_or(
        "provider reported this page section current; source remains authoritative",
    );
    let freshness_badge = format!(
        "Provider-reported {} · {}",
        section.freshness.label(),
        freshness_note
    );
    let mut citations_html = String::new();
    for source in &view.sources {
        let requested = source.requested.relative_path.display().to_string();
        let status = match &source.status {
            SourceStatus::Canonical => "canonical",
            SourceStatus::Missing => "missing",
            SourceStatus::Invalid { .. } => "invalid",
        };
        citations_html.push_str(&format!(
            "<li data-citation-status=\"{}\">{}</li>",
            status,
            escape_html(&requested)
        ));
        if let Some(link) = &source.navigation {
            let line = link.span.map(|span| span.start.line);
            let label = line.map_or_else(
                || format!("section {}: {}", section.id, link.relative_path.display()),
                |line| {
                    format!(
                        "section {}: {}:{line}",
                        section.id,
                        link.relative_path.display()
                    )
                },
            );
            navigation.push(ContextNavigation {
                label,
                path: link.relative_path.clone(),
            });
            if let Ok(canonical) = fs::canonicalize(root.join(&link.relative_path)) {
                canonical_sources.insert(canonical);
            }
        }
    }

    format!(
        "<article class=\"derived-section\" data-section-id=\"{}\"><h2>{}</h2><p class=\"provenance-badge\">{}</p><p class=\"freshness-badge\">{}</p><div class=\"derived-body\">{}</div><h3>Citations</h3><ul>{}</ul></article>",
        escape_html(&section.id),
        escape_html(&section.heading),
        escape_html(&view.badge),
        escape_html(&freshness_badge),
        safe_markdown_html(&section.body_markdown),
        citations_html
    )
}
// HANDWRITE-END
