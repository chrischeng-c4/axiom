// HANDWRITE-BEGIN gap="missing-generator:unit-test:65982fdc" tracker="pending-tracker" reason="Prove every derived section has canonical citation navigation or a visible inference label, freshness disclosure, absence and failure isolation, sentinel independence, and no mutation."
use std::{fs, path::Path, process::Command};

use tempfile::TempDir;
use workbench::{
    context::{
        derived_page::{
            DerivedPageContextRenderer, DerivedPagePayloadSource,
            MAX_DERIVED_PAGE_PAYLOAD_BYTES,
        },
        ContextDocument, ContextDocumentKind, ContextProvenance, ContextRenderer, ContextRequest,
        RendererError, RendererRegistry, RendererSupport,
    },
    native_agent_pty::{PtyCommand, PtyRuntime, PtySize},
};

const PAGE_FIXTURE: &str =
    include_str!("fixtures/derived-page/llm-wiki-out/workbench-pages.json");
const LIB_FIXTURE: &str = include_str!("fixtures/derived-page/src/lib.rs");
const ARCHITECTURE_FIXTURE: &str = include_str!("fixtures/derived-page/docs/architecture.md");

struct PageFixture {
    _temporary_directory: TempDir,
    root: std::path::PathBuf,
}

impl PageFixture {
    fn new() -> Self {
        let temporary_directory = TempDir::new().expect("temporary page fixture");
        let root = temporary_directory.path().to_path_buf();
        fs::create_dir_all(root.join("llm-wiki-out")).expect("derived output directory");
        fs::create_dir_all(root.join("src")).expect("source directory");
        fs::create_dir_all(root.join("docs")).expect("docs directory");
        fs::write(root.join("llm-wiki-out/workbench-pages.json"), PAGE_FIXTURE)
            .expect("derived page payload");
        fs::write(root.join("src/lib.rs"), LIB_FIXTURE).expect("raw Rust source");
        fs::write(root.join("docs/architecture.md"), ARCHITECTURE_FIXTURE)
            .expect("raw architecture source");
        initialize_git(&root);
        Self {
            _temporary_directory: temporary_directory,
            root,
        }
    }
}

fn initialize_git(root: &Path) {
    for arguments in [
        vec!["init", "--quiet"],
        vec!["config", "user.email", "workbench@example.invalid"],
        vec!["config", "user.name", "Workbench Test"],
        vec!["add", "."],
        vec!["commit", "--quiet", "-m", "fixture"],
    ] {
        let status = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(&arguments)
            .env("GIT_OPTIONAL_LOCKS", "0")
            .status()
            .expect("git fixture command");
        assert!(status.success(), "git {} failed", arguments.join(" "));
    }
}

fn size() -> PtySize {
    PtySize {
        rows: 12,
        cols: 40,
        pixel_width: 0,
        pixel_height: 0,
    }
}

struct SentinelRenderer;

impl ContextRenderer for SentinelRenderer {
    fn id(&self) -> &'static str {
        "aw-registry-sentinel"
    }

    fn priority(&self) -> i32 {
        300
    }

    fn supports(&self, request: &ContextRequest) -> RendererSupport {
        match request.target() {
            workbench::context::ContextTarget::File(path)
                if path == Path::new("sentinel.aw") => RendererSupport::Supported,
            _ => RendererSupport::Unsupported,
        }
    }

    fn render(&self, request: &ContextRequest) -> Result<ContextDocument, RendererError> {
        Ok(ContextDocument {
            renderer_id: self.id().to_owned(),
            kind: ContextDocumentKind::Fallback,
            title: "AW sentinel".to_owned(),
            body_html: "<p>sentinel</p>".to_owned(),
            navigation: Vec::new(),
            warnings: Vec::new(),
            provenance: ContextProvenance {
                root: request.root().to_path_buf(),
                sources: Vec::new(),
            },
        })
    }
}

struct FailingSource;

impl DerivedPagePayloadSource for FailingSource {
    fn read(&self, _: &Path, _: usize) -> Result<Vec<u8>, RendererError> {
        Err(RendererError::new("injected derived-page provider failure"))
    }
}

