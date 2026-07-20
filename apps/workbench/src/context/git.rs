// HANDWRITE-BEGIN gap="missing-generator:logic:cf86eb66" tracker="pending-tracker" reason="Render read-only Git status, diff stat, diff, and changed-path navigation for ordinary repositories."
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
    process::Command,
};

use super::{
    escape_html, ContextDocument, ContextDocumentKind, ContextNavigation, ContextProvenance,
    ContextRenderer, ContextRequest, ContextTarget, RendererError, RendererSupport,
};

const MAX_GIT_OUTPUT_BYTES: usize = 256 * 1024;

#[derive(Debug, Default)]
pub struct GitRenderer;

impl GitRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl ContextRenderer for GitRenderer {
    fn id(&self) -> &'static str {
        "git"
    }

    fn priority(&self) -> i32 {
        100
    }

    fn supports(&self, request: &ContextRequest) -> RendererSupport {
        if !matches!(request.target(), ContextTarget::Workspace) {
            return RendererSupport::Unsupported;
        }

        match run_git(request.root(), &["rev-parse", "--is-inside-work-tree"]) {
            Ok(output) if output.trim() == "true" => RendererSupport::Supported,
            _ => RendererSupport::Unsupported,
        }
    }

    fn render(&self, request: &ContextRequest) -> Result<ContextDocument, RendererError> {
        let status = run_git(
            request.root(),
            &["status", "--short", "--branch", "--untracked-files=all"],
        )?;
        let diff_stat = run_git(request.root(), &["diff", "--no-ext-diff", "--stat"])?;
        let diff = run_git(request.root(), &["diff", "--no-ext-diff", "--"])?;

        let navigation = changed_paths(&status)
            .into_iter()
            .map(|path| ContextNavigation {
                label: path.display().to_string(),
                path,
            })
            .collect();

        Ok(ContextDocument {
            renderer_id: self.id().to_owned(),
            kind: ContextDocumentKind::Git,
            title: "Git working tree".to_owned(),
            body_html: format!(
                "<section><h2>Status</h2><pre>{}</pre><h2>Diff summary</h2><pre>{}</pre><h2>Diff</h2><pre>{}</pre></section>",
                escape_html(&status),
                escape_html(&diff_stat),
                escape_html(&diff)
            ),
            navigation,
            warnings: Vec::new(),
            provenance: ContextProvenance {
                root: request.root().to_path_buf(),
                sources: vec![request.root().to_path_buf()],
            },
        })
    }
}

fn run_git(root: &Path, arguments: &[&str]) -> Result<String, RendererError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|error| RendererError::new(format!("could not run git: {error}")))?;

    if !output.status.success() {
        let stderr = bounded_text(&output.stderr);
        return Err(RendererError::new(format!(
            "git {} failed: {}",
            arguments.join(" "),
            stderr.trim()
        )));
    }

    Ok(bounded_text(&output.stdout))
}

fn bounded_text(bytes: &[u8]) -> String {
    if bytes.len() <= MAX_GIT_OUTPUT_BYTES {
        return String::from_utf8_lossy(bytes).into_owned();
    }

    let mut text = String::from_utf8_lossy(&bytes[..MAX_GIT_OUTPUT_BYTES]).into_owned();
    text.push_str("\n[output truncated by Workbench]\n");
    text
}

fn changed_paths(status: &str) -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    for line in status.lines().filter(|line| !line.starts_with("## ")) {
        let Some(raw_path) = line.get(3..) else {
            continue;
        };
        let raw_path = raw_path
            .rsplit_once(" -> ")
            .map_or(raw_path, |(_, path)| path);
        let path = PathBuf::from(raw_path.trim_matches('"'));
        if safe_relative_path(&path) {
            paths.insert(path);
        }
    }
    paths.into_iter().collect()
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}
// HANDWRITE-END
