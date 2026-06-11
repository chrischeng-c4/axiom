// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-tests-spec-cli-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! `lumen spec` surface: the offline, machine-readable self-description an
//! agent reads to wire lumen into a pipeline. Each emitter must produce valid
//! JSON with the expected top-level shape (no server, no I/O).

use lumen::spec::{
    field_catalog, json_schema_json, llm_guide_md, llm_quickstart_md, llm_recipes_md, openapi_json,
    query_shapes,
};
use serde_json::Value;

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

// --- `lumen llm *` agent integration playbook (offline) --------------------

#[test]
fn llm_guide_covers_the_integration_model() {
    let g = llm_guide_md();
    assert!(!g.trim().is_empty(), "guide is non-empty");
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
        assert!(g.contains(needle), "guide missing `{needle}`");
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

// </HANDWRITE>
