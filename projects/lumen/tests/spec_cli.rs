// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! `lumen spec` surface: the offline, machine-readable self-description an
//! agent reads to wire lumen into a pipeline. Each emitter must produce valid
//! JSON with the expected top-level shape (no server, no I/O).

use lumen::spec::{
    field_catalog, json_schema_json, llm_integration_md, llm_outline_md, llm_quickstart_md,
    llm_recipes_md, llm_workflow_md, openapi_json, openapi_yaml, query_shapes,
};
use serde_json::Value;
use serde_yaml::Value as YamlValue;

#[test]
fn openapi_is_valid_json_with_search_path() {
    let v: Value = serde_json::from_str(&openapi_json()).expect("openapi is valid JSON");
    assert_eq!(
        v["openapi"].as_str().map(|s| s.starts_with("3.")),
        Some(true),
        "OpenAPI 3.x document"
    );
    let paths = v["paths"].as_object().expect("has paths");
    assert!(
        paths.keys().any(|p| p.contains("/search")),
        "exposes a search path: {:?}",
        paths.keys().collect::<Vec<_>>()
    );
}

#[test]
fn openapi_yaml_is_valid_with_search_path() {
    let v: YamlValue = serde_yaml::from_str(&openapi_yaml()).expect("openapi is valid YAML");
    let root = v.as_mapping().expect("OpenAPI YAML root is a mapping");
    let openapi = root
        .get(YamlValue::String("openapi".into()))
        .and_then(YamlValue::as_str);
    assert_eq!(
        openapi.map(|s| s.starts_with("3.")),
        Some(true),
        "OpenAPI 3.x YAML document"
    );
    let paths = root
        .get(YamlValue::String("paths".into()))
        .and_then(YamlValue::as_mapping)
        .expect("has paths");
    assert!(
        paths
            .keys()
            .filter_map(YamlValue::as_str)
            .any(|p| p.contains("/search")),
        "exposes a search path: {:?}",
        paths.keys().collect::<Vec<_>>()
    );
}

#[test]
fn json_schema_emits_component_schemas() {
    let v: Value = serde_json::from_str(&json_schema_json()).expect("json-schema is valid JSON");
    assert!(
        v["components"]["schemas"].is_object(),
        "components.schemas present (the request/response data types): {v}"
    );
}

#[test]
fn query_shapes_cover_core_node_types_and_carry_requests() {
    let v = query_shapes();
    let shapes = v["shapes"].as_array().expect("shapes array");
    let names: Vec<&str> = shapes.iter().map(|s| s["name"].as_str().unwrap()).collect();
    for required in [
        "term",
        "terms",
        "range",
        "match_bm25",
        "boolean_and",
        "boolean_not",
        "knn",
        "rrf_hybrid",
        "hamming_near_dup",
        "has_child_nested_group",
        "collapse_group_by",
        "filter_then_sort",
    ] {
        assert!(
            names.contains(&required),
            "cookbook missing shape `{required}`: {names:?}"
        );
    }
    for s in shapes {
        assert!(
            s["request"].is_object(),
            "shape {} carries a request body",
            s["name"]
        );
        assert!(
            s["description"].is_string(),
            "shape {} carries a description",
            s["name"]
        );
    }
}

