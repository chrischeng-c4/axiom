// HANDWRITE-BEGIN gap="missing-generator:unit-test:89dfd718" tracker="1887" reason="Lock Sift to the canonical shared-library package, crate, and path identities."
#[test]
fn manifest_uses_canonical_shared_library_names_without_aliases() {
    let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", manifest_path.display()));

    for expected in [
        r#"service-k8s = { path = "../../libs/service-k8s" }"#,
        r#"storage-durable = { path = "../../libs/storage-durable" }"#,
        r#"metrics-prometheus = { path = "../../libs/metrics-prometheus" }"#,
        r#"raft-runtime = { path = "../../libs/raft-runtime" }"#,
        r#"index-text = { path = "../../libs/index-text" }"#,
        r#"metrics-remote-write = { path = "../../libs/metrics-remote-write" }"#,
        r#"service-collector = { path = "../../libs/service-collector" }"#,
        r#"service-mcp = { path = "../../libs/service-mcp" }"#,
        r#"service-projection = { path = "../../libs/service-projection" }"#,
        r#"storage-object = { path = "../../libs/storage-object" }"#,
        r#"storage-segment = { path = "../../libs/storage-segment" }"#,
        r#"transport-otlp = { path = "../../libs/transport-otlp" }"#,
    ] {
        assert!(
            manifest.contains(expected),
            "missing dependency alias: {expected}"
        );
    }

    for retired_alias in [
        "axiom-operator =",
        "service-durability =",
        "service-metrics =",
        "raft-host =",
    ] {
        assert!(
            !manifest.contains(retired_alias),
            "retired dependency alias remains: {retired_alias}"
        );
    }

    for retired_path in [
        "../../libs/operator",
        "../../libs/service-durability",
        "../../libs/service-metrics",
        "../../libs/raft-host",
    ] {
        assert!(
            !manifest.contains(retired_path),
            "retired dependency path remains: {retired_path}"
        );
    }
}
// HANDWRITE-END
