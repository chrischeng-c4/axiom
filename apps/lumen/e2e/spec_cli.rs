// CODEGEN-BEGIN
//! `lumen spec` surface: the offline, machine-readable self-description an
//! agent reads to wire lumen into a pipeline. Each emitter must produce valid
//! JSON with the expected top-level shape (no server, no I/O).
//!
//! ## Contracts inherited from the retired EC shells
//!
//! These 12 sentences were the whole of the `// Contract:` comment in 12 AW-EC shells
//! under `apps/lumen/e2e/`, each of which ran `cargo test -p lumen --test spec_cli` in
//! a subprocess and asserted the child's exit status. `cargo test -p lumen` already
//! runs this target directly, so the shells added a second, nested run and nothing
//! else. They were deleted on 2026-08-20 with the EC machinery they belonged to, and
//! the sentence is the only thing they held that nothing else did. Each line below is
//! prefixed with the EC id its shell was filed under.
//!
//! - `lumen-claim-agent-llm-topics` — The offline cclab.llm.v2 outline publishes every
//!   typed Lumen task and each task emits a source-backed Markdown/JSON runbook.
//! - `lumen-claim-agent-offline-spec` — Offline schema commands produce valid OpenAPI
//!   JSON/YAML and JSON-schema output for agents.
//! - `lumen-claim-agent-query-catalog` — The offline query-shape and field/analyzer
//!   catalogs remain deterministic for agent ingestion.
//! - `lumen-claim-cli-llm-v2-task-navigation` — The public lumen llm command renders
//!   deterministic Markdown and typed JSON from one source-backed cclab.llm.v2
//!   contract.
//! - `lumen-claim-cli-standard-llm-entrypoint` — The shared `lumen llm` entrypoint
//!   publishes the agent topic set through the standard CLI convention.
//! - `lumen-claim-developer-integration-contract` — The integration contract documents
//!   routed retry semantics and the reshard administration boundary from the canonical
//!   source model.
//! - `lumen-claim-developer-llm-v2-task-navigation` — Agent task navigation exposes one
//!   deterministic typed contract in Markdown and JSON.
//! - `lumen-claim-http2-offline-spec-list` — The offline spec commands publish the
//!   supported HTTP API inventory.
//! - `lumen-claim-standard-offline-openapi` — The offline `lumen spec` OpenAPI output
//!   remains valid and includes the operational search route.
//! - `lumen-cli-interface-llm-playbook` — lumen llm outline publishes the cclab.llm.v2
//!   typed task manifest and every advertised topic parses through the binary.
//! - `lumen-cli-interface-offline-cli` — lumen spec emits valid OpenAPI JSON, OpenAPI
//!   YAML, and JSON-schema output offline.
//! - `lumen-cli-interface-query-catalog` — lumen spec exposes query-shape, field,
//!   analyzer, and vector-metric catalogs.

use lumen::spec::{
    field_catalog, json_schema_json, llm_auth_md, llm_deployment_md, llm_integration_md,
    llm_outline_md, llm_quickstart_md, llm_recipes_md, llm_storage_md, llm_workflow_md,
    openapi_json, openapi_yaml, query_shapes,
};
use lumen::{dx, types::FieldType};
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
// #2871 retired the bearer/identity registry, so `lumen spec --format
// json-schema` must not publish a `TokenRegistry` operational schema. A schema
// for a file no code reads is a supported-looking deployment shape an
// integrator would build against and never get authenticated by.
fn json_schema_no_longer_publishes_a_token_registry_schema() {
    let v: Value = serde_json::from_str(&json_schema_json()).expect("json-schema is valid JSON");
    assert!(
        v.get("operationalSchemas").is_none(),
        "no operationalSchemas block survives the registry retirement: {v}"
    );
    assert!(
        !json_schema_json().contains("token-registry.json"),
        "the retired registry file is not named anywhere in the published schema"
    );
}

#[test]
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
        "prefix",
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
        "native_offset",
        "search_all",
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