#[test]
fn field_catalog_matches_the_real_enums() {
    let v = field_catalog();
    let types: Vec<&str> = v["field_types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["type"].as_str().unwrap())
        .collect();
    // Exactly the FieldType enum variants (lowercase wire form), in order.
    assert_eq!(
        types,
        ["text", "keyword", "number", "set", "vector", "hash"],
        "field types track the FieldType enum"
    );

    let analyzers: Vec<&str> = v["analyzers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    for a in ["whitespace_lower", "ngram", "jieba"] {
        assert!(
            analyzers.contains(&a),
            "analyzer `{a}` listed: {analyzers:?}"
        );
    }

    // Vector metrics match the VectorMetric enum (snake_case).
    let vector = v["field_types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["type"] == "vector")
        .unwrap();
    let metrics: Vec<&str> = vector["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m.as_str().unwrap())
        .collect();
    for m in ["cosine", "dot", "l2"] {
        assert!(
            metrics.contains(&m),
            "vector metric `{m}` listed: {metrics:?}"
        );
    }
}

// --- `lumen llm *` agent integration topics (offline) ----------------------

#[test]
fn llm_outline_maps_agent_topics() {
    let outline = llm_outline_md();
    assert!(!outline.trim().is_empty(), "outline is non-empty");
    for needle in [
        "lumen llm workflow",
        "lumen llm integration",
        "lumen llm quickstart",
        "lumen llm recipes",
        "lumen spec --format openapi-yaml",
        "lumen spec",
    ] {
        assert!(outline.contains(needle), "outline missing `{needle}`");
    }
}

#[test]
fn llm_workflow_covers_the_integration_model() {
    let g = llm_workflow_md();
    assert!(!g.trim().is_empty(), "workflow is non-empty");
    // Mental model + the 4-step workflow + flavor guide + non-goals must be
    // present so an agent can wire lumen in without a docs site.
    for needle in [
        "search index",   // mental model: not a database
        "external_id",    // returns ids, not documents
        "Declare",        // step 1
        "Ingest",         // step 2 (caller pub/sub)
        "Search",         // step 3
        "Hydrate",        // step 4
        "Which \"find\"", // flavor decision guide
        ":7373",          // connection
        "Do NOT",         // non-goals
    ] {
        assert!(g.contains(needle), "workflow missing `{needle}`");
    }
}

#[test]
fn llm_integration_recommends_postgres_alloydb_adapter_boundary() {
    let integration = llm_integration_md();
    assert!(
        !integration.trim().is_empty(),
        "integration topic is non-empty"
    );
    for needle in [
        "Recommended Postgres / AlloyDB integration",
        "outbox",
        "ACK/retry/DLQ",
        "Do not publish directly to lumen's broker stream",
        "Ownership boundary",
    ] {
        assert!(
            integration.contains(needle),
            "integration topic missing `{needle}`"
        );
    }
}

#[test]
fn llm_quickstart_is_a_copy_paste_end_to_end() {
    let q = llm_quickstart_md();
    assert!(!q.trim().is_empty(), "quickstart is non-empty");
    assert!(q.contains("curl"), "quickstart has runnable curl");
    for path in ["/collections/products", "/index", "/search"] {
        assert!(q.contains(path), "quickstart exercises `{path}`");
    }
}

#[test]
fn llm_recipes_render_every_cookbook_shape_without_drift() {
    let md = llm_recipes_md();
    assert!(!md.trim().is_empty(), "recipes non-empty");
    // Single source of truth: every shape name from query_shapes() appears as a
    // recipe heading, so the playbook never drifts from `spec --shapes`.
    let shapes = query_shapes();
    for s in shapes["shapes"].as_array().unwrap() {
        let name = s["name"].as_str().unwrap();
        assert!(
            md.contains(&format!("## {name}")),
            "recipes missing `{name}`"
        );
    }
    assert!(
        md.contains("## rrf_hybrid"),
        "recipes include the hybrid recipe"
    );
}

/// #200: the emitted OpenAPI must be self-complete (every `$ref` resolves to a
/// defined component schema) and advertise the real serving port 7373.
/// @spec projects/lumen/tech-design/interfaces/rest/lumen-openapi-define-4-dangling-ref-schemas-fix-servers-port-808.md
#[test]
fn openapi_is_self_complete_and_uses_port_7373() {
    let v: Value = serde_json::from_str(&openapi_json()).expect("openapi is valid JSON");

    let defined: std::collections::BTreeSet<String> = v["components"]["schemas"]
        .as_object()
        .expect("components.schemas object")
        .keys()
        .cloned()
        .collect();

    // Every `#/components/schemas/<Name>` reference must resolve to a definition.
    let text = v.to_string();
    let needle = "#/components/schemas/";
    let mut missing = std::collections::BTreeSet::new();
    let mut rest = text.as_str();
    while let Some(i) = rest.find(needle) {
        rest = &rest[i + needle.len()..];
        let end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        if !defined.contains(&rest[..end]) {
            missing.insert(rest[..end].to_string());
        }
    }
    assert!(missing.is_empty(), "dangling $refs in OpenAPI: {missing:?}");

    let servers: Vec<String> = v["servers"]
        .as_array()
        .expect("servers array")
        .iter()
        .filter_map(|s| s["url"].as_str().map(str::to_string))
        .collect();
    assert!(!servers.is_empty(), "servers block present");
    assert!(
        servers.iter().all(|u| u.contains(":7373")),
        "servers must use the real port :7373, got {servers:?}"
    );
}
// CODEGEN-END
