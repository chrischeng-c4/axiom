// HANDWRITE-BEGIN gap="missing-generator:logic:19ae9215" tracker="pending-tracker" reason="Define provider identity, extraction classification, confined file/span inputs, canonical/missing/invalid resolution, visible authority labels, and source navigation."
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderIdentity {
    pub id: String,
    pub label: String,
}

impl ProviderIdentity {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceClassification {
    Extracted,
    Inferred,
    Ambiguous,
}

impl ProvenanceClassification {
    fn label(self) -> &'static str {
        match self {
            Self::Extracted => "Extracted",
            Self::Inferred => "Inferred",
            Self::Ambiguous => "Ambiguous",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePosition {
    pub line: u32,
    pub column: u32,
}

impl SourcePosition {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

impl SourceSpan {
    pub fn new(start: SourcePosition, end: SourcePosition) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub relative_path: PathBuf,
    pub span: Option<SourceSpan>,
}

impl SourceLocation {
    pub fn file(relative_path: impl Into<PathBuf>) -> Self {
        Self {
            relative_path: relative_path.into(),
            span: None,
        }
    }

    pub fn with_span(relative_path: impl Into<PathBuf>, span: SourceSpan) -> Self {
        Self {
            relative_path: relative_path.into(),
            span: Some(span),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextProvenanceItem {
    pub provider: ProviderIdentity,
    pub classification: ProvenanceClassification,
    pub sources: Vec<SourceLocation>,
}

impl ContextProvenanceItem {
    pub fn extracted(provider: ProviderIdentity, source: SourceLocation) -> Self {
        Self {
            provider,
            classification: ProvenanceClassification::Extracted,
            sources: vec![source],
        }
    }

    pub fn inferred(provider: ProviderIdentity, sources: Vec<SourceLocation>) -> Self {
        Self {
            provider,
            classification: ProvenanceClassification::Inferred,
            sources,
        }
    }

    pub fn ambiguous(provider: ProviderIdentity, sources: Vec<SourceLocation>) -> Self {
        Self {
            provider,
            classification: ProvenanceClassification::Ambiguous,
            sources,
        }
    }

    pub fn resolve(&self, root: impl AsRef<Path>) -> ProvenanceView {
        let root = root.as_ref();
        let canonical_root = fs::canonicalize(root)
            .ok()
            .filter(|candidate| candidate.is_dir());
        let sources: Vec<ResolvedSource> = self
            .sources
            .iter()
            .cloned()
            .map(|source| resolve_source(canonical_root.as_deref(), source))
            .collect();
        let unavailable = sources
            .iter()
            .filter(|source| !matches!(source.status, SourceStatus::Canonical))
            .count();
        let authority = match self.classification {
            ProvenanceClassification::Extracted if sources.len() == 1 && unavailable == 0 => {
                ProvenanceAuthority::Canonical
            }
            ProvenanceClassification::Extracted => ProvenanceAuthority::Unavailable,
            ProvenanceClassification::Inferred | ProvenanceClassification::Ambiguous => {
                ProvenanceAuthority::Derived
            }
        };
        let badge = authority_badge(
            self.classification,
            authority,
            &self.provider,
            sources.len(),
            unavailable,
        );

        ProvenanceView {
            provider: self.provider.clone(),
            classification: self.classification,
            authority,
            badge,
            sources,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceAuthority {
    Canonical,
    Derived,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum SourceStatus {
    Canonical,
    Missing,
    Invalid { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceNavigation {
    pub relative_path: PathBuf,
    pub span: Option<SourceSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSource {
    pub requested: SourceLocation,
    pub status: SourceStatus,
    pub navigation: Option<ProvenanceNavigation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceView {
    pub provider: ProviderIdentity,
    pub classification: ProvenanceClassification,
    pub authority: ProvenanceAuthority,
    pub badge: String,
    pub sources: Vec<ResolvedSource>,
}

fn resolve_source(canonical_root: Option<&Path>, requested: SourceLocation) -> ResolvedSource {
    let Some(root) = canonical_root else {
        return invalid(requested, "selected root is missing or is not a directory");
    };
    if !safe_relative_path(&requested.relative_path) {
        return invalid(requested, "source path is not a confined relative path");
    }
    if requested.span.is_some_and(|span| !valid_span(span)) {
        return invalid(requested, "source span must be ordered and one-based");
    }

    let candidate = root.join(&requested.relative_path);
    if !candidate.exists() {
        return ResolvedSource {
            requested,
            status: SourceStatus::Missing,
            navigation: None,
        };
    }
    let Ok(canonical) = fs::canonicalize(&candidate) else {
        return invalid(requested, "source path could not be canonicalized");
    };
    if !canonical.starts_with(root) {
        return invalid(requested, "source path escapes the selected root");
    }
    if !canonical.is_file() {
        return invalid(requested, "source location is not a regular file");
    }
    let relative_path = canonical
        .strip_prefix(root)
        .expect("confined canonical source has root prefix")
        .to_path_buf();

    ResolvedSource {
        navigation: Some(ProvenanceNavigation {
            relative_path,
            span: requested.span,
        }),
        requested,
        status: SourceStatus::Canonical,
    }
}

fn invalid(requested: SourceLocation, reason: impl Into<String>) -> ResolvedSource {
    ResolvedSource {
        requested,
        status: SourceStatus::Invalid {
            reason: reason.into(),
        },
        navigation: None,
    }
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn valid_span(span: SourceSpan) -> bool {
    if span.start.line == 0 || span.start.column == 0 || span.end.line == 0 || span.end.column == 0
    {
        return false;
    }
    (span.start.line, span.start.column) < (span.end.line, span.end.column)
}

fn authority_badge(
    classification: ProvenanceClassification,
    authority: ProvenanceAuthority,
    provider: &ProviderIdentity,
    input_count: usize,
    unavailable: usize,
) -> String {
    let authority_label = match authority {
        ProvenanceAuthority::Canonical => "canonical source".to_owned(),
        ProvenanceAuthority::Derived => format!("derived from {input_count} source input(s)"),
        ProvenanceAuthority::Unavailable => "non-authoritative source".to_owned(),
    };
    let unavailable_label = if unavailable == 0 {
        String::new()
    } else {
        format!(" · {unavailable} unavailable")
    };
    format!(
        "{} · {} · {}{}",
        classification.label(),
        authority_label,
        provider.label,
        unavailable_label
    )
}
// HANDWRITE-END
