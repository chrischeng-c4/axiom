// HANDWRITE-BEGIN gap="missing-generator:unit-test:894ee1ab" tracker="#777" reason="Assert the keep::spec surface: openapi_json is valid OpenAPI 3.x with keep data-plane paths, openapi_yaml parses, json_schema exposes component schemas, request_shapes carry request bodies, value_catalog matches the KvValue variants, and cclab_openapi_codegen::generate composes on keep's OpenAPI for ts/py/rust."
//! `keep spec` surface: the offline, machine-readable self-description an agent
//! reads to wire keep into a pipeline. Each emitter must produce valid output
//! with the expected shape (no server, no network), and `spec gen` must compose
//! the shared openapi-codegen on keep's own OpenAPI for every language.
//!
//! @spec .aw/tech-design/projects/keep/interfaces/cli/deploy-cli-keep-spec-spec-gen-dockerfile-render.md

use keep::spec::{json_schema_json, openapi_json, openapi_yaml, request_shapes, value_catalog};
use serde_json::Value;
use serde_yaml::Value as YamlValue;

#[test]
fn openapi_is_valid_json_with_kv_data_plane() {
    let v: Value = serde_json::from_str(&openapi_json()).expect("openapi is valid JSON");
    assert_eq!(
        v["openapi"].as_str().map(|s| s.starts_with("3.")),
        Some(true),
        "OpenAPI 3.x document"
    );
    let paths = v["paths"].as_object().expect("has paths");
    assert!(
        paths.keys().any(|p| p.contains("/kv")),
        "exposes the KV data plane: {:?}",
        paths.keys().collect::<Vec<_>>()
    );
}

#[test]
fn openapi_yaml_parses_as_openapi_3() {
    let v: YamlValue = serde_yaml::from_str(&openapi_yaml()).expect("openapi is valid YAML");
    let root = v.as_mapping().expect("YAML root is a mapping");
    let openapi = root
        .get(YamlValue::String("openapi".into()))
        .and_then(YamlValue::as_str);
    assert_eq!(openapi.map(|s| s.starts_with("3.")), Some(true));
}

#[test]
fn json_schema_emits_component_schemas() {
    let v: Value = serde_json::from_str(&json_schema_json()).expect("json-schema is valid JSON");
    assert!(
        v["components"]["schemas"].is_object(),
        "components.schemas present (the request/response data types): {v}"
    );
}

/// The emitted OpenAPI advertises keep's real serving port (7117) and carries a
/// non-empty component-schema map — the offline twin of the served document.
#[test]
fn openapi_advertises_port_7117_with_component_schemas() {
    let v: Value = serde_json::from_str(&openapi_json()).expect("openapi is valid JSON");

    assert!(
        v["components"]["schemas"]
            .as_object()
            .is_some_and(|s| !s.is_empty()),
        "components.schemas is a non-empty object"
    );

    let servers: Vec<String> = v["servers"]
        .as_array()
        .expect("servers array")
        .iter()
        .filter_map(|s| s["url"].as_str().map(str::to_string))
        .collect();
    assert!(!servers.is_empty(), "servers block present");
    assert!(
        servers.iter().any(|u| u.contains(":7117")),
        "servers advertise the real port :7117, got {servers:?}"
    );
}

#[test]
fn request_shapes_cover_core_ops_and_carry_requests() {
    let v = request_shapes();
    let shapes = v["shapes"].as_array().expect("shapes array");
    let names: Vec<&str> = shapes.iter().map(|s| s["name"].as_str().unwrap()).collect();
    for required in [
        "set",
        "get",
        "incr",
        "cas",
        "mset",
        "scan",
        "lock",
        "hset",
        "sadd",
        "zadd",
        "claim_check_input",
    ] {
        assert!(
            names.contains(&required),
            "cookbook missing shape `{required}`: {names:?}"
        );
    }
    for s in shapes {
        assert!(
            s["description"].is_string(),
            "shape {} carries a description",
            s["name"]
        );
        assert!(
            s["request"]["method"].is_string(),
            "shape {} carries an HTTP method",
            s["name"]
        );
        assert!(
            s["request"]["path"].is_string(),
            "shape {} carries a path",
            s["name"]
        );
    }
}

#[test]
fn value_catalog_matches_the_kvvalue_enum() {
    let v = value_catalog();
    let types: Vec<&str> = v["value_types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["type"].as_str().unwrap())
        .collect();
    // Every KvValue variant (wire form) is documented.
    for t in [
        "int",
        "float",
        "decimal",
        "string",
        "bytes",
        "list",
        "map",
        "set",
        "sorted_set",
        "null",
    ] {
        assert!(types.contains(&t), "value catalog missing `{t}`: {types:?}");
    }
}

/// Recursively collect every `$ref` value anywhere in the document.
fn collect_refs(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(map) => {
            for (k, val) in map {
                if k == "$ref" {
                    if let Some(s) = val.as_str() {
                        out.push(s.to_string());
                    }
                }
                collect_refs(val, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_refs(item, out);
            }
        }
        _ => {}
    }
}

/// R1+R2: the document is self-complete — every `$ref` in the emitted OpenAPI
/// resolves to a key registered under `components.schemas`. Strict: walks ALL
/// refs recursively, no allowlist (no `crate.*`-dotted dangling refs).
#[test]
fn every_ref_resolves_to_a_registered_component_schema() {
    let v: Value = serde_json::from_str(&openapi_json()).expect("openapi is valid JSON");
    let schemas = v["components"]["schemas"]
        .as_object()
        .expect("components.schemas object");

    let mut refs = Vec::new();
    collect_refs(&v, &mut refs);
    assert!(!refs.is_empty(), "document carries schema refs");

    let dangling: Vec<&String> = refs
        .iter()
        .filter(|r| {
            let Some(name) = r.strip_prefix("#/components/schemas/") else {
                // Any non-schema ref shape is itself a self-completeness defect.
                return true;
            };
            !schemas.contains_key(name)
        })
        .collect();
    assert!(
        dangling.is_empty(),
        "dangling $refs (must all resolve in components.schemas): {dangling:?}"
    );
}

/// R2: `spec gen` composes the SHARED codegen on keep's own OpenAPI — one
/// codegen path, no external tool — and every language emits client files.
#[test]
fn spec_gen_composes_openapi_codegen_for_every_language() {
    use cclab_openapi_codegen::{generate, GenOptions, HttpClient, Lang};
    let doc = openapi_json();
    for lang in [Lang::Ts, Lang::Py, Lang::Rust] {
        let opts = GenOptions {
            lang,
            spec_path: std::path::PathBuf::new(),
            out_dir: std::path::PathBuf::new(),
            client_name: "createClient".to_string(),
            http_client: HttpClient::Fetch,
            emit_types: true,
            emit_client: true,
            emit_hooks: matches!(lang, Lang::Ts),
        };
        let out = generate(&doc, &opts).expect("codegen composes on keep's OpenAPI");
        assert!(!out.files.is_empty(), "{lang:?} produced client files");
    }
}
// HANDWRITE-END
