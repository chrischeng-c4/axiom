// HANDWRITE-BEGIN gap="missing-generator:logic:6fd8b17e" tracker="pending-tracker" reason="Define renderer requests, structured documents, deterministic registry selection, error isolation, fallback, and path confinement."
pub mod git;
pub mod markdown;

use std::{
    fmt,
    fs,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub use git::GitRenderer;
pub use markdown::MarkdownRenderer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextTarget {
    Workspace,
    File(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextRequest {
    root: PathBuf,
    target: ContextTarget,
}

impl ContextRequest {
    pub fn workspace(root: impl AsRef<Path>) -> Result<Self, RendererError> {
        Ok(Self {
            root: canonical_directory(root.as_ref())?,
            target: ContextTarget::Workspace,
        })
    }

    pub fn file(
        root: impl AsRef<Path>,
        relative_path: impl AsRef<Path>,
    ) -> Result<Self, RendererError> {
        let relative_path = relative_path.as_ref();
        if !safe_relative_path(relative_path) {
            return Err(RendererError::new(format!(
                "context target must be a confined relative path: {}",
                relative_path.display()
            )));
        }

        Ok(Self {
            root: canonical_directory(root.as_ref())?,
            target: ContextTarget::File(relative_path.to_path_buf()),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn target(&self) -> &ContextTarget {
        &self.target
    }

    pub fn resolve_target(&self) -> Result<PathBuf, RendererError> {
        match &self.target {
            ContextTarget::Workspace => Ok(self.root.clone()),
            ContextTarget::File(relative_path) => {
                let candidate = fs::canonicalize(self.root.join(relative_path)).map_err(|error| {
                    RendererError::new(format!(
                        "could not resolve context target {}: {error}",
                        relative_path.display()
                    ))
                })?;
                if !candidate.starts_with(&self.root) {
                    return Err(RendererError::new(format!(
                        "context target escapes the selected root: {}",
                        relative_path.display()
                    )));
                }
                Ok(candidate)
            }
        }
    }

    fn navigation_path(&self) -> PathBuf {
        match &self.target {
            ContextTarget::Workspace => PathBuf::from("."),
            ContextTarget::File(path) => path.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextDocumentKind {
    Markdown,
    Git,
    Fallback,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextNavigation {
    pub label: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextProvenance {
    pub root: PathBuf,
    pub sources: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextDocument {
    pub renderer_id: String,
    pub kind: ContextDocumentKind,
    pub title: String,
    pub body_html: String,
    pub navigation: Vec<ContextNavigation>,
    pub warnings: Vec<String>,
    pub provenance: ContextProvenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererSupport {
    Unsupported,
    Supported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererError {
    message: String,
}

impl RendererError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RendererError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RendererError {}

pub trait ContextRenderer: Send + Sync {
    fn id(&self) -> &'static str;
    fn priority(&self) -> i32;
    fn supports(&self, request: &ContextRequest) -> RendererSupport;
    fn render(&self, request: &ContextRequest) -> Result<ContextDocument, RendererError>;
}

#[derive(Default)]
pub struct RendererRegistry {
    renderers: Vec<Box<dyn ContextRenderer>>,
}

impl RendererRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generic() -> Self {
        let mut registry = Self::new();
        registry.register(MarkdownRenderer::new());
        registry.register(GitRenderer::new());
        registry
    }

    pub fn register(&mut self, renderer: impl ContextRenderer + 'static) {
        self.renderers.push(Box::new(renderer));
    }

    pub fn render(&self, request: &ContextRequest) -> ContextDocument {
        let mut candidates: Vec<&dyn ContextRenderer> = self
            .renderers
            .iter()
            .map(Box::as_ref)
            .filter(|renderer| renderer.supports(request) == RendererSupport::Supported)
            .collect();
        candidates.sort_by(|left, right| {
            right
                .priority()
                .cmp(&left.priority())
                .then_with(|| left.id().cmp(right.id()))
        });

        let mut warnings = Vec::new();
        for renderer in candidates {
            match renderer.render(request) {
                Ok(mut document) => {
                    warnings.append(&mut document.warnings);
                    document.warnings = warnings;
                    return document;
                }
                Err(error) => warnings.push(format!("{}: {error}", renderer.id())),
            }
        }

        if warnings.is_empty() {
            warnings.push("No registered renderer supports this context target".to_owned());
        }
        fallback_document(request, warnings)
    }
}

fn fallback_document(request: &ContextRequest, warnings: Vec<String>) -> ContextDocument {
    let navigation_path = request.navigation_path();
    let label = navigation_path.display().to_string();
    ContextDocument {
        renderer_id: "fallback".to_owned(),
        kind: ContextDocumentKind::Fallback,
        title: "Context preview unavailable".to_owned(),
        body_html: format!(
            "<section><h2>Preview unavailable</h2><p>{}</p></section>",
            escape_html(warnings.first().map(String::as_str).unwrap_or("Unknown error"))
        ),
        navigation: vec![ContextNavigation {
            label,
            path: navigation_path,
        }],
        warnings,
        provenance: ContextProvenance {
            root: request.root.clone(),
            sources: vec![request.root.join(request.navigation_path())],
        },
    }
}

fn canonical_directory(path: &Path) -> Result<PathBuf, RendererError> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        RendererError::new(format!("could not resolve context root {}: {error}", path.display()))
    })?;
    if !canonical.is_dir() {
        return Err(RendererError::new(format!(
            "context root is not a directory: {}",
            path.display()
        )));
    }
    Ok(canonical)
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
// HANDWRITE-END
