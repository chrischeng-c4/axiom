// HANDWRITE-BEGIN gap="missing-generator:unit-test:6e590b3f" tracker="pending-tracker" reason="Prove every node/edge has canonical navigation or a visible derived label, plus absence, malformed data, injected source failure, sentinel isolation, and no mutation surface."
use std::{fs, path::Path, process::Command};

use tempfile::TempDir;
use workbench::{
    context::{
        graph::{GraphContextRenderer, GraphPayloadSource, MAX_GRAPH_PAYLOAD_BYTES},
        ContextDocument, ContextDocumentKind, ContextProvenance, ContextRenderer, ContextRequest,
        RendererError, RendererRegistry, RendererSupport,
    },
    native_agent_pty::{PtyCommand, PtyRuntime, PtySize},
};

const GRAPH_FIXTURE: &str =
    include_str!("fixtures/graph-context/graphify-out/workbench-graph.json");
const SERVICE_FIXTURE: &str = include_str!("fixtures/graph-context/src/service.rs");
const HANDLER_FIXTURE: &str = include_str!("fixtures/graph-context/src/handler.rs");

struct GraphFixture {
    _temporary_directory: TempDir,
    root: std::path::PathBuf,
}

impl GraphFixture {
    fn new() -> Self {
        let temporary_directory = TempDir::new().expect("temporary graph fixture");
        let root = temporary_directory.path().to_path_buf();
        fs::create_dir_all(root.join("graphify-out")).expect("graph output directory");
        fs::create_dir_all(root.join("src")).expect("source directory");
        fs::write(
            root.join("graphify-out/workbench-graph.json"),
            GRAPH_FIXTURE,
        )
        .expect("graph payload");
        fs::write(root.join("src/service.rs"), SERVICE_FIXTURE).expect("service source");
        fs::write(root.join("src/handler.rs"), HANDLER_FIXTURE).expect("handler source");
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
            workbench::context::ContextTarget::File(path) if path == Path::new("sentinel.aw") => {
                RendererSupport::Supported
            }
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

impl GraphPayloadSource for FailingSource {
    fn read(&self, _: &Path, _: usize) -> Result<Vec<u8>, RendererError> {
        Err(RendererError::new("injected provider read failure"))
    }
}

#[test]
fn renders_source_or_visible_inference_for_every_node_and_edge() {
    let fixture = GraphFixture::new();
    let document = RendererRegistry::production()
        .render(&ContextRequest::workspace(&fixture.root).expect("workspace context"));
    assert_eq!(document.kind, ContextDocumentKind::Graph);
    assert_eq!(document.renderer_id, "graph-context");

    let payload: serde_json::Value = serde_json::from_str(GRAPH_FIXTURE).expect("fixture JSON");
    for (record_kind, records) in [
        ("node", payload["nodes"].as_array().unwrap()),
        ("edge", payload["edges"].as_array().unwrap()),
    ] {
        for record in records {
            let id = record["id"].as_str().unwrap();
            let classification = record["classification"].as_str().unwrap();
            assert!(
                document
                    .body_html
                    .contains(&format!("data-record-id=\"{id}\"")),
                "missing rendered {record_kind} {id}"
            );
            let has_source_link = document
                .navigation
                .iter()
                .any(|link| link.label.starts_with(&format!("{record_kind} {id}:")));
            let has_visible_derived_label = classification != "extracted"
                && document.body_html.contains(&format!(
                    ">{}</p>",
                    if classification == "inferred" {
                        "Inferred"
                    } else {
                        "Ambiguous"
                    }
                ));
            assert!(
                has_source_link || has_visible_derived_label,
                "{record_kind} {id} has neither canonical navigation nor a derived label"
            );
            for source in record["sources"].as_array().unwrap() {
                let path = source["relative_path"].as_str().unwrap();
                assert!(
                    document.body_html.contains(path),
                    "missing source input {path}"
                );
            }
        }
    }
    assert!(document.body_html.contains("Inferred · derived"));
    assert!(document.body_html.contains("Ambiguous · derived"));
    assert!(document.body_html.contains("src/not-yet-present.rs"));
}

#[test]
fn provider_absence_leaves_generic_and_aw_sentinel_renderers_usable() {
    let temporary_directory = TempDir::new().expect("absence fixture");
    let root = temporary_directory.path();
    fs::write(root.join("README.md"), "# Generic Markdown\n").expect("Markdown fixture");
    fs::write(root.join("sentinel.aw"), "sentinel\n").expect("sentinel fixture");
    initialize_git(root);
    assert!(!root.join("graphify-out/workbench-graph.json").exists());

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
    isolated.register(GraphContextRenderer::new());
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

    let test_source = include_str!("graph_context_adapter.rs");
    let concrete_renderer = ["Aw", "Typed", "Renderer"].concat();
    assert!(
        !test_source.contains(&concrete_renderer),
        "test imported a concrete AW renderer"
    );
}

#[test]
fn malformed_and_failing_provider_are_isolated() {
    let fixture = GraphFixture::new();
    fs::write(
        fixture.root.join("graphify-out/workbench-graph.json"),
        "{ malformed",
    )
    .expect("malformed payload");
    fs::write(fixture.root.join("dirty.txt"), "force Git context\n").expect("dirty source");
    let request = ContextRequest::workspace(&fixture.root).expect("workspace request");
    let malformed = RendererRegistry::production().render(&request);
    assert_eq!(malformed.kind, ContextDocumentKind::Git);
    assert!(malformed
        .warnings
        .iter()
        .any(|warning| warning.contains("graph-context") && warning.contains("malformed")));

    fs::write(
        fixture.root.join("graphify-out/workbench-graph.json"),
        GRAPH_FIXTURE,
    )
    .expect("restore valid payload");
    let mut isolated = RendererRegistry::generic();
    isolated.register(GraphContextRenderer::with_source(FailingSource));
    let failure = isolated.render(&request);
    assert_eq!(failure.kind, ContextDocumentKind::Git);
    assert!(failure.warnings.iter().any(|warning| {
        warning.contains("graph-context") && warning.contains("injected provider read failure")
    }));
}

#[test]
fn adapter_contract_is_reference_only_and_read_only() {
    let fixture = GraphFixture::new();
    let watched = [
        fixture.root.join("graphify-out/workbench-graph.json"),
        fixture.root.join("src/service.rs"),
        fixture.root.join("src/handler.rs"),
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
    assert_eq!(
        before, after,
        "graph rendering mutated provider or source files"
    );

    let implementation = include_str!("../src/context/graph.rs");
    for forbidden in [
        "graphify::",
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
            "graph adapter unexpectedly exposes {forbidden}"
        );
    }
    assert!(implementation.contains("workbench.graph-context.v1"));
    assert!(implementation.contains("File::open"));
}

#[test]
fn payload_limits_and_source_confinement_fail_closed() {
    let fixture = GraphFixture::new();
    let request = ContextRequest::workspace(&fixture.root).expect("workspace request");
    let renderer = GraphContextRenderer::new();

    fs::write(
        fixture.root.join("graphify-out/workbench-graph.json"),
        vec![b' '; MAX_GRAPH_PAYLOAD_BYTES + 1],
    )
    .expect("oversized payload");
    let oversized = renderer
        .render(&request)
        .expect_err("oversized graph must fail");
    assert!(oversized.to_string().contains("exceeds"));

    let unsafe_payload = GRAPH_FIXTURE.replacen("src/service.rs", "../outside.rs", 1);
    fs::write(
        fixture.root.join("graphify-out/workbench-graph.json"),
        unsafe_payload,
    )
    .expect("unsafe payload");
    let unsafe_error = renderer
        .render(&request)
        .expect_err("unsafe path must fail");
    assert!(unsafe_error.to_string().contains("confined"));

    let unknown_endpoint =
        GRAPH_FIXTURE.replacen("\"to\": \"handler\"", "\"to\": \"missing-node\"", 1);
    fs::write(
        fixture.root.join("graphify-out/workbench-graph.json"),
        unknown_endpoint,
    )
    .expect("unknown endpoint payload");
    let endpoint_error = renderer
        .render(&request)
        .expect_err("unknown endpoint must fail");
    assert!(endpoint_error.to_string().contains("unknown endpoint"));
}
// HANDWRITE-END