#[test]
fn renders_canonical_citation_or_visible_inference_for_every_section() {
    let fixture = PageFixture::new();
    let document = RendererRegistry::production()
        .render(&ContextRequest::workspace(&fixture.root).expect("workspace context"));
    assert_eq!(document.kind, ContextDocumentKind::DerivedPage);
    assert_eq!(document.renderer_id, "derived-page-context");

    let payload: serde_json::Value = serde_json::from_str(PAGE_FIXTURE).expect("fixture JSON");
    for section in payload["page"]["sections"].as_array().unwrap() {
        let id = section["id"].as_str().unwrap();
        let classification = section["classification"].as_str().unwrap();
        let start = document
            .body_html
            .find(&format!("data-section-id=\"{id}\""))
            .unwrap_or_else(|| panic!("missing rendered section {id}"));
        let article = &document.body_html[start..];
        let article = &article[..article.find("</article>").expect("article close")];
        let has_citation = document
            .navigation
            .iter()
            .any(|link| link.label.starts_with(&format!("section {id}:")));
        let classification_label = match classification {
            "extracted" => "Extracted",
            "inferred" => "Inferred",
            "ambiguous" => "Ambiguous",
            other => panic!("unexpected classification {other}"),
        };
        let has_visible_inference = classification != "extracted"
            && article.contains(classification_label)
            && article.contains("derived from");
        assert!(
            has_citation || has_visible_inference,
            "section {id} has neither canonical navigation nor a visible inference label"
        );
        for citation in section["citations"].as_array().unwrap() {
            let path = citation["relative_path"].as_str().unwrap();
            assert!(article.contains(path), "section {id} omitted citation {path}");
        }
    }
    for freshness in ["Current", "Stale", "Unknown"] {
        assert!(
            document
                .body_html
                .contains(&format!("Provider-reported {freshness}")),
            "missing {freshness} freshness disclosure"
        );
    }
    assert!(document.body_html.contains("Raw repository sources remain authoritative"));
    assert!(document.body_html.contains("&lt;script&gt;"));
    assert!(!document.body_html.contains("<script"));
    assert!(!document.body_html.contains("javascript:"));
}

#[test]
fn provider_absence_leaves_generic_and_aw_sentinel_renderers_usable() {
    let temporary_directory = TempDir::new().expect("absence fixture");
    let root = temporary_directory.path();
    fs::write(root.join("README.md"), "# Generic Markdown\n").expect("Markdown fixture");
    fs::write(root.join("sentinel.aw"), "sentinel\n").expect("sentinel fixture");
    initialize_git(root);
    assert!(!root.join("llm-wiki-out/workbench-pages.json").exists());

    let production = RendererRegistry::production();
    assert_eq!(
        production
            .render(&ContextRequest::file(root, "README.md").expect("Markdown request"))
            .kind,
        ContextDocumentKind::Markdown
    );
    assert_eq!(
        production
            .render(&ContextRequest::workspace(root).expect("Git request"))
            .kind,
        ContextDocumentKind::Git
    );

    let mut isolated = RendererRegistry::new();
    isolated.register(DerivedPageContextRenderer::new());
    isolated.register(SentinelRenderer);
    assert_eq!(
        isolated
            .render(&ContextRequest::file(root, "sentinel.aw").expect("sentinel request"))
            .renderer_id,
        "aw-registry-sentinel"
    );

    #[cfg(unix)]
    {
        let status = PtyRuntime::default()
            .spawn(
                &PtyCommand::new("/bin/sh", root).args(["-c", "exit 0"]),
                size(),
            )
            .expect("PTY remains launchable")
            .wait()
            .expect("PTY child is reaped");
        assert!(status.success());
    }

    let test_source = include_str!("derived_page_context_adapter.rs");
    let concrete_renderer = ["Aw", "Typed", "Renderer"].concat();
    assert!(!test_source.contains(&concrete_renderer));
}

