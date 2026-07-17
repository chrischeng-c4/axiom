// HANDWRITE-BEGIN gap="missing-generator:unit-test:89dfd718" tracker="1887" reason="Lock the four current package names and paths while proving the legacy Rust dependency aliases remain stable."
#[test]
fn manifest_uses_current_shared_library_packages() {
    let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", manifest_path.display()));

    for expected in [
        r#"axiom-operator = { package = "service-k8s", path = "../../libs/service-k8s" }"#,
        r#"service-durability = { package = "storage-durable", path = "../../libs/storage-durable" }"#,
        r#"service-metrics = { package = "metrics-prometheus", path = "../../libs/metrics-prometheus" }"#,
        r#"raft-host = { package = "raft-runtime", path = "../../libs/raft-runtime" }"#,
    ] {
        assert!(
            manifest.contains(expected),
            "missing dependency alias: {expected}"
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
