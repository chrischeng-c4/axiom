#![cfg(test)]

// Schema gate for #1537 — cloud SDK ecosystem umbrella.
// Locks the umbrella acceptance shape (leaf sets +
// hello-worlds + carve-outs); does NOT build mamba.

use std::fs;
use std::path::PathBuf;
use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/ecosystem/cloud_sdk_umbrella_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

fn get<'a>(v: &'a Value, key: &str) -> &'a Value {
    v.get(key).unwrap_or_else(|| panic!("missing key: {key}"))
}
fn b(v: &Value, key: &str) -> bool {
    get(v, key).as_bool().unwrap_or_else(|| panic!("{key} not bool"))
}
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    get(v, key).as_str().unwrap_or_else(|| panic!("{key} not str"))
}
fn i(v: &Value, key: &str) -> i64 {
    get(v, key).as_integer().unwrap_or_else(|| panic!("{key} not int"))
}
fn a<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    get(v, key).as_array().unwrap_or_else(|| panic!("{key} not array"))
}

fn leaf_libs(arr: &[Value]) -> Vec<&str> {
    arr.iter()
        .filter_map(|v| v.get("lib").and_then(|x| x.as_str()))
        .collect()
}

fn leaf_issues(arr: &[Value]) -> Vec<i64> {
    arr.iter()
        .filter_map(|v| v.get("issue").and_then(|x| x.as_integer()))
        .collect()
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(i(&m, "version"), 1);
    assert_eq!(s(&m, "fixture"), "cloud_sdk_umbrella_gate");
    assert_eq!(i(&m, "issue"), 1537);
    assert_eq!(i(&m, "parent_issue"), 1265);
    assert_eq!(s(&m, "profile"), "conformance");
    assert_eq!(s(&m, "family"), "cloud_sdk_umbrella_gate");
    assert_eq!(s(&m, "network"), "offline");
}

#[test]
fn isolation_pins_no_global_state() {
    let m = manifest();
    let iso = get(&m, "isolation");
    assert!(b(iso, "forbid_writes_outside_project"));
    assert!(b(iso, "forbid_user_home_reads"));
    assert!(b(iso, "forbid_global_cache_reads"));
    assert!(b(iso, "forbid_global_cache_writes"));
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let m = manifest();
    let py = get(&m, "python_target");
    assert_eq!(i(py, "python_major"), 3);
    assert_eq!(i(py, "python_minor"), 12);
    assert!(b(py, "must_be_python_3_12"));
}

#[test]
fn surface_pins_the_five_required_axes() {
    let m = manifest();
    let sf = get(&m, "surface");
    assert!(b(sf, "must_cover_aws_leaf_set_and_hello_world"));
    assert!(b(sf, "must_cover_azure_leaf_set_and_hello_world"));
    assert!(b(sf, "must_cover_gcp_leaf_set_via_grpclib_path_and_hello_world"));
    assert!(b(sf, "must_cover_grpcio_deferred_and_out_of_scope_carveouts"));
    assert!(b(sf, "must_cover_stdlib_dependency_edges_recorded"));
    assert!(b(sf, "must_be_offline_or_loopback_only"));
    assert!(b(sf, "must_be_deterministic"));
}

#[test]
fn r1_aws_leaf_set_and_hello_world() {
    let m = manifest();
    let r = get(&m, "r1_aws_leaf_set_and_hello_world_contract");
    assert_eq!(s(r, "requirement_id"), "R1");
    assert!(b(r, "must_enumerate_aws_leaf_set"));
    assert!(b(r, "must_pin_aws_hello_world_call"));
    let leafs = a(r, "aws_leafs");
    let libs = leaf_libs(leafs);
    for lib in &["boto3", "botocore", "s3transfer", "jmespath"] {
        assert!(libs.contains(lib), "missing aws leaf: {lib}");
    }
    let issues = leaf_issues(leafs);
    for issue in &[1500_i64, 1501, 1502, 1503] {
        assert!(issues.contains(issue), "missing aws issue: {issue}");
    }
    assert_eq!(s(r, "expected_aws_hello_world"), "boto3.client('s3').list_buckets()");
    assert_eq!(i(r, "aws_leaf_dropped_exit_code"), 500);
    assert_eq!(i(r, "aws_hello_modified_exit_code"), 501);
    assert!(b(r, "must_distinguish_aws_leaf_dropped_from_hello_modified"));
}