#[test]
fn malformed_and_failing_provider_are_isolated() {
    let fixture = PageFixture::new();
    fs::write(
        fixture.root.join("llm-wiki-out/workbench-pages.json"),
        "{ malformed",
    )
    .expect("malformed payload");
    fs::write(fixture.root.join("dirty.txt"), "force Git context\n").expect("dirty source");
    let request = ContextRequest::workspace(&fixture.root).expect("workspace request");
    let malformed = RendererRegistry::production().render(&request);
    assert_eq!(malformed.kind, ContextDocumentKind::Git);
    assert!(malformed.warnings.iter().any(|warning| {
        warning.contains("derived-page-context") && warning.contains("malformed")
    }));

    fs::write(
        fixture.root.join("llm-wiki-out/workbench-pages.json"),
        PAGE_FIXTURE,
    )
    .expect("restore valid payload");
    let mut isolated = RendererRegistry::generic();
    isolated.register(DerivedPageContextRenderer::with_source(FailingSource));
    let failure = isolated.render(&request);
    assert_eq!(failure.kind, ContextDocumentKind::Git);
    assert!(failure.warnings.iter().any(|warning| {
        warning.contains("derived-page-context")
            && warning.contains("injected derived-page provider failure")
    }));
}

#[test]
fn adapter_contract_is_reference_only_and_read_only() {
    let fixture = PageFixture::new();
    let watched = [
        fixture.root.join("llm-wiki-out/workbench-pages.json"),
        fixture.root.join("src/lib.rs"),
        fixture.root.join("docs/architecture.md"),
    ];
    let before: Vec<Vec<u8>> = watched
        .iter()
        .map(|path| fs::read(path).expect("before bytes"))
        .collect();
    let _ = RendererRegistry::production()
        .render(&ContextRequest::workspace(&fixture.root).expect("workspace request"));
    let after: Vec<Vec<u8>> = watched
        .iter()
        .map(|path| fs::read(path).expect("after bytes"))
        .collect();
    assert_eq!(before, after, "derived-page rendering mutated input files");

    let implementation = include_str!("../src/context/derived_page.rs");
    for forbidden in [
        "llm_wiki::",
        "Command::new",
        "std::process",
        "fs::write",
        "fs::remove",
        "fs::rename",
        "native_agent_pty",
        "aw::",
        "github",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "derived-page adapter unexpectedly exposes {forbidden}"
        );
    }
    assert!(implementation.contains("workbench.derived-page-context.v1"));
    assert!(implementation.contains("File::open"));
}

#[test]
fn payload_limits_freshness_and_citation_confinement_fail_closed() {
    let fixture = PageFixture::new();
    let request = ContextRequest::workspace(&fixture.root).expect("workspace request");
    let renderer = DerivedPageContextRenderer::new();

    fs::write(
        fixture.root.join("llm-wiki-out/workbench-pages.json"),
        vec![b' '; MAX_DERIVED_PAGE_PAYLOAD_BYTES + 1],
    )
    .expect("oversized payload");
    assert!(renderer
        .render(&request)
        .expect_err("oversized page must fail")
        .to_string()
        .contains("exceeds"));

    let unsafe_payload = PAGE_FIXTURE.replacen("src/lib.rs", "../outside.rs", 1);
    fs::write(
        fixture.root.join("llm-wiki-out/workbench-pages.json"),
        unsafe_payload,
    )
    .expect("unsafe payload");
    assert!(renderer
        .render(&request)
        .expect_err("unsafe citation must fail")
        .to_string()
        .contains("confined"));

    let unexplained_stale = PAGE_FIXTURE.replacen(
        "\"freshnessNote\": \"architecture source changed after the provider snapshot\",",
        "",
        1,
    );
    fs::write(
        fixture.root.join("llm-wiki-out/workbench-pages.json"),
        unexplained_stale,
    )
    .expect("unexplained stale payload");
    assert!(renderer
        .render(&request)
        .expect_err("unexplained stale state must fail")
        .to_string()
        .contains("visible explanation"));

    let duplicate = PAGE_FIXTURE.replacen("\"id\": \"architecture\"", "\"id\": \"overview\"", 1);
    fs::write(
        fixture.root.join("llm-wiki-out/workbench-pages.json"),
        duplicate,
    )
    .expect("duplicate section payload");
    assert!(renderer
        .render(&request)
        .expect_err("duplicate sections must fail")
        .to_string()
        .contains("duplicate"));
}
// HANDWRITE-END
