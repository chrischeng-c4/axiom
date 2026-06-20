// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
//! Build script: compile the vendored google.pubsub.v1 proto for the built-in
//! Pub/Sub emulator. No-op for a lean build (no `emulator` feature) or before
//! the proto is vendored, so the build never depends on a system protoc.

fn main() {
    // Cargo exposes enabled features as CARGO_FEATURE_<NAME>.
    if std::env::var_os("CARGO_FEATURE_EMULATOR").is_none() {
        return;
    }
    let proto = "proto/google/pubsub/v1/pubsub.proto";
    if !std::path::Path::new(proto).exists() {
        return;
    }
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc binary");
    std::env::set_var("PROTOC", protoc);
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&[proto], &["proto"])
        .expect("compile google.pubsub.v1 proto");
    println!("cargo:rerun-if-changed={proto}");
}
// CODEGEN-END
