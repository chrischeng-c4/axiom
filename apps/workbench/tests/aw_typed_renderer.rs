// HANDWRITE-BEGIN gap="missing-generator:unit-test:31774056" tracker="pending-tracker" reason="Prove all four typed artifact kinds, relationships, commands, Mermaid, byte identity, missing-configuration fallback, and mutation isolation."
use std::{collections::BTreeMap, fs, path::PathBuf};

use tempfile::TempDir;
use workbench::context::{ContextDocumentKind, ContextRequest, RendererRegistry};

const ARTIFACTS: [&str; 4] = [
    "tech-design.md",
    "external-contract.md",
    "capabilities.md",
    "work-item.md",
];

struct Fixture {
    _temporary_directory: TempDir,
    root: PathBuf,
}

impl Fixture {
    fn configured() -> Self {
        Self::copy(true)
    }

    fn without_configuration() -> Self {
        Self::copy(false)
    }

    fn copy(include_configuration: bool) -> Self {
        let temporary_directory = TempDir::new().expect("temporary fixture");
        let root = temporary_directory.path().to_path_buf();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/aw-context");
        for artifact in ARTIFACTS {
            fs::copy(source.join(artifact), root.join(artifact)).expect("copy AW artifact fixture");
        }
        if include_configuration {
            fs::copy(source.join("aw.toml"), root.join("aw.toml"))
                .expect("copy AW activation fixture");
        }
        Self {
            _temporary_directory: temporary_directory,
            root,
        }
    }

    fn snapshot(&self) -> BTreeMap<&'static str, Vec<u8>> {
        ARTIFACTS
            .into_iter()
            .map(|artifact| {
                (
                    artifact,
                    fs::read(self.root.join(artifact)).expect("read fixture bytes"),
                )
            })
            .collect()
    }
}

#[test]
fn renders_td_ec_capability_and_wi_fixtures() {
    let fixture = Fixture::configured();
    let registry = RendererRegistry::generic_with_optional_aw();
    let expectations = [
        ("tech-design.md", "Tech design"),
        ("external-contract.md", "External contract"),
        ("capabilities.md", "Capability contract"),
        ("work-item.md", "Work item"),
    ];

    for (artifact, kind_label) in expectations {
        let document = registry.render(
            &ContextRequest::file(&fixture.root, artifact).expect("confined artifact request"),
        );
        assert_eq!(document.renderer_id, "aw-typed", "artifact {artifact}");
        assert_eq!(document.kind, ContextDocumentKind::AwTyped);
        assert!(document.title.contains(kind_label));
        assert!(document.body_html.contains("Sections"));
        assert!(!document.navigation.is_empty());
        assert!(document
            .navigation
            .iter()
            .all(|navigation| navigation.path == PathBuf::from(artifact)));
        assert_eq!(document.provenance.sources.len(), 1);
    }
}

#[test]
fn renders_commands_assertions_mermaid_and_relationships() {
    let fixture = Fixture::configured();
    let registry = RendererRegistry::generic_with_optional_aw();

    let td = registry.render(
        &ContextRequest::file(&fixture.root, "tech-design.md").expect("TD request"),
    );
    assert!(td.body_html.contains("Frontmatter"));
    assert!(td.body_html.contains("fill_sections"));
    assert!(td.body_html.contains("data-language=\"mermaid\""));
    assert!(td.body_html.contains("aw td gen 2196"));
    assert!(td.body_html.contains("external-contract.md"));
    assert!(td.body_html.contains("#2196"));

    let ec = registry.render(
        &ContextRequest::file(&fixture.root, "external-contract.md").expect("EC request"),
    );
    assert!(ec.body_html.contains("response_ok"));
    assert!(ec.body_html.contains("aw ec verify"));
    assert!(ec.body_html.contains("tech-design.md"));

    let capability = registry.render(
        &ContextRequest::file(&fixture.root, "capabilities.md").expect("capability request"),
    );
    assert!(capability.body_html.contains("#2196"));

    let work_item = registry.render(
        &ContextRequest::file(&fixture.root, "work-item.md").expect("WI request"),
    );
    assert!(work_item.body_html.contains("tech-design.md"));
}

#[test]
fn open_navigate_refresh_and_close_preserve_source_bytes() {
    let fixture = Fixture::configured();
    let before = fixture.snapshot();
    let registry = RendererRegistry::generic_with_optional_aw();

    for artifact in ARTIFACTS {
        let request = ContextRequest::file(&fixture.root, artifact).expect("artifact request");
        let opened = registry.render(&request);
        for navigation in &opened.navigation {
            assert!(fixture.root.join(&navigation.path).is_file());
        }
        let refreshed = registry.render(&request);
        assert_eq!(opened, refreshed);
    }
    drop(registry);

    assert_eq!(before, fixture.snapshot());
}

#[test]
fn missing_aw_configuration_uses_generic_markdown() {
    let fixture = Fixture::without_configuration();
    assert!(!fixture.root.join("aw.toml").exists());
    let before = fs::read(fixture.root.join("tech-design.md")).expect("before bytes");
    let document = RendererRegistry::generic_with_optional_aw().render(
        &ContextRequest::file(&fixture.root, "tech-design.md").expect("TD request"),
    );
    assert_eq!(document.renderer_id, "markdown");
    assert_eq!(document.kind, ContextDocumentKind::Markdown);
    assert_eq!(
        before,
        fs::read(fixture.root.join("tech-design.md")).expect("after bytes")
    );
}

#[test]
fn typed_renderer_exposes_no_mutating_operation() {
    let implementation = [
        include_str!("../src/context/aw.rs"),
        include_str!("../src/context/mod.rs"),
    ]
    .join("\n");
    for forbidden in [
        "Command::new",
        "OpenOptions",
        "fs::write",
        "fs::remove",
        "fs::rename",
        "native_agent_pty",
        "cwd_context",
        "folder_shell",
        "aw goal",
        "aw td create",
        "aw ec review",
        "gh issue",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "typed renderer unexpectedly exposes mutation surface {forbidden}"
        );
    }
    assert!(implementation.contains("fs::read"));
}
// HANDWRITE-END
