// HANDWRITE-BEGIN gap="missing-generator:unit-test:cb9e709e" tracker="pending-tracker" reason="Prove extracted round-trip, inferred labels and inputs, missing/invalid degradation, path confinement, and mutation-surface isolation."
use std::{fs, path::Path};

use tempfile::TempDir;
use workbench::context::provenance::{
    ContextProvenanceItem, ProvenanceAuthority, ProvenanceClassification, ProviderIdentity,
    SourceLocation, SourcePosition, SourceSpan, SourceStatus,
};

fn source_fixture() -> TempDir {
    let fixture = TempDir::new().expect("temporary source root");
    fs::create_dir(fixture.path().join("src")).expect("source directory");
    fs::write(
        fixture.path().join("src/lib.rs"),
        "fn one() {}\nfn two() {}\n",
    )
    .expect("source fixture");
    fixture
}

fn provider(id: &str, label: &str) -> ProviderIdentity {
    ProviderIdentity::new(id, label)
}

#[test]
fn extracted_item_round_trips_to_canonical_file_and_span() {
    let fixture = source_fixture();
    let span = SourceSpan::new(SourcePosition::new(2, 1), SourcePosition::new(2, 9));
    let item = ContextProvenanceItem::extracted(
        provider("builtin-markdown", "Workbench Markdown"),
        SourceLocation::with_span("src/lib.rs", span),
    );
    let json = serde_json::to_string(&item).expect("serialize provenance item");
    let decoded: ContextProvenanceItem =
        serde_json::from_str(&json).expect("deserialize provenance item");
    assert_eq!(decoded, item);

    let view = decoded.resolve(fixture.path());
    assert_eq!(view.provider, item.provider);
    assert_eq!(view.classification, ProvenanceClassification::Extracted);
    assert_eq!(view.authority, ProvenanceAuthority::Canonical);
    assert!(view.badge.contains("Extracted"));
    assert!(view.badge.contains("canonical source"));
    assert!(view.badge.contains("Workbench Markdown"));
    assert_eq!(view.sources.len(), 1);
    assert_eq!(view.sources[0].status, SourceStatus::Canonical);
    let navigation = view.sources[0]
        .navigation
        .as_ref()
        .expect("canonical navigation");
    assert_eq!(navigation.relative_path, Path::new("src/lib.rs"));
    assert_eq!(navigation.span, Some(span));
}

#[test]
fn inferred_items_disclose_provider_label_and_all_inputs() {
    let fixture = source_fixture();
    let item = ContextProvenanceItem::inferred(
        provider("graph-adapter", "Local Graph Adapter"),
        vec![
            SourceLocation::file("src/lib.rs"),
            SourceLocation::file("src/missing.rs"),
        ],
    );
    let view = item.resolve(fixture.path());
    assert_eq!(view.classification, ProvenanceClassification::Inferred);
    assert_eq!(view.authority, ProvenanceAuthority::Derived);
    assert!(view.badge.contains("Inferred"));
    assert!(view.badge.contains("derived from 2 source input(s)"));
    assert!(view.badge.contains("Local Graph Adapter"));
    assert!(view.badge.contains("1 unavailable"));
    assert_eq!(view.sources.len(), 2);
    assert_eq!(view.sources[0].status, SourceStatus::Canonical);
    assert_eq!(view.sources[1].status, SourceStatus::Missing);
    assert!(view.sources[1].navigation.is_none());

    let ambiguous = ContextProvenanceItem::ambiguous(
        provider("unknown", "Uncertain Provider"),
        vec![SourceLocation::file("src/lib.rs")],
    )
    .resolve(fixture.path());
    assert_eq!(ambiguous.authority, ProvenanceAuthority::Derived);
    assert!(ambiguous.badge.contains("Ambiguous"));
}

#[test]
fn invalid_and_missing_sources_never_fabricate_links() {
    let fixture = source_fixture();
    let cases = [
        ContextProvenanceItem::extracted(
            provider("fixture", "Fixture"),
            SourceLocation::file("src/missing.rs"),
        ),
        ContextProvenanceItem::extracted(
            provider("fixture", "Fixture"),
            SourceLocation::with_span(
                "src/lib.rs",
                SourceSpan::new(SourcePosition::new(0, 1), SourcePosition::new(1, 1)),
            ),
        ),
        ContextProvenanceItem::extracted(
            provider("fixture", "Fixture"),
            SourceLocation::file("../outside.rs"),
        ),
        ContextProvenanceItem::extracted(
            provider("fixture", "Fixture"),
            SourceLocation::file("/tmp/outside.rs"),
        ),
        ContextProvenanceItem::extracted(
            provider("fixture", "Fixture"),
            SourceLocation::file("src"),
        ),
    ];

    for item in cases {
        let view = item.resolve(fixture.path());
        assert_eq!(view.authority, ProvenanceAuthority::Unavailable);
        assert!(view.badge.contains("non-authoritative"));
        assert!(view.sources[0].navigation.is_none());
        assert!(matches!(
            view.sources[0].status,
            SourceStatus::Missing | SourceStatus::Invalid { .. }
        ));
    }

    #[cfg(unix)]
    {
        let outside = TempDir::new().expect("outside directory");
        fs::write(outside.path().join("outside.rs"), "outside\n").expect("outside source");
        std::os::unix::fs::symlink(
            outside.path().join("outside.rs"),
            fixture.path().join("src/escape.rs"),
        )
        .expect("escape symlink");
        let escaped = ContextProvenanceItem::extracted(
            provider("fixture", "Fixture"),
            SourceLocation::file("src/escape.rs"),
        )
        .resolve(fixture.path());
        assert_eq!(escaped.authority, ProvenanceAuthority::Unavailable);
        assert!(escaped.sources[0].navigation.is_none());
        assert!(matches!(
            escaped.sources[0].status,
            SourceStatus::Invalid { .. }
        ));
    }
}

#[test]
fn provenance_api_is_provider_neutral_and_read_only() {
    let fixture = source_fixture();
    let source_path = fixture.path().join("src/lib.rs");
    let before = fs::read(&source_path).expect("before bytes");
    let _ = ContextProvenanceItem::extracted(
        provider("arbitrary-provider", "Arbitrary Provider"),
        SourceLocation::file("src/lib.rs"),
    )
    .resolve(fixture.path());
    assert_eq!(before, fs::read(&source_path).expect("after bytes"));

    let implementation = include_str!("../src/context/provenance.rs");
    for forbidden in [
        "Command::new",
        "OpenOptions",
        "fs::write",
        "fs::remove",
        "fs::rename",
        "graphify::",
        "aw::",
        "native_agent_pty",
        "cwd_context",
        "folder_shell",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "provenance core unexpectedly exposes {forbidden}"
        );
    }
    assert!(implementation.contains("fs::canonicalize"));
}
// HANDWRITE-END