/// The auth topic is what an agent reads before wiring a client, so it is the
/// one place a stale credential story does the most damage (#3113 AC6). It must
/// state both halves of the production contract — private ClusterIP TLS the
/// listener terminates itself, and KSA identity the cluster answers — and stop
/// handing out a registry file shape to fill in.
#[test]
fn llm_auth_states_the_private_clusterip_tls_and_ksa_contract() {
    let auth = llm_auth_md();
    assert!(!auth.trim().is_empty(), "auth topic is non-empty");
    for needle in [
        // Request identity: the cluster answers it, and only for a KSA.
        "LUMEN_AUTH=required",
        "LUMEN_AUTH=disabled",
        "TokenReview",
        "SubjectAccessReview",
        "system:auth-delegator",
        "system:serviceaccount:<namespace>:<name>",
        "lumencollections",
        "lumenadmin",
        "tokensSecret",
        // Transport: private ClusterIP TLS, terminated by lumen itself.
        "LUMEN_URL=https://<instance>.<namespace>.svc:7373",
        "spec.servingTlsSecret",
        "LUMEN_TLS_SERVER_NAMES",
        "public CA",
        // Clients, generated and CLI.
        "--ca-file",
        "with_private_ca",
        "PrivateTrust",
        "auth_token",
        "default_headers",
        "Shared auth primitive",
        "service-auth",
    ] {
        assert!(auth.contains(needle), "auth topic missing `{needle}`");
    }

    let lumen_half = auth
        .split("\n## Shared auth primitive\n")
        .next()
        .expect("auth topic has a lumen-authored half");

    // AC6: production is a private ClusterIP the listener terminates. An agent
    // that reads "Ingress" here builds the one topology where the last hop is
    // unauthenticated while every client-side check still passes.
    assert!(
        lumen_half.contains(
            "There is no Ingress, no Gateway, no\nLoadBalancer, no NodePort, and no service mesh terminating TLS"
        ),
        "each published-edge shape has to be named and ruled out in one breath; \
         staying silent about them, or listing them separately, reads as a menu"
    );
    assert!(
        lumen_half.contains("Production traffic is **not** published"),
        "the topic must say what production is before it says what it is not"
    );

    // The registry file shape is gone, not merely deprecated in place: an
    // example an agent can copy is the thing that outlives the prose around it.
    // Only lumen's own half is asserted on — the appended `service-auth` topic
    // is the shared library's contract, still live for the services that have
    // not migrated.
    for retired in [
        "token-registry.json",
        "LUMEN_TOKEN_REGISTRY_FILE",
        "\"admin-token\"",
        "\"*\": \"admin\"",
    ] {
        assert!(
            !lumen_half.contains(retired),
            "auth topic still publishes retired registry detail `{retired}`"
        );
    }

    // No generated or documented client may be taught to stop checking.
    for weakening in WEAKENINGS {
        assert!(
            !lumen_half.contains(weakening),
            "auth topic teaches `{weakening}`; verification is never the thing to turn off"
        );
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
        "Shared raft-runtime topology primitive",
        "RaftStateMachine",
        "REPLICAS_PER_SHARD > 1",
    ] {
        assert!(
            deployment.contains(needle),
            "deployment topic missing `{needle}`"
        );
    }
}

/// #3113 AC6: whoever renders the deployment artifacts decides where TLS is
/// terminated, so the deployment topic — not only the auth topic — has to say
/// that lumen terminates it itself on a private ClusterIP. An operator who
/// reads this and reaches for an Ingress builds the one topology where the
/// last hop carries a bearer token in the clear while every client-side check
/// still passes.
#[test]
fn llm_deployment_states_the_private_clusterip_tls_contract() {
    let deployment = llm_deployment_md();
    for needle in [
        "LUMEN_URL=https://<instance>.<namespace>.svc:7373",
        "spec.servingTlsSecret",
        "spec.peerTlsSecret",
        "LUMEN_TLS_CERT",
        "LUMEN_TLS_KEY",
        "LUMEN_TLS_CA",
        "LUMEN_TLS_SERVER_NAMES",
        "public CA",
        "--ca-file",
        "PrivateTrust",
        // The anchor is published without the key, and replaces the public
        // roots rather than joining them. Both are the whole point of a
        // private trust domain, and both are easy to get subtly wrong.
        "private-key-bearing serving Secret",
        "replaces the public roots",
    ] {
        assert!(
            deployment.contains(needle),
            "deployment topic missing `{needle}`"
        );
    }

    assert!(
        deployment.contains(
            "There is no Ingress, no Gateway, no LoadBalancer, no NodePort, and no service\nmesh terminating TLS on lumen's behalf."
        ),
        "each published-edge shape has to be named and ruled out in one breath; \
         staying silent about them, or listing them separately, reads as a menu"
    );
    assert!(
        deployment.contains("Production traffic is **not** published"),
        "the topic must say what production is before it says what it is not"
    );

    for weakening in WEAKENINGS {
        assert!(
            !deployment.contains(weakening),
            "deployment topic teaches `{weakening}`; verification is never the thing to turn off"
        );
    }
}

/// #812: the serving fleet is always a StatefulSet with a durable PVC-backed
/// WAL, including at `replicasPerShard: 1` — this must be discoverable
/// offline via `lumen llm --topic storage`, not only in the CRD doc comments.
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
        "10Gi",
        "no raft consensus",
        "legacy single-shard HPA path",
        "continuously catch",
    ] {
        assert!(storage.contains(needle), "storage topic missing `{needle}`");
    }
}

