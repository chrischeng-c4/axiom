//! External structure contract for the shared collector runtime.

#[test]
fn sift_collector_is_domain_hooks_over_service_collector() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("read Sift manifest");
    let runtime = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/collector/runtime.rs"
    ))
    .expect("read Sift collector adapter");
    let checkpoint = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/collector/checkpoint.rs"
    ))
    .expect("read Sift checkpoint adapter");
    let client = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/collector/client.rs"
    ))
    .expect("read Sift collector sink");

    assert!(manifest.contains("service-collector ="));
    assert!(runtime.contains("service_collector::run_collector"));
    assert!(runtime.contains("impl service_collector::RecordDecoder"));
    assert!(client.contains("impl service_collector::BatchSink"));
    assert!(checkpoint.contains("service_collector::save_json_checkpoint"));
    assert!(!runtime.contains("loop {"));
    assert!(!client.contains("tokio::time::sleep"));
    assert!(!checkpoint.contains("OpenOptions"));
}