#[test]
fn r2_azure_leaf_set_and_hello_world() {
    let m = manifest();
    let r = get(&m, "r2_azure_leaf_set_and_hello_world_contract");
    assert_eq!(s(r, "requirement_id"), "R2");
    assert!(b(r, "must_enumerate_azure_leaf_set"));
    let leafs = a(r, "azure_leafs");
    let libs = leaf_libs(leafs);
    for lib in &["azure-core", "azure-identity", "azure-storage-blob", "azure-keyvault-secrets"] {
        assert!(libs.contains(lib), "missing azure leaf: {lib}");
    }
    let issues = leaf_issues(leafs);
    for issue in &[1504_i64, 1505, 1506, 1507] {
        assert!(issues.contains(issue), "missing azure issue: {issue}");
    }
    assert_eq!(
        s(r, "expected_azure_hello_world"),
        "BlobServiceClient.from_connection_string(cs).list_containers()"
    );
    assert_eq!(i(r, "azure_leaf_dropped_exit_code"), 502);
    assert_eq!(i(r, "azure_hello_modified_exit_code"), 503);
    assert!(b(r, "must_distinguish_azure_leaf_dropped_from_hello_modified"));
}

#[test]
fn r3_gcp_leaf_set_via_grpclib_path() {
    let m = manifest();
    let r = get(&m, "r3_gcp_leaf_set_via_grpclib_path_and_hello_world_contract");
    assert_eq!(s(r, "requirement_id"), "R3");
    assert!(b(r, "must_enumerate_gcp_leaf_set"));
    assert!(b(r, "must_pin_gcp_grpclib_path"));
    assert!(b(r, "must_pin_pure_python_protobuf_backend"));
    let leafs = a(r, "gcp_leafs");
    let libs = leaf_libs(leafs);
    for lib in &[
        "google-api-core",
        "google-cloud-storage",
        "google-cloud-pubsub",
        "googleapis-common-protos",
        "protobuf",
        "grpclib",
    ] {
        assert!(libs.contains(lib), "missing gcp leaf: {lib}");
    }
    let issues = leaf_issues(leafs);
    for issue in &[1509_i64, 1510, 1511, 1512, 1513, 1514] {
        assert!(issues.contains(issue), "missing gcp issue: {issue}");
    }
    let transports = a(r, "allowed_gcp_grpc_transport_values");
    let tnames: Vec<&str> = transports.iter().filter_map(|v| v.as_str()).collect();
    assert!(tnames.contains(&"grpclib_pure_python"));
    assert!(tnames.contains(&"grpcio_c_wrapper"));
    assert_eq!(s(r, "expected_gcp_grpc_transport"), "grpclib_pure_python");
    assert_eq!(
        s(r, "expected_gcp_hello_world"),
        "google.cloud.storage.Client().list_buckets()"
    );
    assert_eq!(i(r, "gcp_leaf_dropped_exit_code"), 504);
    assert_eq!(i(r, "gcp_switched_to_grpcio_exit_code"), 505);
    assert_eq!(i(r, "gcp_hello_modified_exit_code"), 506);
    assert!(b(
        r,
        "must_distinguish_gcp_leaf_dropped_from_grpcio_switch_from_hello_modified"
    ));
}

