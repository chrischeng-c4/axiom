//! External structure contract for the shared Remote Write transport.

#[test]
fn sift_keeps_only_remote_write_domain_conversion() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("read Sift manifest");
    let adapter =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/prometheus.rs"))
            .expect("read Sift Prometheus adapter");
    let runtime = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("read Sift runtime");

    assert!(manifest.contains("metrics-remote-write ="));
    assert!(adapter.contains("pub use metrics_remote_write::{proto as remote"));
    assert!(adapter.contains("impl metrics_remote_write::RemoteWriteConsumer"));
    assert!(!adapter.contains("pub mod remote"));
    assert!(runtime.contains("metrics_remote_write::validate_headers"));
    assert!(runtime.contains("metrics_remote_write::decode_snappy"));
    assert!(!runtime.contains("snap::raw::Decoder"));
}
