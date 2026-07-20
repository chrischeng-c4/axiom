// HANDWRITE-BEGIN gap="missing-generator:unit-test:b0a0ded1" tracker="pending-tracker" reason="Prove non-AW Markdown and Git rendering, deterministic selection, failure isolation, safe output, and navigable fallback."
use std::{fs, path::Path, process::Command};

use tempfile::TempDir;
use workbench::context::{
    ContextDocument, ContextDocumentKind, ContextProvenance, ContextRenderer, ContextRequest,
    MarkdownRenderer, RendererError, RendererRegistry, RendererSupport,
};

struct GitFixture {
    _temporary_directory: TempDir,
    root: std::path::PathBuf,
}

impl GitFixture {
    fn new() -> Self {
        let temporary_directory = TempDir::new().expect("temporary fixture");
        let root = temporary_directory.path().to_path_buf();
        run_git(&root, &["init", "--quiet"]);
        run_git(&root, &["config", "user.email", "workbench@example.invalid"]);
        run_git(&root, &["config", "user.name", "Workbench Test"]);
        fs::write(root.join("README.md"), "# Baseline\n").expect("baseline Markdown");
        run_git(&root, &["add", "README.md"]);
        run_git(&root, &["commit", "--quiet", "-m", "baseline"]);

        fs::write(
            root.join("README.md"),
            "# Workbench fixture\n\n<script>alert('no')</script>\n\n[unsafe](javascript:alert(1))\n",
        )
        .expect("modified Markdown");
        fs::create_dir(root.join("docs")).expect("docs directory");
        fs::write(root.join("docs/guide.md"), "# Guide\n").expect("untracked Markdown");

        Self {
            _temporary_directory: temporary_directory,
            root,
        }
    }
}

fn run_git(root: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .status()
        .expect("git command launches");
    assert!(status.success(), "git {} failed", arguments.join(" "));
}

struct StubRenderer {
    id: &'static str,
    priority: i32,
    fails: bool,
}

impl ContextRenderer for StubRenderer {
    fn id(&self) -> &'static str {
        self.id
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn supports(&self, _: &ContextRequest) -> RendererSupport {
        RendererSupport::Supported
    }

    fn render(&self, request: &ContextRequest) -> Result<ContextDocument, RendererError> {
        if self.fails {
            return Err(RendererError::new("intentional renderer failure"));
        }
        Ok(ContextDocument {
            renderer_id: self.id.to_owned(),
            kind: ContextDocumentKind::Fallback,
            title: self.id.to_owned(),
            body_html: format!("<p>{}</p>", self.id),
            navigation: Vec::new(),
            warnings: Vec::new(),
            provenance: ContextProvenance {
                root: request.root().to_path_buf(),
                sources: Vec::new(),
            },
        })
    }
}

#[test]
fn selection_is_deterministic_and_failures_are_isolated() {
    let fixture = GitFixture::new();
    let request = ContextRequest::file(&fixture.root, "README.md").expect("file request");

    let mut isolated = RendererRegistry::new();
    isolated.register(StubRenderer {
        id: "broken",
        priority: 500,
        fails: true,
    });
    isolated.register(MarkdownRenderer::new());
    let document = isolated.render(&request);
    assert_eq!(document.renderer_id, "markdown");
    assert_eq!(document.kind, ContextDocumentKind::Markdown);
    assert!(document.warnings.iter().any(|warning| {
        warning.contains("broken") && warning.contains("intentional renderer failure")
    }));

    let mut tied = RendererRegistry::new();
    tied.register(StubRenderer {
        id: "zeta",
        priority: 10,
        fails: false,
    });
    tied.register(StubRenderer {
        id: "alpha",
        priority: 10,
        fails: false,
    });
    assert_eq!(tied.render(&request).renderer_id, "alpha");
}

#[test]
fn non_aw_fixture_renders_markdown_and_git_context() {
    let fixture = GitFixture::new();
    assert!(!fixture.root.join("aw.toml").exists());
    let registry = RendererRegistry::generic();

    let markdown = registry.render(
        &ContextRequest::file(&fixture.root, "README.md").expect("Markdown request"),
    );
    assert_eq!(markdown.kind, ContextDocumentKind::Markdown);
    assert!(markdown.body_html.contains("<h1>Workbench fixture</h1>"));
    assert!(markdown.body_html.contains("&lt;script&gt;"));
    assert!(!markdown.body_html.contains("<script"));
    assert!(!markdown.body_html.contains("javascript:"));
    assert_eq!(markdown.navigation[0].path, Path::new("README.md"));
    assert!(markdown.provenance.sources[0].ends_with("README.md"));

    let git = registry.render(&ContextRequest::workspace(&fixture.root).expect("workspace request"));
    assert_eq!(git.kind, ContextDocumentKind::Git);
    assert!(git.body_html.contains("README.md"));
    assert!(git.body_html.contains("Workbench fixture"));
    assert!(git.navigation.iter().any(|item| item.path == Path::new("README.md")));
    assert!(git.navigation.iter().any(|item| item.path == Path::new("docs/guide.md")));
}

#[test]
fn unsupported_and_corrupt_artifacts_have_navigable_fallbacks() {
    let fixture = GitFixture::new();
    fs::write(fixture.root.join("corrupt.md"), [0xff, 0xfe]).expect("invalid UTF-8 fixture");
    fs::write(fixture.root.join("artifact.bin"), [0, 1, 2]).expect("unsupported fixture");
    let registry = RendererRegistry::generic();

    for target in ["corrupt.md", "artifact.bin", "missing.md"] {
        let document = registry.render(
            &ContextRequest::file(&fixture.root, target).expect("confined request"),
        );
        assert_eq!(document.kind, ContextDocumentKind::Fallback);
        assert_eq!(document.renderer_id, "fallback");
        assert_eq!(document.navigation[0].path, Path::new(target));
        assert!(!document.warnings.is_empty());
    }

    assert!(ContextRequest::file(&fixture.root, "../outside.md").is_err());
    assert!(ContextRequest::file(&fixture.root, "/tmp/outside.md").is_err());
}

#[test]
fn renderers_are_path_confined_and_runtime_independent() {
    let fixture = GitFixture::new();
    let before = fs::read(fixture.root.join("README.md")).expect("before snapshot");
    let registry = RendererRegistry::generic();
    let _ = registry.render(
        &ContextRequest::file(&fixture.root, "README.md").expect("Markdown request"),
    );
    let _ = registry.render(&ContextRequest::workspace(&fixture.root).expect("Git request"));
    let after = fs::read(fixture.root.join("README.md")).expect("after snapshot");
    assert_eq!(before, after, "rendering must not mutate workspace files");

    let implementation = [
        include_str!("../src/context/mod.rs"),
        include_str!("../src/context/markdown.rs"),
        include_str!("../src/context/git.rs"),
    ]
    .join("\n");
    for forbidden in [
        "native_agent_pty",
        "cwd_context",
        "folder_shell",
        "Command::new(\"aw\")",
        "aw.toml",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "generic renderer unexpectedly depends on {forbidden}"
        );
    }
    assert!(implementation.contains("GIT_OPTIONAL_LOCKS"));
}
// HANDWRITE-END