#[test]
fn r4_grpcio_deferred_and_carveouts() {
    let m = manifest();
    let r = get(&m, "r4_grpcio_deferred_and_out_of_scope_carveouts_contract");
    assert_eq!(s(r, "requirement_id"), "R4");
    assert!(b(r, "must_record_grpcio_deferred"));
    assert!(b(r, "must_carve_out_grpc_server_side"));
    assert!(b(r, "must_carve_out_cloud_regional_federation_oddities"));
    assert!(b(r, "must_carve_out_per_lib_perf_vs_cpython_at_umbrella_level"));
    let deferred = a(r, "deferred_libs");
    let dnames: Vec<&str> = deferred
        .iter()
        .filter_map(|v| v.get("lib").and_then(|x| x.as_str()))
        .collect();
    assert!(dnames.contains(&"grpcio"));
    let dissues: Vec<i64> = deferred
        .iter()
        .filter_map(|v| v.get("issue").and_then(|x| x.as_integer()))
        .collect();
    assert!(dissues.contains(&1515));
    let carved = a(r, "carved_out_axes");
    let cnames: Vec<&str> = carved.iter().filter_map(|v| v.as_str()).collect();
    for axis in &["grpc_server_side", "cloud_regional_federation_oddities", "per_lib_perf_vs_cpython"] {
        assert!(cnames.contains(axis), "missing carve-out: {axis}");
    }
    assert_eq!(i(r, "grpcio_unblocked_in_umbrella_exit_code"), 507);
    assert_eq!(i(r, "carve_out_violated_exit_code"), 508);
    assert!(b(r, "must_distinguish_grpcio_unblocked_from_carve_out_violation"));
}

#[test]
fn r5_stdlib_dependency_edges_recorded() {
    let m = manifest();
    let r = get(&m, "r5_stdlib_dependency_edges_recorded_contract");
    assert_eq!(s(r, "requirement_id"), "R5");
    assert!(b(r, "must_pin_stdlib_dependency_edges"));
    let edges = a(r, "stdlib_dependency_edges");
    let names: Vec<&str> = edges
        .iter()
        .filter_map(|v| v.get("stdlib").and_then(|x| x.as_str()))
        .collect();
    for lib in &["ssl", "socket", "asyncio", "urllib.parse", "hashlib", "hmac"] {
        assert!(names.contains(lib), "missing stdlib edge: {lib}");
    }
    let issues: Vec<i64> = edges
        .iter()
        .filter_map(|v| v.get("issue").and_then(|x| x.as_integer()))
        .collect();
    for issue in &[1414_i64, 1415, 1416, 1419, 1424, 1425] {
        assert!(issues.contains(issue), "missing stdlib issue: {issue}");
    }
    assert_eq!(i(r, "stdlib_edge_dropped_exit_code"), 509);
    assert_eq!(i(r, "stdlib_edge_unrelated_added_exit_code"), 510);
    assert!(b(r, "must_distinguish_edge_dropped_from_unrelated_edge_added"));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let m = manifest();
    let rc = get(&m, "runner_contract");
    let keys = a(rc, "keys");
    let names: Vec<&str> = keys.iter().filter_map(|v| v.as_str()).collect();
    for k in &[
        "outcome",
        "case",
        "requirement_id",
        "aws_leafs",
        "aws_hello_world",
        "azure_leafs",
        "azure_hello_world",
        "gcp_leafs",
        "gcp_grpc_transport",
        "gcp_hello_world",
        "deferred_libs",
        "carved_out_axes",
        "stdlib_dependency_edges",
        "failure_kind",
        "exit_code",
    ] {
        assert!(names.contains(k), "missing key: {k}");
    }
    let cases = a(rc, "case_values");
    assert_eq!(cases.len(), 5);
}

#[test]
fn pins_out_of_scope_per_issue() {
    let m = manifest();
    let oos = get(&m, "out_of_scope");
    assert!(b(oos, "implementation_of_any_leaf_lib"));
    assert!(b(oos, "grpc_server_side"));
    assert!(b(oos, "cloud_regional_or_federation_oddities"));
    assert!(b(oos, "per_lib_perf_vs_cpython"));
    assert!(b(oos, "c_extension_fast_paths"));
}