/// #1387: the operator now activates PVC-backed embedded persistence at
/// `replicasPerShard: 1` (`LUMEN_DATA_DIR` + `LUMEN_PERSISTENCE=segment`),
/// with the disjoint raft subtree, actual crash-durability semantics
/// (`everysec` AOF fsync, ~1s RPO — not the `LUMEN_SNAPSHOT_SECS` interval),
/// and the bare-`lumen serve` dev-mode caveat all discoverable offline via
/// `lumen llm --topic storage`.
#[test]
fn llm_storage_documents_embedded_mode_persistence_wiring() {
    let storage = llm_storage_md();
    for needle in [
        "LUMEN_WAL=auto",
        "MemWal",
        "LUMEN_DATA_DIR=/var/lib/lumen/data",
        "LUMEN_PERSISTENCE=segment",
        "/var/lib/lumen/raft",
        "everysec",
        "src/aof.rs",
        "src/segment_rdb.rs",
        "~1s",
        "LUMEN_SNAPSHOT_SECS",
        "Dev mode: bare `lumen serve` stays in-memory",
        "--data-dir",
        "in-memory",
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
        "Shared raft-runtime primitive",
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
        "--retention-secs",
    ] {
        assert!(storage.contains(needle), "storage topic missing `{needle}`");
    }
    // #2871: the CronJob's metadata-server ID-token fallback is gone, so the
    // topic must not still name the audience list that selected it.
    assert!(
        !storage.contains("LUMEN_AUTH_GOOGLE_AUDIENCES"),
        "storage topic still points `lumen backup` at a retired Google-token path"
    );
}

