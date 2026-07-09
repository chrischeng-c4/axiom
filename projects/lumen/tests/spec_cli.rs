// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! `lumen spec` surface: the offline, machine-readable self-description an
//! agent reads to wire lumen into a pipeline. Each emitter must produce valid
//! JSON with the expected top-level shape (no server, no I/O).

use lumen::spec::{
    field_catalog, json_schema_json, llm_auth_md, llm_deployment_md, llm_integration_md,
    llm_outline_md, llm_quickstart_md, llm_recipes_md, llm_storage_md, llm_workflow_md,
    openapi_json, openapi_yaml, query_shapes,
};
use serde_json::{json, Value};
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
    assert_eq!(
        v["components"]["securitySchemes"]["bearerAuth"]["scheme"], "bearer",
        "OpenAPI advertises the Authorization: Bearer token scheme"
    );
    assert_eq!(
        v["security"][0]["bearerAuth"],
        json!([]),
        "OpenAPI globally requires bearer auth for data-plane routes"
    );
    assert_eq!(
        v["paths"]["/healthz"]["get"]["security"],
        json!([{}]),
        "auth-exempt admin/probe routes override the global bearer requirement"
    );
}

/// #1271: `lumen spec` publishes the batch search endpoint and its request/
/// response schemas — the OpenAPI doc is generated straight from the live
/// router's `ApiDoc`, so this exercises the same source of truth the real
/// `lumen spec` CLI command serves.
#[test]
fn openapi_json_exposes_batch_search_endpoint_and_schemas() {
    let v: Value = serde_json::from_str(&openapi_json()).expect("openapi is valid JSON");
    let batch = &v["paths"]["/collections:search"]["post"];
    assert!(
        !batch.is_null(),
        "OpenAPI is missing POST /collections:search: {:?}",
        v["paths"].as_object().map(|p| p.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        batch["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/BatchSearchRequest",
        "batch search request body schema"
    );
    assert_eq!(
        batch["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/BatchSearchResponse",
        "batch search response schema"
    );
    for schema in [
        "BatchSearchRequest",
        "BatchSearchItem",
        "BatchSearchResponse",
        "BatchSearchResult",
    ] {
        assert!(
            !v["components"]["schemas"][schema].is_null(),
            "OpenAPI components missing schema `{schema}`"
        );
    }
}

/// #1292: `lumen spec` publishes the `docs:replace` batch endpoint, the
/// single-resource `docs/{external_id}` sugar endpoint, and their request/
/// response schemas — generated from the same live-router `ApiDoc` source
/// of truth `openapi_json_exposes_batch_search_endpoint_and_schemas` (#1271)
/// exercises.
#[test]
fn openapi_json_exposes_docs_replace_endpoints_and_schemas() {
    let v: Value = serde_json::from_str(&openapi_json()).expect("openapi is valid JSON");

    let batch = &v["paths"]["/collections/{collection_id}/docs:replace"]["put"];
    assert!(
        !batch.is_null(),
        "OpenAPI is missing PUT /collections/{{collection_id}}/docs:replace: {:?}",
        v["paths"].as_object().map(|p| p.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        batch["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/ReplaceDocsRequest",
        "docs:replace request body schema"
    );
    assert_eq!(
        batch["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/ReplaceDocsResponse",
        "docs:replace response schema"
    );

    let single = &v["paths"]["/collections/{collection_id}/docs/{external_id}"]["put"];
    assert!(
        !single.is_null(),
        "OpenAPI is missing PUT /collections/{{collection_id}}/docs/{{external_id}}: {:?}",
        v["paths"].as_object().map(|p| p.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        single["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/ReplaceDocBody",
        "single-resource docs/{{external_id}} request body schema"
    );
    assert_eq!(
        single["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/ReplaceDocResult",
        "single-resource docs/{{external_id}} response schema"
    );

    for schema in [
        "ReplaceDocsRequest",
        "ReplaceDocItem",
        "ReplaceDocsResponse",
        "ReplaceDocResult",
        "ReplaceDocBody",
    ] {
        assert!(
            !v["components"]["schemas"][schema].is_null(),
            "OpenAPI components missing schema `{schema}`"
        );
    }
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
fn json_schema_emits_token_registry_operational_schema() {
    let v: Value = serde_json::from_str(&json_schema_json()).expect("json-schema is valid JSON");
    let schema = &v["operationalSchemas"]["TokenRegistry"];
    assert_eq!(
        schema["type"], "object",
        "TokenRegistry is an object schema"
    );
    assert_eq!(
        schema["additionalProperties"]["properties"]["roles"]["additionalProperties"]["enum"],
        json!(["read", "write", "admin"]),
        "TokenRegistry publishes the exact role enum"
    );
    assert!(
        schema["examples"][0]["admin-token"]["roles"]["*"] == "admin",
        "TokenRegistry example includes wildcard admin role: {schema}"
    );
}

#[test]
// @spec projects/lumen/tech-design/logic/0-4-4-docs-stale-sort-missing-last-and-has-child-sort-both-work.md
fn search_request_sort_schema_documents_current_sort_behavior() {
    let v: Value = serde_json::from_str(&json_schema_json()).expect("json-schema is valid JSON");
    let desc = v["components"]["schemas"]["SearchRequest"]["properties"]["sort"]["description"]
        .as_str()
        .expect("SearchRequest.sort has a schema description");
    for needle in [
        "up to 4 keys",
        "`first`/`last` keep",
        "`has_child`",
        "exact `total`",
    ] {
        assert!(
            desc.contains(needle),
            "sort description missing `{needle}`: {desc}"
        );
    }
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
        "autocomplete_ngram",
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
    let has_child = shapes
        .iter()
        .find(|s| s["name"] == "has_child_nested_group")
        .expect("has_child query shape exists");
    let desc = has_child["description"].as_str().unwrap();
    assert!(
        desc.contains("parent-field sort"),
        "has_child shape description should mention parent-field sort: {desc}"
    );
    assert!(
        has_child["request"]["sort"].is_array(),
        "has_child shape should show sort composition: {has_child}"
    );
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

    // #825: the `lumen spec --fields` catalog must document how to declare
    // and query hash fields, matching the README field-type table.
    let hash = v["field_types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["type"] == "hash")
        .unwrap();
    assert_eq!(hash["schema"], json!({ "type": "hash" }));
    assert!(
        hash["value"].as_str().unwrap().contains("16-hex"),
        "hash catalog should document the 64-bit hex value shape: {hash}"
    );
    assert!(
        hash["queries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|q| q == "hamming"),
        "hash catalog should name hamming query support: {hash}"
    );
}

// --- `lumen llm *` agent integration topics (offline) ----------------------

/// #824: the outline must teach the convention-canonical `--topic` form, not
/// the positional form rejected by clap.
/// @spec projects/lumen/tech-design/interfaces/cli/self-docs-teach-positional-lumen-llm-topic-but-the-cli-only-acce.md#unit-test
#[test]
fn llm_outline_maps_agent_topics() {
    let outline = llm_outline_md();
    assert!(!outline.trim().is_empty(), "outline is non-empty");
    for needle in [
        "lumen llm --topic workflow",
        "lumen llm --topic integration",
        "lumen llm --topic quickstart",
        "lumen llm --topic auth",
        "lumen llm --topic deployment",
        "lumen llm --topic storage",
        "lumen llm --topic recipes",
        "lumen spec --format openapi-yaml",
        "lumen spec",
    ] {
        assert!(outline.contains(needle), "outline missing `{needle}`");
    }
    for rejected in [
        "`lumen llm workflow`",
        "`lumen llm integration`",
        "`lumen llm quickstart`",
        "`lumen llm auth`",
        "`lumen llm deployment`",
        "`lumen llm storage`",
        "`lumen llm recipes`",
    ] {
        assert!(
            !outline.contains(rejected),
            "outline advertises rejected positional command `{rejected}`"
        );
    }
}

#[test]
fn llm_auth_publishes_token_registry_shape() {
    let auth = llm_auth_md();
    assert!(!auth.trim().is_empty(), "auth topic is non-empty");
    for needle in [
        "LUMEN_AUTH=required",
        "LUMEN_TOKEN_REGISTRY_FILE=/var/run/secrets/lumen/token-registry.json",
        "LUMEN_TOKEN=<token>",
        "Authorization: Bearer <LUMEN_TOKEN>",
        "\"admin-token\"",
        "\"roles\"",
        "\"*\": \"admin\"",
        "\"products\": \"read\"",
        "tokensSecret",
        "Secret Manager",
        "Client",
        "auth_token",
        "default_headers",
        "Shared auth primitive",
        "<SVC>_TOKEN_REGISTRY_FILE",
        "service-auth",
    ] {
        assert!(auth.contains(needle), "auth topic missing `{needle}`");
    }
}

#[test]
fn llm_deployment_documents_shard_cluster_topology() {
    let deployment = llm_deployment_md();
    assert!(
        !deployment.trim().is_empty(),
        "deployment topic is non-empty"
    );
    for needle in [
        "lumen dockerfile render",
        "lumen k8s crd render",
        "lumen k8s operator render",
        "lumen k8s instance render",
        "spec.shardCount",
        "spec.replicasPerShard",
        "totalPods = shardCount * replicasPerShard",
        "shardIndex = ordinal % shardCount",
        "replicaIndex = ordinal / shardCount",
        "replicasPerShard: 1",
        "primary/follower replication",
        "replicasPerShard: 2",
        "replicasPerShard: 3",
        "voterCount",
        "HPA is for stateless or near-stateless serving capacity",
        "HPA-created pods in a single-member topology",
        "production data fan-out",
        "Dynamic shard growth",
        "50% of the configured shard ceiling",
        "versioned virtual-bucket map",
        "Search without a routing key scatters/gathers",
        "LUMEN_BOOTSTRAP_SEED_URI",
        "Backup is the cold disaster-recovery and seed surface",
        "Shared raft-host topology primitive",
        "RaftStateMachine",
        "REPLICAS_PER_SHARD > 1",
    ] {
        assert!(
            deployment.contains(needle),
            "deployment topic missing `{needle}`"
        );
    }
}

/// #812: the serving fleet is always a StatefulSet with a durable PVC-backed
/// WAL, including at `replicasPerShard: 1` — this must be discoverable
/// offline via `lumen llm --topic storage`, not only in the CRD doc comments.
/// @spec projects/lumen/tech-design/logic/render-serving-as-a-statefulset-unconditionally-even-at-replicas.md
#[test]
fn llm_storage_documents_unconditional_statefulset_pvc() {
    let storage = llm_storage_md();
    assert!(!storage.trim().is_empty(), "storage topic is non-empty");
    for needle in [
        "StatefulSet",
        "volumeClaimTemplates",
        "raft",
        "/var/lib/lumen",
        "replicasPerShard: 1",
        "20Gi",
        "no raft consensus",
        "legacy single-shard HPA path",
        "continuously catch",
    ] {
        assert!(storage.contains(needle), "storage topic missing `{needle}`");
    }
}

#[test]
fn llm_storage_documents_shard_replica_and_bootstrap_boundaries() {
    let storage = llm_storage_md();
    for needle in [
        "spec.shardCount",
        "spec.replicasPerShard",
        "shardCount * replicasPerShard",
        "shardIndex = ordinal % shardCount",
        "replicaIndex = ordinal / shardCount",
        "HPA does not change storage ownership",
        "Dynamic shard growth is an operator workflow",
        "storage pressure",
        "versioned virtual-bucket map",
        "bounded snapshot-batch",
        "Empty-PVC replica bootstrap",
        "LUMEN_BOOTSTRAP_SEED_URI",
        "before WAL/raft delta catch-up",
        "not the normal live replica synchronization mechanism",
        "Shared backup primitive",
        "BackupDestination",
        "fetch_backup_object",
        "Shared raft-host primitive",
        "RaftStateMachine",
    ] {
        assert!(storage.contains(needle), "storage topic missing `{needle}`");
    }
}

/// #834: clusters that reconciled `<=0.4.9` with the Deployment-backed
/// single-replica topology need an explicit handoff because `>=0.4.10`
/// renders the same serving fleet name as a StatefulSet and the operator does
/// not prune the stale different-kind Deployment automatically.
#[test]
fn llm_storage_documents_deployment_to_statefulset_upgrade_handoff() {
    let storage = llm_storage_md();
    for needle in [
        "<=0.4.9",
        ">=0.4.10",
        "Deployment/<name>",
        "StatefulSet/<name>",
        "does not prune a stale child object",
        "Apply the new CRD first",
        "GET /admin/backup",
        "Pause the old `<=0.4.9` operator reconciliation",
        "kubectl -n <ns> scale deployment/<name> --replicas=0",
        "kubectl -n <ns> delete deployment/<name> --wait=true",
        "kubectl -n <ns> rollout status statefulset/<name>",
        "Do not run both",
        "independent WAL",
        "does not copy a Deployment pod's filesystem",
        "restore an admin backup",
    ] {
        assert!(storage.contains(needle), "storage topic missing `{needle}`");
    }
}

/// #808 R1: the manual admin backup/restore procedure, the optional
/// `spec.serving.backup` CRD field, and the `lumen backup` CLI verb must all
/// be discoverable offline via `lumen llm --topic storage`.
/// @spec projects/lumen/tech-design/logic/no-snapshot-backup-mechanism-for-lumen-s-wal-any-replicaspershar.md
#[test]
fn llm_storage_documents_admin_backup_and_scheduled_cronjob() {
    let storage = llm_storage_md();
    for needle in [
        "GET /admin/backup",
        "POST /admin/backup/local",
        "POST /admin/restore",
        "Role::Admin",
        "spec.serving.backup",
        "schedule",
        "destination",
        "retentionSecs",
        "adminTokenSecret",
        "lumen backup",
        "LUMEN_BACKUP_TOKEN",
        "--retention-secs",
    ] {
        assert!(storage.contains(needle), "storage topic missing `{needle}`");
    }
}

/// #809: a `spec.serving.raftStorage` CR edit does not, by itself, resize
/// existing per-pod PVCs (StatefulSet `volumeClaimTemplates` are immutable
/// after creation) — the manual patch procedure, its `StorageClass`
/// precondition, the shrink limitation, and the `resize-storage` CLI helper
/// must all be discoverable offline via `lumen llm --topic storage`.
/// @spec projects/lumen/tech-design/logic/raftstorage-pvc-has-no-auto-expansion-cr-field-change-doesn-t-re.md
#[test]
fn llm_storage_documents_resize_gap() {
    let storage = llm_storage_md();
    for needle in [
        "Resizing",
        "volumeClaimTemplates",
        "immutable",
        "kubectl patch pvc",
        "allowVolumeExpansion: true",
        "does not support shrinking",
        "lumen k8s operator resize-storage",
        "--namespace",
        "--dry-run",
    ] {
        assert!(storage.contains(needle), "storage topic missing `{needle}`");
    }
}

/// #810: `serving.raftStorageClass` unset means cluster default, which is
/// commonly not SSD-backed (e.g. GKE's `standard-rwo`) — a deployer with a
/// raft/WAL write-latency workload needs this called out explicitly, plus
/// reference example StorageClass names per common provider, rather than
/// only in the CRD field doc comment. Documentation-only: no `serving.ssd`
/// toggle, no new CRD field.
/// @spec projects/lumen/tech-design/logic/expose-ssd-as-a-simple-toggle-serving-ssd-instead-of-requiring-a.md
#[test]
fn llm_storage_documents_ssd_guidance() {
    let storage = llm_storage_md();
    for needle in [
        "raftStorageClass",
        "cluster default is not SSD-backed",
        "standard-rwo",
        "premium-rwo",
        "pd-ssd",
        "gp3",
        "managed-csi-premium",
        "kubectl get storageclass",
        "serving.ssd",
    ] {
        assert!(storage.contains(needle), "storage topic missing `{needle}`");
    }
}

#[test]
fn llm_workflow_covers_the_integration_model() {
    let g = llm_workflow_md();
    assert!(!g.trim().is_empty(), "workflow is non-empty");
    // Mental model + the 4-step workflow + flavor guide + non-goals must be
    // present so an agent can wire lumen in without a docs site.
    for needle in [
        "search index",         // mental model: not a database
        "external_id",          // returns ids, not documents
        "Declare",              // step 1
        "Ingest",               // step 2 (caller pub/sub)
        "Search",               // step 3
        "Hydrate",              // step 4
        "Which \"find\"",       // flavor decision guide
        "parent-field `sort`",  // has_child + parent-field sort support
        "Geo / spatial search", // explicit unsupported/search-boundary list
        "Phrase / proximity queries",
        "Fuzzy / typo tolerance",
        "Synonyms",
        "Autocomplete / suggest",
        "Highlighting",
        "Per-field / per-clause boost",
        "Document TTL / expiry",
        ":7373", // connection
        "compatibility/smoke path",
        "high-QPS",
        "pooled HTTP/2 streams",
        "Authorization: Bearer",
        "LUMEN_TOKEN_REGISTRY_FILE",
        "Do NOT", // non-goals
    ] {
        assert!(g.contains(needle), "workflow missing `{needle}`");
    }
}

/// #1271: the workflow topic must teach the batch search verb — endpoint,
/// request/response shape, partial-failure semantics, and the batch limit —
/// so an agent reading `lumen llm --topic workflow` learns it without a docs
/// site.
#[test]
fn llm_workflow_documents_batch_search_endpoint() {
    let g = llm_workflow_md();
    for needle in [
        "POST /collections:search",
        "concurrent fan-out",
        "\"searches\"",
        "\"results\"",
        "\"status\": \"ok\"",
        "\"status\": \"error\"",
        "collection_not_found",
        "Partial failure never fails the batch",
        "Max batch size is 32",
        "no merged cursor",
    ] {
        assert!(
            g.contains(needle),
            "workflow missing batch search `{needle}`"
        );
    }
}

/// #1271: the outline must point an agent at the batch search verb from the
/// workflow topic entry (not invent a dedicated `--topic batch`, which would
/// duplicate the single search-flavor topic map).
#[test]
fn llm_outline_mentions_batch_search() {
    let outline = llm_outline_md();
    assert!(
        outline.contains("collections:search"),
        "outline should point at batch search: {outline}"
    );
}

/// #1292: the workflow topic must document the docs:replace full-replacement
/// write surface — implicit field deletion, doc-level LWW `version`, the
/// `/index`-vs-`docs:replace` division rule, and the single-resource sugar
/// endpoint — mirroring the batch search topic's coverage pattern (#1271).
#[test]
fn llm_workflow_documents_docs_replace_endpoint() {
    let g = llm_workflow_md();
    for needle in [
        "PUT /collections/{id}/docs:replace",
        "implicitly deleted",
        "Own the complete row for a doc?",
        "doc-level",
        "current_version",
        "Partial failure never fails the batch",
        "MAX_BATCH_REPLACE_SIZE",
        "PUT /collections/{id}/docs/{external_id}",
    ] {
        assert!(
            g.contains(needle),
            "workflow missing docs:replace `{needle}`"
        );
    }
}

/// #1292: the outline must point an agent at the docs:replace full-replacement
/// write verb from the workflow topic entry.
#[test]
fn llm_outline_mentions_docs_replace() {
    let outline = llm_outline_md();
    assert!(
        outline.contains("docs:replace"),
        "outline should point at docs:replace: {outline}"
    );
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
        "Do not publish directly to lumen's internal WAL",
        "Ownership boundary",
        "Shared generated-client primitive",
        "spec gen --lang ts|py|rust --out <dir>",
        "GeneratedOutput",
        "target_concurrency",
        "max_in_flight_per_origin",
        "pool_timeout",
        "Shared h2c client primitive",
        "ceil(ln(concurrency))",
        "max_keepalive_connections",
        "Server boundary",
        "inbound traffic",
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
    assert!(
        q.contains("LUMEN_TOKEN_REGISTRY_FILE"),
        "quickstart documents production auth env"
    );
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