/// #809: a `spec.serving.raftStorage` CR edit does not, by itself, resize
/// existing per-pod PVCs (StatefulSet `volumeClaimTemplates` are immutable
/// after creation) — the manual patch procedure, its `StorageClass`
/// precondition, the shrink limitation, and the `resize-storage` CLI helper
/// must all be discoverable offline via `lumen llm --topic storage`.
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
        "Do NOT", // non-goals
    ] {
        assert!(g.contains(needle), "workflow missing `{needle}`");
    }
    // #2871: the connection section no longer tells a caller to send a bearer
    // or points at a registry file the server stopped reading.
    for retired in ["Authorization: Bearer", "LUMEN_TOKEN_REGISTRY_FILE"] {
        assert!(
            !g.contains(retired),
            "workflow topic still teaches the retired bearer path `{retired}`"
        );
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

/// #1297 (epic #1296 R1): the workflow topic must recommend the RFC 10008
/// `QUERY` method for search — `QUERY /collections/{id}` + `QUERY
/// /collections` as dual-registered twins of the POST search endpoints —
/// and always document `POST` as the permanent, always-available fallback,
/// never a deprecated path.
#[test]
fn llm_workflow_documents_query_method_first_with_post_fallback() {
    let g = llm_workflow_md();
    for needle in [
        "QUERY method (RFC 10008)",
        "QUERY-first, POST-always-available",
        "QUERY /collections/{id}",
        "QUERY /collections",
        "byte-identical response",
        "is the permanent fallback",
        "Content-Type: application/json` is mandatory",
        "Accept-Query: application/json",
    ] {
        assert!(g.contains(needle), "workflow missing QUERY `{needle}`");
    }
}

/// #3113 AC6: the workflow topic's Connection section is where an agent learns
/// what URL to build. It used to say lumen speaks cleartext and carries no
/// credential, full stop — true of a localhost node, and a recipe for sending a
/// KSA token in the clear at a production fleet. Production and development are
/// now two named cases, and the production one names the anchor and the check.
#[test]
fn llm_workflow_separates_production_tls_from_local_h2c() {
    let workflow = llm_workflow_md();
    for needle in [
        "https://<instance>.<namespace>.svc:7373",
        "http://localhost:7373",
        "public CA distributed separately",
        "TokenReview",
        "SubjectAccessReview",
        "in place of\n  the public roots",
    ] {
        assert!(
            workflow.contains(needle),
            "workflow Connection missing `{needle}`"
        );
    }
    assert!(
        !workflow.contains("has not landed"),
        "workflow still claims the KSA verifier is missing"
    );
    assert!(
        !workflow.contains("Requests carry no credential in this"),
        "workflow still states the retired build-wide no-credential claim as fact"
    );
}

/// #1297: the outline must point an agent at QUERY-first search guidance
/// from the workflow topic entry, mirroring the batch-search outline pointer
/// (#1271).
#[test]
fn llm_outline_mentions_query_first() {
    let outline = llm_outline_md();
    assert!(
        outline.contains("QUERY-first"),
        "outline should point at QUERY-first search: {outline}"
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

/// #1398 R5/AC4: the workflow topic must disclose that `X-Read-Consistency:
/// bounded(<ms>)` always rejects on a follower today (no real replication
/// lag measurement yet — a follower reports the "lag unknown" sentinel),
/// and that the headerless/unrecognized default stays `leader`, so this
/// text can't silently regress back to promising follower reads "at or
/// under the bound".
#[test]
fn llm_workflow_discloses_bounded_read_consistency_narrowing() {
    let g = llm_workflow_md();
    for needle in [
        "X-Read-Consistency",
        "leader` — the default",
        "always rejects today",
        "lag unknown",
        "do not rely on it to read from a follower",
        "standalone deployments (no",
    ] {
        assert!(
            g.to_lowercase().contains(&needle.to_lowercase()),
            "workflow missing read-consistency disclosure `{needle}`"
        );
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
    // #2871: the quickstart used to hand out the production auth env. There is
    // none to hand out, so it must say the local node is open rather than imply
    // a credential the reader could go find.
    assert!(
        !q.contains("LUMEN_TOKEN_REGISTRY_FILE"),
        "quickstart still names the retired registry env"
    );
    assert!(
        q.contains("LUMEN_AUTH=disabled"),
        "quickstart names the mode the local node it targets runs in"
    );
    // #3113 AC6: `disabled` is a property of the localhost node this topic
    // targets, not of the build. Between #2871 and #2869/#2878 it was both, and
    // the text said so; leaving that in would tell a reader that the open node
    // in front of them is the only thing lumen can be, and that reaching a
    // production fleet the same way is fine.
    for stale in [
        "has not landed",
        "the only mode a server starts in",
        "has not landed (#2871)",
    ] {
        assert!(
            !q.contains(stale),
            "quickstart still claims the KSA verifier is missing (`{stale}`)"
        );
    }
    for needle in [
        "https://<instance>.<namespace>.svc:7373",
        "public CA distributed separately",
        "TokenReview",
    ] {
        assert!(
            q.contains(needle),
            "quickstart does not say how production differs (`{needle}`)"
        );
    }
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

/// #1298 (epic #1296): the offline `lumen spec` OpenAPI document is stamped
/// OpenAPI 3.2 and describes the #1297 `QUERY` twins — `QUERY /collections`
/// and `QUERY /collections/{collection_id}` — each carrying the
/// `x-post-twin` extension libs/openapi-codegen's IR resolves the POST
/// fallback path from, alongside the POST twin itself still being
/// registered.
#[test]
fn openapi_json_declares_3_2_and_describes_query_twins() {
    let v: Value = serde_json::from_str(&openapi_json()).expect("openapi is valid JSON");
    assert_eq!(
        v["openapi"], "3.2.0",
        "OpenAPI document declares 3.2 (RFC 10008 QUERY support)"
    );

    let collections_query = &v["paths"]["/collections"]["query"];
    assert!(
        !collections_query.is_null(),
        "OpenAPI is missing QUERY /collections: {:?}",
        v["paths"]["/collections"]
            .as_object()
            .map(|p| p.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        collections_query["x-post-twin"], "/collections:search",
        "QUERY /collections names its POST twin"
    );
    assert_eq!(
        collections_query["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/BatchSearchRequest",
        "QUERY /collections request body schema matches its POST twin"
    );
    assert_eq!(
        collections_query["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/BatchSearchResponse",
        "QUERY /collections response schema matches its POST twin"
    );
    assert!(
        !v["paths"]["/collections:search"]["post"].is_null(),
        "QUERY /collections keeps its POST twin registered"
    );

    let collection_id_query = &v["paths"]["/collections/{collection_id}"]["query"];
    assert!(
        !collection_id_query.is_null(),
        "OpenAPI is missing QUERY /collections/{{collection_id}}: {:?}",
        v["paths"]["/collections/{collection_id}"]
            .as_object()
            .map(|p| p.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        collection_id_query["x-post-twin"], "/collections/{collection_id}/search",
        "QUERY /collections/{{collection_id}} names its POST twin"
    );
    assert_eq!(
        collection_id_query["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SearchRequest",
        "QUERY /collections/{{collection_id}} request body schema matches its POST twin"
    );
    assert_eq!(
        collection_id_query["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SearchResponse",
        "QUERY /collections/{{collection_id}} response schema matches its POST twin"
    );
    assert!(
        !v["paths"]["/collections/{collection_id}/search"]["post"].is_null(),
        "QUERY /collections/{{collection_id}} keeps its POST twin registered"
    );
}

/// #1480 R1: `clients/openapi.json` is a committed snapshot of
/// `lumen spec --format openapi`'s live output, not a hand-maintained copy —
/// it must byte-match live generation exactly so the offline contract cannot
/// silently lag the surface it
/// describes. Modeled on `openapi_is_valid_json_with_search_path`.
#[test]
fn openapi_committed_snapshot_matches_live_generation() {
    let committed = include_str!("../clients/openapi.json");
    let live = openapi_json();
    assert_eq!(
        committed, live,
        "clients/openapi.json is stale: regenerate via \
         `./target/debug/lumen spec --format openapi > clients/openapi.json` \
         (CONTRIBUTING.md DX convention: offline-contract must not lag the live surface)"
    );
}

#[test]
fn dx_field_catalog_matches_runtime_field_capabilities() {
    let catalog = dx::field_catalog();
    let fields = catalog["field_types"]
        .as_array()
        .expect("field catalogue is an array");
    assert_eq!(fields.len(), FieldType::ALL.len());

    for field_type in FieldType::ALL {
        let name = match field_type {
            FieldType::Text => "text",
            FieldType::Keyword => "keyword",
            FieldType::Number => "number",
            FieldType::Set => "set",
            FieldType::Vector => "vector",
            FieldType::Hash => "hash",
        };
        let entry = fields
            .iter()
            .find(|entry| entry["type"] == name)
            .unwrap_or_else(|| panic!("catalogue omits {name}"));
        let operations = &entry["operations"];
        let expected = field_type.capabilities();
        assert_eq!(operations["bm25"], expected.bm25);
        assert_eq!(operations["exact"], expected.exact);
        assert_eq!(operations["prefix"], expected.prefix);
        assert_eq!(operations["range"], expected.range);
        assert_eq!(operations["sort"], expected.sort);
        assert_eq!(operations["set_membership"], expected.set_membership);
        assert_eq!(operations["vector_search"], expected.vector_search);
        assert_eq!(operations["hamming"], expected.hamming);
    }

    let text = fields.iter().find(|entry| entry["type"] == "text").unwrap();
    assert_eq!(text["operations"]["bm25"], true);
    assert_eq!(text["operations"]["prefix"], false);
    assert_eq!(text["operations"]["range"], false);
    assert_eq!(text["operations"]["sort"], false);
    for field in ["keyword", "number"] {
        let entry = fields.iter().find(|entry| entry["type"] == field).unwrap();
        assert_eq!(entry["operations"]["range"], true);
        assert_eq!(entry["operations"]["sort"], true);
    }
    let keyword = fields
        .iter()
        .find(|entry| entry["type"] == "keyword")
        .unwrap();
    assert_eq!(keyword["operations"]["prefix"], true);
    assert!(keyword["queries"]
        .as_array()
        .expect("keyword queries")
        .contains(&json!("prefix")));
}

#[test]
fn openapi_exposes_native_offset_prefix_and_search_all_contracts() {
    let spec: Value = serde_json::from_str(&openapi_json()).expect("valid OpenAPI JSON");
    let search_request = &spec["components"]["schemas"]["SearchRequest"];
    assert_eq!(search_request["properties"]["offset"]["type"], "integer");
    assert_eq!(search_request["properties"]["offset"]["default"], 0);

    let query_node = &spec["components"]["schemas"]["QueryNode"];
    assert!(query_node.to_string().contains("PrefixQuery"));
    assert!(!spec["components"]["schemas"]["PrefixQuery"].is_null());

    let operation = &spec["paths"]["/collections/{collection_id}/search:all"]["post"];
    assert_eq!(operation["operationId"], "search_all");
    assert_eq!(
        operation["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SearchAllRequest"
    );
    assert_eq!(
        operation["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SearchAllResponse"
    );
}

/// #3113 AC6: `lumen llm --topic <t>` renders from the DX contract, not from
/// the `llm_*_md()` bodies asserted elsewhere in this file — so this is the
/// text an agent driving the CLI actually receives. Three topics decide
/// whether it builds a plaintext production path: `authenticate` (what the
/// token crosses), `deploy-kubernetes` (whether the manifest turns TLS on),
/// and `connect-kubernetes` (whether the developer path verifies anything).
/// Each needs the contract in its own topic; an agent reads one, not all.
#[test]
fn dx_topics_teach_the_private_clusterip_tls_contract() {
    let md = |topic: &str| dx::render_llm(topic, cli_std::llm::Format::Md).unwrap();

    let authenticate = md("authenticate");
    for needle in [
        "https://<instance>.<namespace>.svc:7373",
        "public CA",
        "in place of the public roots",
    ] {
        assert!(
            authenticate.contains(needle),
            "`authenticate` topic missing `{needle}`:\n{authenticate}"
        );
    }

    let deploy = md("deploy-kubernetes");
    for needle in [
        "spec.servingTlsSecret",
        "spec.peerTlsSecret",
        "it is cleartext",
        "Ingress, Gateway, LoadBalancer, NodePort, or mesh TLS terminator",
    ] {
        assert!(
            deploy.contains(needle),
            "`deploy-kubernetes` topic missing `{needle}`:\n{deploy}"
        );
    }

    let connect = md("connect-kubernetes");
    for needle in [
        "--ca-file",
        "SNI",
        "there is no flag that skips verification",
    ] {
        assert!(
            connect.contains(needle),
            "`connect-kubernetes` topic missing `{needle}`:\n{connect}"
        );
    }

    // The one instruction that must never appear: every topic above exists to
    // make verification possible, so none of them may also teach a way around
    // it. A single such string turns the whole contract into a suggestion.
    for topic in ["authenticate", "deploy-kubernetes", "connect-kubernetes"] {
        let text = md(topic);
        for weakening in WEAKENINGS {
            assert!(
                !text.contains(weakening),
                "`{topic}` topic teaches `{weakening}`"
            );
        }
    }
}

#[test]
fn dx_deploy_kubernetes_teaches_bounded_placement_split() {
    let rendered = dx::render_llm("deploy-kubernetes", cli_std::llm::Format::Json).unwrap();
    let value: Value = serde_json::from_str(&rendered).expect("deploy topic JSON parses");
    let markdown = value["markdown"]
        .as_str()
        .expect("deploy topic contains rendered Markdown");

    for needle in [
        "Placement support is Limited",
        "not the full kubernetes-native-placement roadmap target",
        "non-empty placement.nodeSelector with the default placement.initialMachineType skips the legacy capacity catalog",
        "preserves the exact selector and tolerations",
        "An empty placement.nodeSelector, tolerations-only placement, or a non-default placement.initialMachineType stays on the legacy capacity-catalog path",
        "The dev instance renderer supplies kubernetes.io/os: linux",
        "Staging and prod remain on the legacy placement path in 0.4.28",
    ] {
        assert!(
            markdown.contains(needle),
            "`deploy-kubernetes` topic missing `{needle}`:\n{markdown}"
        );
    }
}

#[test]
fn dx_llm_v2_json_and_markdown_share_one_typed_contract() {
    let protocol = dx::llm_protocol();
    assert_eq!(protocol.topics().len(), 14);
    for topic in protocol.topics() {
        let json = dx::render_llm(&topic.task.topic, cli_std::llm::Format::Json).unwrap();
        let value: Value = serde_json::from_str(&json).expect("runbook JSON parses");
        let markdown = dx::render_llm(&topic.task.topic, cli_std::llm::Format::Md).unwrap();
        assert_eq!(value["protocol"], "cclab.llm.v2");
        assert_eq!(value["topic"], topic.task.topic);
        assert!(value["markdown"].is_string());
        assert!(value["runbook"]["purpose"].is_string());
        assert_eq!(
            value["markdown"].as_str(),
            Some(markdown.as_str()),
            "Markdown must be rendered from the JSON runbook model"
        );
        assert!(
            value.get("next").is_none(),
            "LLM output never advertises an unbound next command"
        );
        for step in &topic.runbook.steps {
            assert_ne!(step.command.is_some(), step.command_template.is_some());
            if let Some(command) = &step.command {
                assert!(step.inputs.is_empty());
                assert!(!command.contains('{') && !command.contains('<'));
            }
            if step.command_template.is_some() {
                assert!(!step.inputs.is_empty());
            }
        }
    }

    let outline: Value =
        serde_json::from_str(&dx::render_llm("outline", cli_std::llm::Format::Json).unwrap())
            .unwrap();
    for required in [
        "run-standalone",
        "local-search",
        "model-schema",
        "select-query",
        "querying",
        "integrate-source-db",
        "authenticate",
        "connect-kubernetes",
        "deploy-kubernetes",
        "grant-access",
        "backup-restore",
        "generate-client",
        "diagnose",
        "verify-release",
    ] {
        assert!(
            outline["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["id"] == required),
            "outline omits {required}"
        );
    }
}

#[test]
fn dx_llm_composes_library_owned_provider_content() {
    let outline = dx::render_llm("outline", cli_std::llm::Format::Json).unwrap();
    assert!(
        !outline.contains("\"providers\""),
        "provider details must not change the outline envelope: {outline}"
    );

    let generated = dx::render_llm("generate-client", cli_std::llm::Format::Json).unwrap();
    let value: Value = serde_json::from_str(&generated).expect("generated-client JSON parses");
    assert!(value["task"]["reads"]
        .as_array()
        .unwrap()
        .iter()
        .any(|source| source == "apps/lumen/clients/codegen.toml"));
    assert_eq!(value["providers"].as_array().unwrap().len(), 1);
    assert_eq!(value["providers"][0]["id"], "openapi-codegen");
    let provider = openapi_codegen::llm::topic();
    assert_eq!(value["providers"][0]["summary"], provider.summary);
    let library_body = provider.body;
    assert_eq!(value["providers"][0]["markdown"], library_body);
    let rendered_markdown = value["markdown"].as_str().unwrap();
    assert!(rendered_markdown.contains(library_body));
    assert!(
        rendered_markdown.find("## Verification").unwrap()
            < rendered_markdown.find(library_body).unwrap(),
        "provider Markdown must follow the app-owned runbook"
    );

    let release = dx::render_llm("verify-release", cli_std::llm::Format::Json).unwrap();
    let release_value: Value = serde_json::from_str(&release).expect("release JSON parses");
    assert_eq!(release_value["task"]["risk"], "remote_write");
    assert!(release_value["task"]["reads"]
        .as_array()
        .unwrap()
        .iter()
        .any(|source| source == "apps/lumen/e2e/release_artifacts.rs"));
    for source in [
        ".github/workflows/lumen-release-candidate.yml",
        "apps/lumen/scripts/verify-release-candidate.sh",
        "apps/lumen/e2e/release_candidate.rs",
        ".github/workflows/lumen-release.yml",
        "apps/lumen/scripts/verify-release-artifacts.sh",
        "apps/lumen/e2e/release_promotion.rs",
        ".agents/skills/lumen-build-release/SKILL.md",
    ] {
        assert!(
            release_value["task"]["reads"]
                .as_array()
                .unwrap()
                .iter()
                .any(|got| got == source),
            "verify-release discovery omits {source}"
        );
    }
    assert!(release_value["task"]["reads"]
        .as_array()
        .unwrap()
        .iter()
        .any(|source| source == "apps/lumen/install.sh"));
    assert_eq!(release_value["providers"].as_array().unwrap().len(), 1);
    assert_eq!(release_value["providers"][0]["id"], "raft-runtime");
    let raft_provider = raft_runtime::llm::topic();
    assert_eq!(
        release_value["providers"][0]["markdown"],
        raft_provider.body
    );
    let release_markdown = release_value["markdown"].as_str().unwrap();
    for needle in [
        "Follow the exact release order",
        "release_candidate.rs",
        "verify-release-candidate.sh",
        "release_promotion.rs",
        "verify-release-artifacts.sh",
        "git land main",
        "one protected annotated tag",
        "candidate_run_id",
        "public verifier",
        "tracker closure",
        "target tag",
        "refs/tags/lumen@*",
        "update",
        "deletion",
        "no bypass actors",
        "must not rebuild",
        "re-sign",
        "re-attest",
        "never move latest backward",
        "do not create a stable Git tag",
        "semver/latest image tag",
        "Run kind before public promotion",
        "Land each release to main exactly once before public promotion",
        "GHCR root and platform digests",
        "downloaded host archive passes its checksum",
        "scripts/raft-implementor-build.sh",
    ] {
        assert!(
            release_markdown.contains(needle),
            "release topic missing `{needle}`: {release_markdown}"
        );
    }

    let local = dx::render_llm("run-standalone", cli_std::llm::Format::Json).unwrap();
    let local_value: Value = serde_json::from_str(&local).expect("standalone JSON parses");
    assert!(local_value.get("providers").is_none());
    assert_eq!(
        local_value["runbook"]["steps"][0]["command_template"],
        "lumen serve --host 127.0.0.1 --data-dir {data_dir} --wal embedded --persistence segment"
    );
    for needle in [
        "bare binary defaults to 127.0.0.1",
        "in-memory ephemeral storage",
        "container listens on 0.0.0.0",
        "No mount and no data directory means replacement loses state",
        "LUMEN_WAL=embedded",
        "LUMEN_DATA_DIR=/var/lib/lumen/data",
        "LUMEN_PERSISTENCE=segment",
        "Stop the old container cleanly",
        "declares no Docker VOLUME",
        "Auth off is safe only behind a trusted local network boundary",
        "not Managed, Fleet, or a production HA claim",
    ] {
        assert!(
            local_value["markdown"].as_str().unwrap().contains(needle),
            "standalone topic missing `{needle}`: {local}"
        );
    }
}

#[test]
fn dx_querying_topic_separates_current_and_target_contracts() {
    let markdown = dx::render_llm("querying", cli_std::llm::Format::Md).unwrap();
    for needle in [
        "current query API",
        "0.5 target",
        "facets, metrics, strict result controls, and capability activation are not current",
        "caller owns CDC, freshness, and source-record hydration",
        "Do not send 0.5 request fields",
    ] {
        assert!(
            markdown.contains(needle),
            "querying topic missing `{needle}`:\n{markdown}"
        );
    }

    let rendered = dx::render_llm("querying", cli_std::llm::Format::Json).unwrap();
    let value: Value = serde_json::from_str(&rendered).expect("querying JSON parses");
    assert_eq!(value["protocol"], "cclab.llm.v2");
    assert_eq!(value["topic"], "querying");
    assert_eq!(value["task"]["risk"], "inspect");
    assert_eq!(
        value["task"]["reads"],
        json!([
            "lumen spec --fields",
            "lumen spec --shapes",
            "apps/lumen/docs/querying.md",
            "apps/lumen/STATUS.md"
        ])
    );
    assert_eq!(value["markdown"], markdown);
}

/// #1480 R2: the reshard admin verbs section must cover all six
/// `Role::Admin`-gated verbs, including `POST /admin/reshard:fence`'s TTL
/// semantics, driver-owned framing, and manual-use risk warning.
#[test]
fn llm_storage_documents_reshard_fence_admin_verb() {
    let storage = llm_storage_md();
    for needle in [
        "Six more",
        "POST /admin/reshard:fence",
        "arms or clears a bounded write",
        "ttl_secs",
        "defaults to 300",
        "capped at 3600",
        "400 invalid_ttl_secs",
        "503 bucket_write_paused",
        "driver-owned",
        "advance_catching_up",
        "WRITE_FENCE_TTL_SECS",
        "risks a real write outage",
        "These six verbs",
    ] {
        assert!(storage.contains(needle), "storage topic missing `{needle}`");
    }
}

/// #1480 R3: the workflow topic must disclose the routed multi-shard client
/// retry contract — the three retryable `503` codes and the two rejected
/// (not retryable) verbs with their alternatives.
#[test]
fn llm_workflow_discloses_routed_mode_retry_contract() {
    let g = llm_workflow_md();
    for needle in [
        "Routed multi-shard mode: client retry contract",
        "bucket_write_paused",
        "shard_forward_unavailable",
        "shard_map_version_mismatch",
        "safe to retry with backoff",
        "duplicates_not_routed",
        "Do not retry.",
        "reindex_stream_not_routed",
        "POST /collections/{id}/index",
        "Do not retry the stream endpoint",
    ] {
        assert!(g.contains(needle), "workflow topic missing `{needle}`");
    }
}

/// #1480 R4: fold #1467's reshard/convergence observability additions
/// (the `lumen_shard_map_version` gauge, the
/// `lumen_scatter_map_version_mismatches_total` counter, and the
/// `awaitingTopologyConvergence`/`topologyConvergenceStalled` status
/// conditions) into the deployment topic.
#[test]
fn llm_deployment_documents_reshard_convergence_observability() {
    let deployment = llm_deployment_md();
    for needle in [
        "Reshard/convergence observability",
        "lumen_shard_map_version",
        "gauge",
        "lumen_scatter_map_version_mismatches_total",
        "counter",
        "awaitingTopologyConvergence",
        "topologyConvergenceStalled",
        "CONVERGENCE_STALL_TICKS",
    ] {
        assert!(
            deployment.contains(needle),
            "deployment topic missing `{needle}`"
        );
    }
}

/// #3113 R5: every language `lumen spec gen` emits configures a private trust
/// anchor and the name it expects the server to assert.
///
/// Asserted against the client this repository actually ships — generated from
/// lumen's own committed OpenAPI, through the same target policy `lumen spec
/// gen` uses — rather than against the emitters' own fixtures, so a generator
/// change that lands the seam for a toy spec but not for lumen's is caught here.
#[test]
fn every_generated_client_takes_a_private_ca_and_the_name_it_verifies() {
    use openapi_codegen::{generate_for_target, GenOptions, HttpClient, Lang, TargetPolicy};

    const TARGET_POLICY: &str = include_str!("../clients/codegen.toml");
    let policy = TargetPolicy::from_toml(TARGET_POLICY).expect("client target policy");

    // (language, entrypoint carrying the seam, what the seam has to say)
    let expected: [(Lang, &str, &[&str]); 3] = [
        (
            Lang::Rust,
            "client.rs",
            &[
                "pub struct PrivateTrust {",
                "pub ca_bundle: std::path::PathBuf,",
                "pub server_name: String,",
                ".tls_built_in_root_certs(false)",
                "if addressed != trust.server_name {",
            ],
        ),
        (
            Lang::Py,
            "client.py",
            &[
                "class PrivateTrust:",
                "ca_bundle: str",
                "server_name: str",
                "ssl.create_default_context(cafile=trust.ca_bundle)",
                "if addressed != trust.server_name:",
            ],
        ),
        (
            Lang::Ts,
            "runtime.ts",
            &[
                "export interface PrivateTrust {",
                "caBundle: string;",
                "serverName: string;",
                "trust?: PrivateTrust;",
                "if (addressed !== trust.serverName) {",
            ],
        ),
    ];

    for (lang, entry, needles) in expected {
        let target = policy.resolve(lang, None).expect("pinned target");
        let opts = GenOptions {
            lang,
            target: Some(target),
            spec_path: std::path::PathBuf::new(),
            out_dir: std::path::PathBuf::new(),
            client_name: "createClient".to_string(),
            http_client: HttpClient::Fetch,
            emit_types: true,
            emit_client: true,
            emit_hooks: matches!(lang, Lang::Ts),
        };
        let out = generate_for_target(&openapi_json(), &opts, target).expect("generate client");
        let file = out
            .files
            .iter()
            .find(|f| f.rel_path == entry)
            .unwrap_or_else(|| panic!("{lang:?} client emits {entry}"));
        for needle in needles {
            assert!(
                file.contents.contains(needle),
                "{entry} has no private-CA seam ({needle}): an in-cluster caller would have \
                 to trust the public roots for a name no public CA can vouch for"
            );
        }
        // The anchor is the fix for an unverifiable server; skipping the check
        // is not, so no generated client may offer it.
        for weakening in WEAKENINGS {
            for generated in &out.files {
                assert!(
                    !generated.contents.contains(weakening),
                    "{} offers {weakening}; verification is never the thing to turn off",
                    generated.rel_path
                );
            }
        }
    }
}

/// Ways a client could be talked into not checking who it is talking to. None of
/// them may appear in generated output.
const WEAKENINGS: [&str; 6] = [
    "danger_accept_invalid_certs",
    "danger_accept_invalid_hostnames",
    "check_hostname = False",
    "CERT_NONE",
    "rejectUnauthorized: false",
    "NODE_TLS_REJECT_UNAUTHORIZED",
];

#[test]
fn operator_tls_ownership_is_consistently_documented_across_surfaces() {
    let deployment_md = llm_deployment_md();
    // README carries the capability contract; there is no separate
    // CAPABILITIES.md to read, so the two canonical surfaces are these.
    let readme = std::fs::read_to_string("README.md").expect("read README.md");

    // Both canonical surfaces describe externally provisioned TLS and no
    // longer teach the retired operator issuer/CAS/controller path.
    for doc in [&deployment_md, &readme] {
        assert!(
            (doc.contains("servingTlsSecret") && doc.contains("peerTlsSecret"))
                || (doc.contains("serving") && doc.contains("peer") && doc.contains("TLS Secrets")),
            "doc missing externally provisioned serving/peer TLS Secret boundary: {doc}"
        );
        for retired in [
            "--issuer cas",
            "--issuer ephemeral",
            "--trust-domain",
            "--ca-pool",
            "certificate_controller",
            "--workload-identity-audience",
            "--projected-token-path",
            "LUMEN_WORKLOAD_IDENTITY_AUDIENCE",
            "LUMEN_PROJECTED_TOKEN_PATH",
        ] {
            assert!(
                !doc.contains(retired),
                "doc contains retired direct-STS name '{retired}': {doc}"
            );
        }
    }
}
// CODEGEN-END
