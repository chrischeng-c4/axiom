---
id: projects-lumen-src-spec-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized apps/lumen/src/spec.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/lumen/src/spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `field_catalog` | apps/lumen/src/spec.rs | function | pub | 265 | field_catalog() -> Value |
| `json_schema_json` | apps/lumen/src/spec.rs | function | pub | 47 | json_schema_json() -> String |
| `llm_auth_md` | apps/lumen/src/spec.rs | function | pub | 448 | llm_auth_md() -> String |
| `llm_deployment_md` | apps/lumen/src/spec.rs | function | pub | 313 | llm_deployment_md() -> String |
| `llm_integration_md` | apps/lumen/src/spec.rs | function | pub | 784 | llm_integration_md() -> String |
| `llm_outline_md` | apps/lumen/src/spec.rs | function | pub | 273 | llm_outline_md() -> String |
| `llm_quickstart_md` | apps/lumen/src/spec.rs | function | pub | 824 | llm_quickstart_md() -> String |
| `llm_recipes_md` | apps/lumen/src/spec.rs | function | pub | 899 | llm_recipes_md() -> String |
| `llm_storage_md` | apps/lumen/src/spec.rs | function | pub | 925 | llm_storage_md() -> String |
| `llm_workflow_md` | apps/lumen/src/spec.rs | function | pub | 540 | llm_workflow_md() -> String |
| `openapi_json` | apps/lumen/src/spec.rs | function | pub | 16 | openapi_json() -> String |
| `openapi_yaml` | apps/lumen/src/spec.rs | function | pub | 22 | openapi_yaml() -> String |
| `query_shapes` | apps/lumen/src/spec.rs | function | pub | 178 | query_shapes() -> Value |
| `token_registry_example_json` | apps/lumen/src/spec.rs | function | pub | 147 | token_registry_example_json() -> String |
| `token_registry_schema` | apps/lumen/src/spec.rs | function | pub | 62 | token_registry_schema() -> Value |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Offline, machine-readable self-description for agent integration.
//!
//! The `lumen spec` CLI subset emits everything an LLM agent needs to wire
//! lumen into a RAG / tool pipeline — schema, query-shape cookbook, field /
//! analyzer catalog — straight from the installed binary, with no running
//! server and no network. This module is the single source for that surface;
//! the CLI and the (legacy) `lumen-openapi-dump` binary both call into it.

use serde_json::{json, Value};

/// The full OpenAPI 3.2 document as pretty JSON (every route + schema,
/// including the #1297 `QUERY` twins injected by `crate::api::openapi`).
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn openapi_json() -> String {
    serde_json::to_string_pretty(&openapi_value()).expect("OpenApi value serializes to JSON")
}

/// The full OpenAPI 3.2 document as YAML for LLM/agent reading.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn openapi_yaml() -> String {
    serde_yaml::to_string(&openapi_value()).expect("OpenApi value serializes to YAML")
}

/// `crate::api::openapi()` as a JSON [`Value`] stamped as OpenAPI 3.2 (#1298,
/// epic #1296): utoipa 4.2.3's `OpenApiVersion` enum predates OpenAPI 3.2 and
/// only knows how to serialize the literal `"3.0.3"`, so the typed
/// `utoipa::openapi::OpenApi` — and the live `GET /openapi.json` route that
/// serves it verbatim via `service_http::standard_probe_routes` — keeps
/// declaring 3.0.3. This offline `lumen spec` surface (and the
/// `clients/openapi.json` contract file regenerated from it) is not bound by
/// that typed field, so it stamps the real document version here; the
/// `query`/`x-post-twin` operations are unaffected either way since they are
/// injected upstream in `crate::api::openapi`.
fn openapi_value() -> Value {
    let mut v = serde_json::to_value(crate::api::openapi()).expect("OpenApi serializes to JSON");
    if let Value::Object(map) = &mut v {
        map.insert("openapi".to_string(), Value::String("3.2.0".to_string()));
    }
    v
}

/// Just the component schemas (the request/response data types) as pretty JSON
/// — the JSON-Schema view an agent uses to build/validate request bodies.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn json_schema_json() -> String {
    let api = crate::api::openapi();
    // #2871 retired the bearer/identity registry, so `operationalSchemas` no
    // longer carries a `TokenRegistry` entry: nothing reads that file, and a
    // published schema for it would read as a supported deployment shape.
    serde_json::to_string_pretty(&json!({ "components": api.components }))
        .expect("components serialize to JSON")
}

/// A cookbook of canonical query shapes. Each entry is a ready-to-POST
/// `{name, description, request}` for `POST /collections/{id}/search` (or
/// `/duplicates` where noted) using the exact wire form of every `QueryNode`
/// variant plus sort / collapse.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn query_shapes() -> Value {
    json!({
        "search_endpoint": "POST /collections/{collection}/search",
        "search_all_endpoint": "POST /collections/{collection}/search:all",
        "note": "lumen returns ranked/sorted external_id hits only — never documents.",
        "shapes": [
            { "name": "term", "description": "exact keyword/number/bool match",
              "request": { "query": { "term": { "field": "status", "value": "active" } }, "limit": 20 } },
            { "name": "terms", "description": "keyword in a set (IN)",
              "request": { "query": { "terms": { "field": "status", "values": ["active", "trial"] } }, "limit": 20 } },
            { "name": "prefix", "description": "case-sensitive UTF-8 starts-with match on a keyword field",
              "request": { "query": { "prefix": { "field": "path", "value": "台北市/" } }, "limit": 20 } },
            { "name": "ids", "description": "filter by a set of external_ids (row_id_in); unknown ids skipped",
              "request": { "query": { "ids": { "values": ["row-42", "row-91"] } }, "limit": 20 } },
            { "name": "range", "description": "numeric range on a `number` field (e.g. 1000 <= price < 5000)",
              "request": { "query": { "range": { "field": "price", "gte": 1000, "lt": 5000 } }, "limit": 20 } },
            { "name": "range_keyword", "description": "byte/lexicographic range on a `keyword` field — string bounds are valid only against `keyword` fields (rejected with 400 against `number` or `text`), and compare the same way ISO-8601 date/datetime strings sort chronologically",
              "request": { "query": { "range": { "field": "created_at", "gte": "2026-01-01", "lt": "2026-02-01" } }, "limit": 20 } },
            { "name": "match_bm25", "description": "lexical BM25 ranking over a text field",
              "request": { "query": { "match": { "field": "bio", "text": "rust search engineer" } }, "limit": 20 } },
            { "name": "autocomplete_ngram", "description": "autocomplete/suggest recipe: declare a text field with analyzer=ngram, index the searchable label, then run match on the prefix/substring; lumen returns external_ids, not suggestion payloads",
              "request": { "query": { "match": { "field": "title_suggest", "text": "wire" } }, "limit": 10 } },
            { "name": "boolean_and", "description": "conjunction; planner drives from the most selective clause",
              "request": { "query": { "and": [
                  { "match": { "field": "name", "text": "手機殼" } },
                  { "range": { "field": "price", "gte": 1000, "lt": 5000 } }
              ] }, "limit": 20 } },
            { "name": "boolean_or", "description": "disjunction",
              "request": { "query": { "or": [
                  { "term": { "field": "brand", "value": "apple" } },
                  { "term": { "field": "brand", "value": "samsung" } }
              ] }, "limit": 20 } },
            { "name": "boolean_not", "description": "AND with a negated filter clause",
              "request": { "query": { "and": [
                  { "term": { "field": "category", "value": "phone" } },
                  { "not": { "term": { "field": "refurbished", "value": "true" } } }
              ] }, "limit": 20 } },
            { "name": "knn", "description": "vector kNN (caller supplies the embedding)",
              "request": { "query": { "knn": { "field": "embedding", "vector": [0.12, -0.03, 0.88], "k": 10 } }, "limit": 10 } },
            { "name": "rrf_hybrid", "description": "hybrid lexical+semantic: fuse a BM25 match and a vector kNN by rank (Reciprocal Rank Fusion)",
              "request": { "query": { "rrf": { "k": 60, "queries": [
                  { "match": { "field": "title", "text": "wireless earbuds" } },
                  { "knn": { "field": "embedding", "vector": [0.12, -0.03, 0.88], "k": 50 } }
              ] } }, "limit": 10 } },
            { "name": "rrf_hybrid_filtered", "description": "filter-correct hybrid: put the filter INSIDE each leg so the kNN leg stays filter-correct (no recall collapse)",
              "request": { "query": { "rrf": { "k": 60, "queries": [
                  { "and": [ { "match": { "field": "title", "text": "wireless earbuds" } }, { "term": { "field": "brand", "value": "acme" } } ] },
                  { "and": [ { "knn": { "field": "embedding", "vector": [0.12, -0.03, 0.88], "k": 50 } }, { "term": { "field": "brand", "value": "acme" } } ] }
              ] } }, "limit": 10 } },
            { "name": "hamming_near_dup", "description": "perceptual near-duplicate: hashes within N Hamming bits",
              "request": { "query": { "hamming": { "field": "phash", "hash": "f0e1d2c3b4a59687", "max_distance": 8 } }, "limit": 20 } },
            { "name": "has_child_nested_group", "description": "rows whose nested group has an element matching a sub-query; may be combined with parent-field sort",
              "request": { "query": { "has_child": {
                  "collection": "orders_items", "field": "parent_row_id",
                  "query": { "and": [
                      { "term": { "field": "sku", "value": "S0" } },
                      { "range": { "field": "qty", "gte": 5 } }
                  ] } } },
                  "sort": [ { "field": "score", "order": "asc" } ],
                  "track_total": true,
                  "limit": 20 } },
            { "name": "collapse_group_by", "description": "one hit per distinct keyword value (group-by), scored by the max member",
              "request": { "query": { "term": { "field": "in_stock", "value": "true" } }, "collapse": "brand", "limit": 20 } },
            { "name": "filter_then_sort", "description": "filter, then sort by a field instead of relevance",
              "request": { "query": { "range": { "field": "price", "gte": 100 } },
                           "sort": [ { "field": "price", "order": "asc" } ], "track_total": false, "limit": 20 } },
            { "name": "native_offset", "description": "jump directly after global filtering and ordering without client over-fetch; offset and cursor are mutually exclusive",
              "request": { "query": { "range": { "field": "price", "gte": 100 } },
                           "sort": [ { "field": "price", "order": "asc" } ], "offset": 1000, "limit": 20 } },
            { "name": "search_all", "description": "explicitly expensive full materialization of every matching external_id (POST /collections/{id}/search:all); local consistency is one read-lock snapshot, routed consistency is one snapshot per shard",
              "request": { "query": { "term": { "field": "status", "value": "active" } },
                           "sort": [ { "field": "created_at", "order": "asc" } ] } },
            { "name": "duplicates", "description": "find external_ids sharing a value (POST /collections/{id}/duplicates)",
              "request": { "field": "email", "min_group_size": 2, "limit": 100 } },
            { "name": "index", "description": "index one or more field values (POST /collections/{id}/index); the wire shape is FLAT — {items:[{external_id,field,value}]} — not the nested {id, fields:{...}} shape a caller might assume",
              "request": { "items": [
                  { "external_id": "row-42", "field": "email", "value": "person@example.com" },
                  { "external_id": "row-42", "field": "price", "value": 79 }
              ] } }
        ]
    })
}

/// The field-type + analyzer + vector-metric catalog — what `type`/`analyzer`/
/// `metric` values a `PUT /collections/{id}` schema may use. Mirrors the
/// `FieldType` / `Analyzer` / `VectorMetric` enums.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn field_catalog() -> Value {
    crate::dx::field_catalog()
}

/// The agent-facing LLM topic outline (`lumen llm --topic outline`) as
/// Markdown.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
/// @spec apps/lumen/tech-design/interfaces/cli/self-docs-teach-positional-lumen-llm-topic-but-the-cli-only-acce.md#logic
pub fn llm_outline_md() -> String {
    r#"# lumen LLM outline

Use the smallest topic that answers the task:

- `lumen llm --topic workflow` — product model, declare→ingest→search→hydrate, query
  flavor choices, batch search (`POST /collections:search`), full-replacement
  writes (`PUT /collections/{id}/docs:replace`), QUERY-first search
  (RFC 10008 `QUERY /collections/{id}` and `QUERY /collections`, POST always
  available), connection, and non-goals.
- `lumen llm --topic integration` — recommended Postgres/AlloyDB adapter boundary:
  outbox or CDC, external Pub/Sub retry/DLQ ownership, HTTP writes into lumen,
  and no direct external writes to lumen's internal WAL.
- `lumen llm --topic quickstart` — copy-paste local create → index → search flow.
- `lumen llm --topic auth` — request-authentication contract: private ClusterIP
  TLS and a short-lived Kubernetes ServiceAccount token as two independent
  checks, `required` vs `disabled`, and the TokenReview/SubjectAccessReview
  model that replaced the retired bearer/Google registry.
- `lumen llm --topic deployment` — Kubernetes-native deployment topology:
  StatefulSet, shardCount, replicasPerShard, HPA boundary, reshard workflow,
  and empty-PVC bootstrap.
- `lumen llm --topic storage` — operator storage/ops contract: the serving fleet is
  always a StatefulSet with a durable PVC-backed WAL, including at
  `replicasPerShard: 1`.
- `lumen llm --topic recipes` — task → ready-to-POST query bodies.
- `lumen spec --format openapi-yaml` — OpenAPI YAML for LLM/agent reading.
- `lumen spec` — OpenAPI JSON, JSON-schema, query-shape, field, analyzer, and
  vector metric catalogs.
- `lumen connect` — manage a `kubectl port-forward` for the duration of a
  wrapped command against a k8s-deployed Lumen instance (`--cr`/`--service` +
  `--namespace`), tearing it down when the command exits. Against an
  `auth: required` fleet, add `--client-sa` to mint a short-lived
  audience-bound token and `--ca-file` to verify the server against the
  externally distributed public CA; the token is attached by a loopback proxy, so it
  reaches no environment variable and no child process.
- `lumen query index|search|duplicates|collections list` — one-shot query
  wrappers against a reachable node (`--url`/`LUMEN_URL`); request bodies match
  `lumen spec --shapes`. There is no credential flag by design — run them under
  `lumen connect`, which owns minting and attaching. See `--topic auth`.
"#
    .to_string()
}

/// Kubernetes-native deployment topology (`lumen llm --topic deployment`) as
/// Markdown.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn llm_deployment_md() -> String {
    let mut out = r#"# lumen deployment

## Artifact layers
Use the layered service CLI surface so image, cluster API, operator, and
instance ownership stay separate:

```
lumen dockerfile render --variant release --version lumen@<version> --out Dockerfile
lumen k8s crd render --out lumen-crd.yaml
lumen k8s operator render --namespace lumen-system --out operator/
lumen k8s instance render --profile prod --out lumen.yaml
```

`dockerfile render` is intentionally outside `k8s`: compose, kind, and
registries all consume the same image artifact. `k8s crd render` is the
cluster-scoped API layer, `k8s operator render|run` is the control plane, and
`k8s instance render` is the app-namespace custom resource.

## Externally Provisioned TLS Secrets
Deployment administrators or an external platform provision the serving and peer TLS Secrets named by each Lumen instance. The operator only consumes those Secrets and does not resolve issuers or perform CAS automation.

The deployment administrator or an external platform provisions the named
`servingTlsSecret` and `peerTlsSecret` in the instance namespace. The operator
consumes those Secrets through the existing TLS loaders and never resolves an
issuer, contacts CAS, selects a trust domain, or obtains certificate tokens.

## Serving transport: private ClusterIP TLS
Production traffic is **not** published. A Lumen instance is reached at its
Service DNS name inside the cluster and nowhere else:

```
LUMEN_URL=https://<instance>.<namespace>.svc:7373
```

There is no Ingress, no Gateway, no LoadBalancer, no NodePort, and no service
mesh terminating TLS on lumen's behalf. The serving pod holds the private key
itself, so the connection a caller authenticates is the connection lumen
serves. An edge that terminated TLS and re-originated plaintext would leave the
last hop unauthenticated while every client-side check still passed — and the
KSA token in `Authorization` would cross that hop in the clear.

Set `spec.servingTlsSecret` to a Secret holding `tls.crt`, `tls.key`, and
`ca.crt`. The operator projects it into every serving pod and switches the
client port from h2c to TLS with ALPN `h2, http/1.1`:

```env
LUMEN_TLS=on
LUMEN_TLS_CERT=/var/run/secrets/lumen-serving/tls.crt
LUMEN_TLS_KEY=/var/run/secrets/lumen-serving/tls.key
LUMEN_TLS_CA=/var/run/secrets/lumen-serving/ca.crt
LUMEN_TLS_SERVER_NAMES=<instance>.<namespace>.svc,<instance>.<namespace>.svc.cluster.local
```

The leaf asserts the Service's own two DNS spellings and nothing else. A name
in the certificate is a name this instance can impersonate, so no node name and
no external name belongs there. While no valid leaf is active the port refuses
connections rather than falling back to plaintext.

Callers verify against the anchor alone. The deployment administrator or
external certificate platform distributes the public CA separately from the
private-key-bearing serving Secret. Pass it to `lumen connect --ca-file`, or
as `PrivateTrust` in a generated client; it replaces the public roots rather
than joining them.

`spec.peerTlsSecret` is a separate field for a separate decision — mutual,
instance-scoped Raft identity on `:7374`. Sharing one Secret between the two
would let either listener's material authenticate on the other's port.

Omit `spec.servingTlsSecret` only for local and kind development, where the
client port stays h2c and `spec.auth` is `disabled`.

## Storage topology knobs
The operator-owned storage topology has two independent knobs:

- `spec.shardCount`: the number of physical storage shards that own the corpus.
- `spec.replicasPerShard`: how many pods belong to each shard group.

The serving StatefulSet replica count is always:

```
totalPods = shardCount * replicasPerShard
```

Pod ordinals map deterministically to topology slots:

```
shardIndex = ordinal % shardCount
replicaIndex = ordinal / shardCount
```

That means `shardCount = 3, replicasPerShard = 2` creates six pods: shard 0
has ordinals 0 and 3, shard 1 has ordinals 1 and 4, and shard 2 has ordinals
2 and 5.

## Replica modes
- `replicasPerShard: 1`: one durable member per shard. It uses the local WAL,
  no raft consensus, and is the simplest topology for dev, small prod, or
  sharded-but-not-HA deployments. It is not a primary/follower replication
  mode: there is no background follower catching up from a primary.
- `replicasPerShard: 2`: failover-oriented shape. It adds a second member per
  shard; use it only when the operator/raft policy for the environment is
  intentionally configured for that failover mode.
- `replicasPerShard: 3`: normal raft quorum shape. Set `voterCount: 3` for a
  three-voter shard group.

`voterCount` is per shard group, not cluster-wide. Extra replicas beyond
`voterCount` are learners.

## HPA boundary
HPA is for stateless or near-stateless serving capacity, not for changing
storage ownership. Do not use HPA to change `shardCount` or to add/remove raft
members. Lumen attaches HPA only where the rendered topology can tolerate it;
raft-HA shard groups use a fixed `shardCount * replicasPerShard` peer set.
HPA-created pods in a single-member topology must not be treated as synced data
replicas; production data fan-out is `shardCount`, and production HA is
`replicasPerShard > 1` raft.

## Dynamic shard growth
Shard growth is an operator workflow, not a direct response to request load.
The normal trigger is storage pressure: for example, prepare a split around
50% of the configured shard ceiling, then move virtual buckets in bounded
snapshot batches. The versioned virtual-bucket map decides ownership:

```
bucket = hash(collection_id, routing_key || external_id) % virtualBucketCount
```

Search without a routing key scatters/gathers across shards. Search with a
routing key can target the owning shard. Do not auto-split when the max shard
size or max shard count is unknown; surface the condition to the operator
instead.

## Reshard/convergence observability (#1467)
Beyond the `blockingConditions`/`message` fields on `status.reshard` covered
above, watch these signals during and after a split:

- `lumen_shard_map_version` (gauge) — each serving pod's own live routed
  shard-map version, `0` outside routed deployments. The reshard driver's
  `advance_convergence` scrapes this over every serving pod's `/metrics`
  (the same admin-reachable surface its usage-polling loop already uses) to
  require every pod to actually report the new map version before clearing
  the post-cutover write-pause fence — a rollout that completed but whose
  ConfigMap write has not yet propagated to every pod is not treated as
  converged.
- `lumen_scatter_map_version_mismatches_total` (counter) — count of scatter
  (routing-key-less) search sub-requests where the responding pod's live
  shard-map version disagreed with the scattering pod's own declared
  version. This is an expected, non-fatal signal during a mixed-map rolling
  restart window, not an error: lumen accepts availability over
  completeness for a scatter search mid-flight during a rollout rather than
  fail the whole search. A sustained non-zero rate outside an active reshard
  points at a stuck rollout.
- `awaitingTopologyConvergence` (`status.reshard.blockingConditions`) — the
  post-cutover write-pause fence is armed and waiting for every serving pod
  to confirm the new `shardMap.version`; expected and bounded during a
  cutover, not itself an error.
- `topologyConvergenceStalled` (`status.reshard.blockingConditions`) —
  layered on top of `awaitingTopologyConvergence` once convergence has been
  pending for an extended, bounded number of driver ticks
  (`CONVERGENCE_STALL_TICKS`) without clearing. The write-pause fence is
  never silently dropped when this budget is exceeded — the driver keeps
  re-arming it — so this condition is the operator-visible signal that the
  wait is abnormally long and needs investigation, not proof writes have
  resumed.

## Empty-PVC bootstrap
Existing pods restart from their PVC: local raft state, snapshots, and WAL are
authoritative. A replacement pod with an empty PVC should seed from an exact
`SnapshotV1` object first, then catch up the WAL/raft delta:

```
LUMEN_BOOTSTRAP_SEED_URI=file:///snapshots/shard-0.json
LUMEN_BOOTSTRAP_SEED_URI=s3://bucket/path/shard-0.json
```

Backup is the cold disaster-recovery and seed surface; it is not the normal
live replica synchronization mechanism.
"#
    .to_string();
    out.push_str("\n## Shared raft-runtime topology primitive\n");
    out.push_str(raft_runtime::llm::topic().body);
    out
}

/// Request-authentication contract (`lumen llm --topic auth`) as Markdown.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn llm_auth_md() -> String {
    let mut out = String::from(
        r#"# lumen auth

## Runtime contract
Two independent checks stand between a caller and a collection, and neither
substitutes for the other:

- **Transport.** In production the serving port is private TLS on a ClusterIP
  Service, signed by a private CA. A caller verifies the server; the server does
  not verify the caller's certificate.
- **Request identity.** A short-lived, audience-bound Kubernetes ServiceAccount
  token in `Authorization: Bearer`, answered by the cluster.

Server modes:

```env
LUMEN_AUTH=required        # production: every request is TokenReview'd
LUMEN_AUTH=disabled        # local and kind only: every request resolves to open
```

`LUMEN_AUTH=required` proves both delegation grants at startup — that it can
reach kube-apiserver, and that its own ServiceAccount may create `TokenReview`
and `SubjectAccessReview` in this namespace. If either fails the process **does
not start**, naming the missing `system:auth-delegator` binding. Discovering it
on the first request would mean serving 503s while looking healthy.

There is no third mode and no degradation: a `required` process that loses its
verifier rejects requests rather than falling back to open.

Probe/spec/scrape routes stay auth-exempt regardless: `/healthz`, `/readyz`,
`/metrics`, `/openapi.json`, and `/docs`.

## Transport: private ClusterIP TLS, terminated by lumen
Production traffic is **not** published. There is no Ingress, no Gateway, no
LoadBalancer, no NodePort, and no service mesh terminating TLS on lumen's
behalf. The listener holds the private key itself, so the connection a caller
authenticates is the connection lumen serves — an edge that terminates TLS and
re-originates plaintext would leave the last hop unauthenticated while the
client's own check still passed.

```env
LUMEN_URL=https://<instance>.<namespace>.svc:7373
```

Set `spec.servingTlsSecret` on the `Lumen` CR and the operator projects the
leaf, key, and CA into the pods and turns the listener on:

```env
LUMEN_TLS=on
LUMEN_TLS_CERT=/var/run/secrets/lumen-serving/tls.crt
LUMEN_TLS_KEY=/var/run/secrets/lumen-serving/tls.key
LUMEN_TLS_CA=/var/run/secrets/lumen-serving/ca.crt
LUMEN_TLS_SERVER_NAMES=<instance>.<namespace>.svc,<instance>.<namespace>.svc.cluster.local
```

The names in `LUMEN_TLS_SERVER_NAMES` are the Service's own two spellings and
nothing else. No node name, no external name: a name in the leaf is a name this
instance can impersonate.

Callers verify against the public CA distributed separately by the deployment
administrator or external certificate platform. Pass that PEM file with
`lumen connect --ca-file`; it replaces the public roots and is never read from
the private-key-bearing serving Secret.

Peer traffic on `:7374` is a separate trust decision with its own material
(`spec.peerTlsSecret`) — mutual, instance-scoped, and never interchangeable
with the serving anchor.

## Kubernetes: who a caller is, is the cluster's answer
The CRD configures **no** credential source. `spec.tokensSecret`,
`spec.identities` and `spec.identityAudiences` are gone (#2872): a Lumen CR
that sets any of them is rejected by the API server's strict decoding rather
than applied and quietly ignored.

Every request under `auth: required` carries a short-lived, audience-bound
ServiceAccount token obtained from the TokenRequest API, and lumen answers two
questions with the cluster rather than with a file it was handed:

- **Who is this?** `TokenReview`, with the audience `lumen.axiom.dev` checked.
  The principal it returns must be exactly
  `system:serviceaccount:<namespace>:<name>`; nothing else is accepted as an
  identity.
- **May they do this?** `SubjectAccessReview` against the virtual resources
  `lumencollections` and `lumenadmin` in API group `lumen.axiom.dev`, scoped to
  the serving instance's own namespace. RBAC in the cluster is the
  authorization source of truth, so a grant is a Role a reviewer can read,
  `kubectl auth can-i` can answer, and the audit log records.

| Action | Resource | Name | Verb |
|--------|----------|------|------|
| read a collection | `lumencollections` | collection id | `get` |
| write a collection | `lumencollections` | collection id | `update` |
| administer a collection | `lumencollections` | collection id | `delete` |
| instance admin | `lumenadmin` | — | per role |

There is no role precedence. `delete` does not imply `get` unless the
RoleBinding says so.

Nothing is a long-lived secret: tokens are minted per use and expire, so there
is no registry to rotate, no Secret to sync from Secret Manager, and no CSI
mount whose refresh behaviour has to be reasoned about. Google user and GSA
credentials authenticate to the kube-apiserver only — lumen never sees one, and
never verifies a Google-issued token itself.

## The CLI: kubeconfig authenticates you to Kubernetes, not to Lumen
Two boundaries, two credentials, and they are never the same string.

- **To the cluster:** your kubeconfig — a Google user, a GSA, or the GKE
  credential plugin. This is what `kubectl` and `lumen connect` already use.
- **To Lumen:** a short-lived, audience-bound ServiceAccount token that the
  cluster mints. Never a Google access token, a Google ID token, an ADC
  credential, a GSA key, or a metadata-server token. Those authenticate to
  kube-apiserver and stop there; sent to Lumen they are rejected, and submitted
  to `TokenReview` a Google ID token comes back `{"user": {}, "error":
  "invalid bearer token"}`.

The bridge between the two is the TokenRequest API: your kubeconfig identity is
RBAC-authorized to request a token for one explicitly named client
ServiceAccount, and that token — not your own credential — is what reaches
Lumen.

`lumen connect` does exactly that (#2878): it mints the token through your
kubeconfig, holds it in memory, and attaches it on a loopback proxy so the
child process is handed `LUMEN_URL` and never the token itself. There is no
token flag, no token environment variable, and no Kubernetes Secret standing
behind either.

The port-forward it opens is transport only. The upstream URL keeps the
Service's real DNS name, so SNI, hostname verification, and `:authority` all
target the name the certificate asserts while only address resolution points at
the forwarded loopback socket — `--ca-file` supplies the anchor:

```sh
lumen connect --namespace search --cr lumen \
  --client-sa agent --ca-file ./ca.crt -- ./my-app
```

Verification is never the thing to switch off. There is no insecure flag: a
server that cannot be verified is a wrong anchor or a wrong name, and both are
fixable.

## Generated clients
`lumen spec gen --lang rust|py|ts` emits a client that takes the same two
values — the CA bundle and the name it expects the server to assert — and
verifies against that anchor *instead of* the public roots:

```rust
Client::with_private_ca(
    "https://lumen.search.svc:7373",
    TransportPolicy::default(),
    PrivateTrust { ca_bundle: "/etc/lumen/ca.crt".into(), server_name: "lumen.search.svc".into() },
)?
```

```python
Client("https://lumen.search.svc:7373",
       trust=PrivateTrust(ca_bundle="/etc/lumen/ca.crt", server_name="lumen.search.svc"),
       auth_token=token)
```

```ts
const trust = { caBundle: "/etc/lumen/ca.crt", serverName: "lumen.search.svc" };
const config = { baseUrl: "https://lumen.search.svc:7373",
                 trust, fetch: await privateCaFetch(trust) };
```

Construction fails if `server_name` is not the host the base URL addresses: a
client that verified one name while addressing another would be checking a
certificate it never actually relies on. No generated client offers a way to
skip verification.

The KSA token travels as it always did — `auth_token="<token>"` or
`default_headers={"Authorization": "Bearer <token>"}` on `Client` and
`AsyncClient`.
"#,
    );
    out.push_str("\n## Shared auth primitive\n");
    out.push_str(service_auth::llm::topic().body);
    out
}

/// The agent workflow model (`lumen llm --topic workflow`) as Markdown — the mental
/// model, declare→ingest→search→hydrate workflow, search-flavor decision map,
/// connection, and non-goals. Where exact wire shape is needed it points at
/// `lumen spec` / `lumen llm --topic recipes` so there is one source of truth.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn llm_workflow_md() -> String {
    r#"# lumen workflow

## What lumen is
lumen is a **search index, not a database**. You (the caller) own the source of
truth — Postgres / AlloyDB / MongoDB / S3. lumen stores only index bits keyed by
your `external_id` and returns **ranked `external_id`s, never documents**. You
hydrate the hits against your own store.

## The integration loop (4 steps)
1. **Declare** a collection schema once — `PUT /collections/{id}` with a map of
   field name → typed field. The type fixes the index; there is no separate
   "index options" knob.
2. **Ingest** — your own pub/sub (CDC / logical replication / app writes) calls
   `POST /collections/{id}/index`. lumen bundles no connector; see
   `examples/consumer_pg_logical.py`. Re-writing `(external_id, field)` fully
   re-indexes that field.
3. **Search** — `POST /collections/{id}/search` with a query (relevance +
   filters + sort). You get back ranked `external_id`s + scores. Prefer the
   `QUERY /collections/{id}` twin when your stack supports it — see "QUERY
   method (RFC 10008)" below.
4. **Hydrate** — look the returned `external_id`s up in YOUR store to get the
   full records. lumen never had them.

## QUERY method (RFC 10008)
Policy: **QUERY-first, POST-always-available** (epic #1296 R1). `QUERY
/collections/{id}` and `QUERY /collections` are dual-registered twins of
`POST /collections/{id}/search` and `POST /collections:search`: same request
body (`SearchRequest` / `BatchSearchRequest`), same handler, and a
byte-identical response for identical bodies.

- Prefer `QUERY` when your HTTP client, proxy, and cache layer support it —
  RFC 10008 QUERY tells intermediaries the request is safe, idempotent, and
  cacheable, which `POST` cannot express.
- `POST` is the permanent fallback, not a deprecated path — every QUERY
  endpoint keeps its POST twin forever, so clients, proxies, and load
  balancers that can't emit `QUERY` (older HTTP libraries, some
  intermediaries) stay fully supported.
- `Content-Type: application/json` is mandatory on `QUERY` requests; a
  missing or mismatched `Content-Type` returns 415, same as the POST twin.
- `OPTIONS`/`HEAD` on both targets advertise `Accept-Query: application/json`
  and list `QUERY` in `Allow`.

## Batch search (multi-collection fan-out)
`POST /collections:search` is an msearch-style batch of independent
`(collection, SearchRequest)` items, executed with server-side concurrent
fan-out — use it instead of N client-side round-trips when a logical action
searches multiple collections at once (per-tenant/per-type partitioning,
for example). `collections:search` is one literal path segment (AIP-136
custom-method syntax), so it never collides with
`/collections/{collection_id}`; collection ids may not contain `:` for the
same reason.

```json
POST /collections:search
{ "searches": [
    { "collection": "users",    "query": {"term": {"field": "tags", "value": "rust"}}, "limit": 10 },
    { "collection": "products", "query": {"match": {"field": "title", "text": "earbuds"}}, "limit": 5 }
] }
→ 200 { "results": [
    { "status": "ok", "response": { "hits": [...], "total": 3, "took_ms": 1 } },
    { "status": "error", "code": "collection_not_found", "message": "..." }
] }
```

- Each item carries a full `SearchRequest` — `limit`, `sort`, `cursor`,
  `collapse`, `routing_key`, `track_total` may all differ per item, exactly
  like `POST /collections/{id}/search`.
- `results` is the same order and length as `searches`.
- **Partial failure never fails the batch.** One bad item (for example an
  unknown collection) reports `{"status":"error","code":"collection_not_found",
  "message":"..."}` for that item while the other items still return
  `{"status":"ok","response":{...}}`. The batch-level HTTP status stays 200
  unless the body is malformed or the batch is over the size limit (400).
- Max batch size is 32 items — this also bounds the concurrent fan-out. An
  over-limit batch is rejected with 400 before any item runs.
- Pagination stays per-item: each result's `cursor` continues independently
  by resubmitting that one item. There is no merged cursor and no
  cross-collection score merging/ranking — that is explicitly out of scope.

## Full-replacement writes (docs:replace)
`PUT /collections/{id}/docs:replace` is a batch **full-replacement** upsert:
each item's `fields` becomes the doc's *entire* indexed state — a declared
schema field the doc has today but that is absent from `fields` is
**implicitly deleted**. `docs:replace` is one literal path segment appended
after `{collection_id}`, so it registers directly in axum next to
`/collections/{collection_id}/docs/{external_id}` without any capture
ambiguity.

```json
PUT /collections/{id}/docs:replace
{ "docs": [
    { "external_id": "row-42", "version": 7, "fields": { "title": "New title", "state": "open" } }
] }
→ 200 { "results": [
    { "status": "ok", "fields_written": 2, "fields_skipped": 0 }
] }
```

- **Own the complete row for a doc?** Use `docs:replace` — replaying the same
  request converges to the same state (PUT semantics). **Own only some
  fields and want to add/update those without touching the rest?** Use
  `POST /collections/{id}/index` instead; `/index` is a merge, `docs:replace`
  is a full replacement.
- `version` is optional **doc-level** last-write-wins over the caller's own
  source-row version — distinct from `IndexItem.version`'s per-`(external_id,
  field)` cell versioning. A strictly-older version arriving later drops the
  *entire* item and is reported as `{"status":"dropped","current_version":...}`,
  not folded into `ok` or `error`.
- Each `ok` result carries `fields_written` and `fields_skipped` counters;
  `fields_skipped` (unchanged-value no-op suppression) is always `0` today.
- **Partial failure never fails the batch.** One bad item (unknown field,
  type mismatch) reports `{"status":"error","code":"...","message":"..."}`
  for that item while its siblings still return `ok`/`dropped`. The
  batch-level HTTP status stays 200 unless the body is malformed or the
  batch is over the size limit (400, max 32 items — the same
  `MAX_BATCH_REPLACE_SIZE` knob family as `collections:search`).
- `PUT /collections/{id}/docs/{external_id}` is single-resource sugar: body
  `{"version": ..., "fields": {...}}`, semantically identical to a one-item
  `docs:replace` batch, unwrapped back into a bare per-item result.

## Which "find" to use
- exact value / membership → `keyword` (`term`, `terms`) or `set`
- numeric range → `number` (`range` with numeric `gt`/`gte`/`lt`/`lte` bounds)
- string / date / datetime range → `keyword` (`range` with string bounds,
  compared byte/lexicographically — the same order ISO-8601 date/datetime
  strings sort chronologically in). String bounds are rejected with 400
  against a non-`keyword` field (and numeric bounds against a non-`number`
  field); `text` is explicitly out of scope for range queries
- full-text relevance → `text` + `match` (BM25). Analyzers: `whitespace_lower`,
  `ngram` (substring/CJK), `jieba` (Chinese)
- semantic similarity → `vector` + `knn` (you supply the embedding)
- perceptual / near-duplicate → `hash` + `hamming`
- hybrid lexical+semantic → `rrf` (fuse `match` + `knn` by rank; put any filter
  INSIDE each leg so the kNN leg stays filter-correct)
- autocomplete / suggest → declare a dedicated `text` field with the `ngram`
  analyzer and use `match`; lumen returns candidate `external_id`s, not
  completion strings
- which `external_id`s share a value → `POST /duplicates`
- nested data-table / "parent whose child matches" → `has_child`; combine it
  with parent-field `sort` for list-row flows that filter by child rows then
  order/count parent rows
- compose any of the above under `and` / `or` / `not`

## Search concept boundaries
These boundaries are explicit so search-engine selection does not infer silent
parity with PostGIS, OpenSearch, or MongoDB features that are not part of
Lumen's current contract.

| Concept | Disposition |
|---------|-------------|
| Geo / spatial search | Roadmap candidate; use PostGIS/MongoDB/OpenSearch or a caller-owned geospatial prefilter today, then pass matching `external_id`s to lumen. |
| Phrase / proximity queries | Roadmap candidate; current `match` is bag-of-words BM25 over analyzer tokens, not phrase order or slop. |
| Fuzzy / typo tolerance | Roadmap candidate; no edit-distance automaton today. For coarse prefix/substring recall, use the `ngram` analyzer recipe. |
| Synonyms | Caller-owned query expansion or normalized companion fields; lumen has no synonym analyzer or managed synonym dictionary. |
| Autocomplete / suggest | Recipe via a dedicated `text` field with `analyzer: "ngram"` plus `match`; hydrate suggestions from the caller's source of truth. |
| Highlighting | Non-goal: responses contain only `external_id` + `score`, and lumen does not store source text to return fragments. |
| Per-field / per-clause boost | Not supported as an arbitrary query knob. Use separate fields/query legs plus `rrf` and, if needed, final reranking in the caller. |
| Document TTL / expiry | Caller-owned lifecycle. Delete/reindex expired `external_id`s from the source-of-truth event stream; collection soft-delete grace is not per-document TTL. |

## Read consistency (`X-Read-Consistency`)
Only meaningful in primary-replica (raft) mode; standalone deployments (no
raft) ignore this header entirely — there is exactly one authoritative copy
per shard, so every level trivially holds.

- `leader` — the default, and what a missing or unrecognized header value
  also falls back to (no formal release exists yet to force a different
  default). Only the pod currently holding leadership for a shard answers;
  any other replica rejects with 503 naming the current leader.
- `any` — unconstrained; the local copy answers regardless of freshness.
- `bounded(<ms>)` — succeeds on the leader (never stale). **On a
  follower/learner it always rejects today**: lumen does not yet measure
  real inter-peer replication lag, so a non-leader replica reports the
  conservative "lag unknown" sentinel and is treated as over any bound
  rather than risk serving a stale read. Until real follower lag reporting
  ships, `bounded(<ms>)` behaves like `leader` with an extra
  follower-rejection path — do not rely on it to read from a follower.

## Routed multi-shard mode: client retry contract
Only applies to a routed multi-shard deployment (`SHARD_COUNT > 1`,
`replicasPerShard <= 1`, cross-pod forwarding — see `lumen llm --topic
deployment`); a standalone or single-shard deployment never returns these
codes. Every code below is a `503`, safe to retry with backoff — a client
that treats them as retryable rather than fatal handles a reshard split or a
rolling restart transparently:

| `code` | Meaning | Client behavior |
|--------|---------|------------------|
| `bucket_write_paused` | The write's virtual bucket is fenced for an in-progress reshard cutover's final migration pass (`POST /admin/reshard:fence`, bounded TTL). | Retry shortly; the pause is always bounded — splits/rollouts complete or the fence TTL lapses on its own. |
| `shard_forward_unavailable` | This pod forwarded the request one hop to the owning shard, but that shard was unreachable (pod down/rolling). | Retry with backoff; expected during a rolling restart. |
| `shard_map_version_mismatch` | The forwarded request declared a shard-map version that disagrees with the receiving pod's own live map — a bounded window where pods in a rolling restart are on two different `SHARD_MAP_*` env snapshots (pods only read env at boot). | Retry with backoff; wait for the rollout to converge rather than trusting either side's answer. |

None of the three above indicate data loss or a wrong answer — each is the
router refusing to give a potentially-wrong answer instead of guessing, in
favor of a retryable rejection.

Two verbs are rejected outright (not retryable) in routed multi-shard mode,
each with a documented alternative:

- `POST /collections/{id}/duplicates` → `501 duplicates_not_routed`.
  Duplicate detection is local-shard-only (filters by `min_group_size`
  before any cross-shard merge could happen) and does not support routed
  multi-shard mode; there is no cross-shard alternative today. Do not retry.
- `POST /collections/{id}/reindex/stream` → `501 reindex_stream_not_routed`.
  The streaming bulk-reindex path bypasses per-item shard ownership and the
  write fence; not supported in routed multi-shard mode. Use `POST
  /collections/{id}/index` instead — it is routed per item and observes the
  write fence like every other write path. Do not retry the stream endpoint
  itself.

## Connection
Any REST client, no driver. HTTP/1.1 is the compatibility/smoke path; the
performance target is high-QPS, large corpus traffic over pooled HTTP/2 streams,
where multiplexing and connection reuse dominate per-request overhead.

- **Production** — `https://<instance>.<namespace>.svc:7373`, a private
  ClusterIP whose TLS the serving pod terminates itself (ALPN `h2,
  http/1.1`). Nothing published sits in front of it, so the connection you
  authenticate is the connection lumen serves. Verify the server against the
  public CA distributed separately by the deployment administrator, in place of
  the public roots; authenticate yourself with a short-lived, audience-bound
  Kubernetes ServiceAccount token, sent as the request's bearer credential,
  which lumen resolves through TokenReview and SubjectAccessReview. The exact
  header form and how to mint the token are `--topic auth`.
- **Local / kind development** — `http://localhost:7373`, h2c, `spec.auth:
  disabled`. Every reachable node serves everyone; keep it off shared
  networks.

Sharded deployments route on the client:
`crc32(collection_id) % shard_count`.

## Do NOT ask lumen to
- store or return documents — it returns `external_id`s; hydrate them yourself
- run transactions or be the system of record
- aggregate (group-by / histogram / percentile / cardinality) — pair it with an
  OLAP store (ClickHouse / Druid / BigQuery / DuckDB)
- generate embeddings or hashes — you compute them; lumen indexes the bits
- return highlights, snippets, stored fields, or document payloads
- enforce per-document TTL/expiry independent of caller-owned delete/reindex
  events

## Exact wire shapes
`lumen spec` (OpenAPI), `lumen spec --shapes` (query cookbook), `lumen spec
--fields` (field/analyzer catalog), or `lumen llm --topic recipes` (task →
ready-to-POST body). `lumen llm --topic integration` covers database/pubsub
adapter boundaries.
"#
    .to_string()
}

/// The recommended database/pubsub integration boundary (`lumen llm --topic
/// integration`) as Markdown.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn llm_integration_md() -> String {
    let mut out = r#"# lumen integration

## Recommended Postgres / AlloyDB integration
Use this boundary when Postgres or AlloyDB is the source of truth:
1. Commit application data in the database first. If you need crash-safe
   delivery, write an outbox row in the same transaction or consume CDC from
   the committed log; do not make lumen a transaction participant.
2. Run an adapter/sidecar that consumes CDC, Pub/Sub, Kafka, or the outbox and
   translates each source change into lumen HTTP writes (`POST
   /collections/{id}/index` and the delete endpoint). The adapter owns cloud
   envelopes, ACK/retry/DLQ policy, upstream auth, and stale-event filtering.
3. POST to the collection's shard and ACK upstream only after lumen returns
   success. Replaying an upsert of `(external_id, field)` is safe because it
   replaces that field; retry deletes until they succeed.
4. If upstream delivery can arrive out of order, carry a monotonic
   `source_version` / commit LSN in the adapter and suppress stale writes before
   POSTing.
5. Do not publish directly to lumen's internal WAL. External producers use the
   HTTP API so every write goes through validation, routing, and the same
   log/apply path.

## Ownership boundary
- lumen core owns schema validation, sharded HTTP writes, the internal WAL,
  ordered apply, search, and ranked `external_id` responses.
- The adapter owns source-specific envelopes, Pub/Sub subscription settings,
  ACK/retry/DLQ, upstream credentials, source offsets, stale-event suppression,
  and hydration against the source database.
"#
    .to_string();
    out.push_str("\n## Shared generated-client primitive\n");
    out.push_str(openapi_codegen::llm::topic().body);
    out.push_str("\n## Shared h2c client primitive\n");
    out.push_str(transport_h2c::llm::topic().body);
    out
}

/// A copy-paste end-to-end (`lumen llm --topic quickstart`) as Markdown:
/// create → index → search against a local `lumen serve` on `:7373`.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn llm_quickstart_md() -> String {
    r#"# lumen quickstart (copy-paste)

Assumes a local node at `http://localhost:7373` (`lumen serve`), which is h2c
and `LUMEN_AUTH=disabled` — every reachable node serves everyone, so keep it
off shared networks. Clients need `LUMEN_URL` and nothing else.

A production fleet is not reached this way. It answers only at
`https://<instance>.<namespace>.svc:7373` inside the cluster, verified against
the public CA distributed separately by the deployment administrator, and each request carries a short-lived
Kubernetes ServiceAccount token that lumen resolves through
TokenReview/SubjectAccessReview. The bodies below are unchanged; the URL and
the `Authorization` header are what differ. See `lumen llm --topic auth` and
`lumen connect`.

## 1. Declare a collection
```bash
curl -sS -XPUT localhost:7373/collections/products \
  -H 'content-type: application/json' -d '{
    "fields": {
      "title":     { "type": "text", "analyzer": "whitespace_lower" },
      "brand":     { "type": "keyword" },
      "price":     { "type": "number" },
      "embedding": { "type": "vector", "dim": 3, "metric": "cosine" }
    }
  }'
```

## 2. Index items (your pub/sub does this in production)
```bash
curl -sS -XPOST localhost:7373/collections/products/index \
  -H 'content-type: application/json' -d '{
    "items": [
      { "external_id": "p1", "field": "title", "value": "wireless earbuds" },
      { "external_id": "p1", "field": "brand", "value": "acme" },
      { "external_id": "p1", "field": "price", "value": 79 },
      { "external_id": "p1", "field": "embedding", "value": [0.1, 0.2, 0.9] }
    ]
  }'
```

## 3. Search (filters + relevance)
```bash
curl -sS -XPOST localhost:7373/collections/products/search \
  -H 'content-type: application/json' -d '{
    "query": { "and": [
      { "match": { "field": "title", "text": "earbuds" } },
      { "range": { "field": "price", "lte": 100 } }
    ] },
    "limit": 10
  }'
```

## 4. Hydrate
The response is `{ "hits": [ { "external_id", "score" } ], ... }`. Fetch the full
records from YOUR store by those `external_id`s — lumen never stored them.

More shapes: `lumen llm --topic recipes`. Full schema: `lumen spec`.

## Agent-friendly one-shot wrappers
No need to hand-build curl bodies or track a port-forward yourself:

```bash
lumen connect --namespace prod --cr search -- \
  lumen query index --collection products --item 'p1:title=wireless earbuds'
lumen query search --collection products --match 'title=earbuds' --limit 10
lumen query duplicates --collection products --field email
lumen query collections list
```

`lumen connect` manages the `kubectl port-forward` and sets `LUMEN_URL` — and
only `LUMEN_URL` — for the wrapped command; `lumen query *` assembles the exact
wire body (same shapes as `lumen spec --shapes`). Neither carries a credential:
see `lumen llm --topic auth`.
"#
    .to_string()
}

/// Task → ready-to-POST body recipes (`lumen llm --topic recipes`) as Markdown,
/// rendered from [`query_shapes`] so the bodies never drift from the canonical
/// cookbook.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn llm_recipes_md() -> String {
    let shapes = query_shapes();
    let endpoint = shapes["search_endpoint"].as_str().unwrap_or("");
    let mut out = String::from("# lumen query recipes\n\n");
    if !endpoint.is_empty() {
        out.push_str(&format!("Search endpoint: `{endpoint}`\n\n"));
    }
    out.push_str(
        "Each recipe is a ready-to-POST request body. Same source as `lumen spec \
         --shapes`.\n\n",
    );
    if let Some(list) = shapes["shapes"].as_array() {
        for s in list {
            let name = s["name"].as_str().unwrap_or("recipe");
            let desc = s["description"].as_str().unwrap_or("");
            let req = serde_json::to_string_pretty(&s["request"]).unwrap_or_default();
            out.push_str(&format!("## {name}\n{desc}\n\n```json\n{req}\n```\n\n"));
        }
    }
    out
}

/// Operator storage/ops contract (`lumen llm --topic storage`) as Markdown: the
/// serving fleet's workload kind and PVC durability guarantee, independent of
/// `replicasPerShard`.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-spec-rs.md#source
pub fn llm_storage_md() -> String {
    let mut out = r#"# lumen storage

## The serving fleet is always a StatefulSet
The operator (`lumen::operator::render`) renders the serving fleet as a
Kubernetes `StatefulSet` unconditionally — never a `Deployment` — regardless
of `spec.replicasPerShard`. Every serving pod mounts a durable
`volumeClaimTemplates`-backed PVC named `raft` at `/var/lib/lumen`, sized by
`spec.serving.raftStorage` (default `20Gi`) and optionally pinned to
`spec.serving.raftStorageClass`.

This means a pod reschedule, eviction, or node loss never wipes the WAL —
including for a `replicasPerShard: 1` deployer who doesn't want or need raft
consensus. `replicasPerShard` only changes whether the fleet runs raft
consensus; it never changes whether the WAL is durable — but the PVC being
mounted is not by itself sufficient; see the next section for what actually
makes the single-member WAL durable.

## `replicasPerShard: 1` (default) — single member, no raft consensus
- One StatefulSet member per shard, with the durable `raft` PVC.
- No raft peer-identity env — the pod runs a local WAL with no consensus
  overhead.
- The legacy single-shard HPA path is serving-capacity only. It is not a
  primary/follower data-replica mode, and extra pods do not continuously catch
  up a shared shard from a primary.

## Embedded-mode persistence and crash durability (#1387)
`LUMEN_WAL=auto` resolves to `Embedded` — an in-process `MemWal` — whenever
there is no raft cluster context, i.e. exactly the `replicasPerShard: 1`
regime above. `Embedded` alone is RAM-only: mounting the `raft` PVC is not
sufficient by itself, because nothing writes to it unless `LUMEN_DATA_DIR` is
also set. Prior to #1387 the operator never set it, so a pod restart —
including the reshard cutover's own rolling restart — silently wiped all
data despite the PVC being durably attached.

The operator now renders, only at `replicasPerShard <= 1`:

```
LUMEN_DATA_DIR=/var/lib/lumen/data
LUMEN_PERSISTENCE=segment
```

`/var/lib/lumen/data` is disjoint from the raft backend's own
`/var/lib/lumen/raft` subtree (`LUMEN_RAFT_DATA_DIR`'s default) on the same
`raft` PVC mount, so both can coexist safely across a `replicasPerShard`
change without colliding. `LUMEN_PERSISTENCE=segment` (rather than the CBOR
default) activates the local AOF (`src/aof.rs`) alongside the periodic
segment checkpoint (`src/segment_rdb.rs`): every applied write is appended to
the AOF and fsynced under the `everysec` policy (at most ~1s of un-fsynced
tail on a crash — a torn tail that replay discards cleanly, not corruption),
so crash durability (kill -9 / OOM, not just a clean SIGTERM drain) is bounded
by roughly a 1-second recovery point, not by `LUMEN_SNAPSHOT_SECS` (default
300s, the periodic checkpoint interval used only to bound cold-start replay
and trim the AOF — not the durability window itself). Cold start reopens the
newest segment checkpoint, replays the AOF tail past it, then tails the
broker from there — the existing `serve()` bootstrap path, unchanged by this
render wiring.

### Dev mode: bare `lumen serve` stays in-memory
Running `lumen serve` directly (outside the operator, with no `LUMEN_DATA_DIR`
set) is unaffected and keeps today's behavior: `--wal auto` still resolves to
`Embedded`, and with no data dir configured the engine is purely in-memory —
any restart loses all data. This is intentional dev-mode behavior, not a bug:
set `--data-dir`/`LUMEN_DATA_DIR` (and optionally `--persistence=segment`)
explicitly to get the same durability the operator now wires by default.

## `replicasPerShard > 1` — raft-HA
- Fixed replica count `shardCount * replicasPerShard` (raft needs a known,
  stable peer set) — no HPA is attached.
- Each pod additionally gets the downward-API env quartet
  `raft_runtime::cluster::ClusterTopology::from_env` reads (`POD_NAME`,
  `POD_NAMESPACE`, `REPLICAS_PER_SHARD`, `VOTER_COUNT`,
  `LUMEN_HEADLESS_SERVICE`) and a stable DNS identity via the serving
  headless Service (`<name>-headless`), required for the StatefulSet's
  `serviceName`.
- This regime, including the PVC, is unchanged from before `replicasPerShard:
  1` also started getting a StatefulSet.

## Shards, replicas, and HPA
Storage topology has two independent knobs:

- `spec.shardCount` controls how many physical storage shards own the corpus.
- `spec.replicasPerShard` controls HA inside each shard group.

The serving pod count is `shardCount * replicasPerShard`. StatefulSet pod
ordinals map to topology slots with `shardIndex = ordinal % shardCount` and
`replicaIndex = ordinal / shardCount`. HPA does not change storage ownership:
it must never be used as the mechanism for increasing `shardCount` or changing
raft membership. Dynamic shard growth is an operator workflow driven by
storage pressure and a versioned virtual-bucket map, with bounded snapshot-batch
movement between physical shards.

When an operator does not know the max shard size or max shard count, it should
report the pressure condition instead of auto-splitting.

## Empty-PVC replica bootstrap
Existing pods restart from their PVC-local raft state, snapshots, and WAL. A
new replacement pod with an empty PVC should seed from an exact `SnapshotV1`
object before WAL/raft delta catch-up:

```
LUMEN_BOOTSTRAP_SEED_URI=file:///snapshots/shard-0.json
LUMEN_BOOTSTRAP_SEED_URI=s3://bucket/path/shard-0.json
```

External backup is the cold disaster-recovery and bootstrap seed surface; it is
not the normal live replica synchronization mechanism.

## Upgrading `<=0.4.9` Deployment-backed instances to `>=0.4.10` (#834)
Lumen `<=0.4.9` rendered `spec.replicasPerShard: 1` serving fleets as an
`apps/v1` `Deployment` named `<name>`. Lumen `>=0.4.10` renders the serving
fleet as an `apps/v1` `StatefulSet` with the same `<name>`. Kubernetes treats
those as different resources, and the shared operator only server-side-applies
the currently rendered child objects; it does not prune a stale child object
whose API kind changed. Applying the new operator/image alone can therefore
leave the old `Deployment/<name>` beside the new `StatefulSet/<name>`.

Use an explicit handoff for any cluster that already reconciled the CR with
`<=0.4.9`:

1. Apply the new CRD first if needed. This only updates the schema and is safe
   before the workload handoff.
2. Schedule write downtime and take an admin backup (`GET /admin/backup`) if
   you need to carry data into the new PVC-backed StatefulSet.
3. Pause the old `<=0.4.9` operator reconciliation, for example by scaling the
   operator `Deployment/lumen-operator` to zero or pausing the GitOps rollout
   that runs it. Otherwise the old operator or old HPA can recreate/scale the
   serving Deployment while you are migrating.
4. Stop the old serving workload before the `>=0.4.10` operator reconciles:

   ```
   kubectl -n <ns> scale deployment/<name> --replicas=0
   kubectl -n <ns> delete deployment/<name> --wait=true
   ```

   Scaling first is reversible; deleting with `--wait=true` makes the handoff
   boundary explicit.
5. Deploy or unpause the `>=0.4.10` operator/image and let it create
   `StatefulSet/<name>` plus the `raft-<name>-<ordinal>` PVCs.
6. Wait for the new fleet before resuming traffic or writes:

   ```
   kubectl -n <ns> rollout status statefulset/<name>
   ```

Do not run both the old `Deployment/<name>` pods and the new
`StatefulSet/<name>` pods behind the same Service. They have independent WAL /
engine storage, and the operator does not copy a Deployment pod's filesystem
or local WAL into the new StatefulSet PVC. If you must preserve data from the
old Deployment-backed pod, restore an admin backup into the new pod or rebuild
from your upstream source-of-truth before reopening writes.

## Snapshot / backup (#808)
The durable `raft` PVC protects against pod reschedule/eviction/node loss,
but it is not an off-node backup: it does not protect against a bad write, a
namespace deletion, or a lost PVC/PV. Lumen already exposes a safe,
consistent, manual snapshot-restore procedure over its admin API; production
CRs schedule the same snapshot bytes to object storage.

### Manual admin API (always available)
Every serving node — regardless of `replicasPerShard` — answers three admin
routes, each requiring `Role::Admin` on `*` (the wildcard subject, not a
per-collection grant) when `spec.auth: required`:

- `GET /admin/backup` — snapshots the live engine (`Engine::snapshot()`, the
  same quiesce-free call the raft snapshotter itself uses — no separate
  flush/quiesce step needed) and returns it as a `SnapshotV1` JSON document.
  Safe to call against any replica at any time; it does not pause writes.
- `POST /admin/backup/local` — same snapshot, written directly to a path on
  the pod's own filesystem via a `LocalFsSink` (`{"path": "...", "prefix":
  "lumen-backup"}` request body). Useful when the pod already has a mounted
  destination volume.
- `POST /admin/restore` — replaces *all* engine state with a `SnapshotV1`
  document (the same shape `/admin/backup` returns). Destructive; there is no
  merge or partial-restore mode.

These three routes are the safe procedure for ad hoc or scripted
snapshot/restore — pull with `GET /admin/backup`, keep the bytes wherever you
like, push back with `POST /admin/restore` to recover.

### Reshard admin verbs (#1380, #1389, #1396, #1457)
Six more `Role::Admin`-gated routes support moving a bounded set of
documents between shards during an operator-driven reshard, without a
full-engine restore:

- `POST /admin/backup:scoped` — like `GET /admin/backup`, but restricted to
  documents routed to a requested set of virtual buckets:
  `{"virtual_bucket_count": N, "buckets": [0, 3, ...]}`. Bucket membership is
  computed with the same hash the engine's own routing uses, so an export
  and a batch computed against the same map can never disagree about which
  documents belong to which bucket.
- `POST /admin/reshard:apply` — additively merges one `ReshardBatch`'s
  snapshot into the live engine: upsert semantics for the batch's documents,
  never a full replace, so a target shard's pre-existing data outside the
  batch is untouched. Safe to retry — replaying the same batch (operator
  resume after a checkpoint) converges to the same query-visible state.
- `POST /admin/reshard:evict` — source-side post-cutover cleanup. Given a
  newer virtual-bucket map (`{"shard": N, "map_version": V, "assignments":
  [...], "physical_shard_count": N}`) and this shard's own index within it,
  removes exactly the documents whose bucket no longer routes to this
  shard — nothing else. A separate, explicitly-invoked step; never implicit
  in `/admin/reshard:apply` or the backup routes above.
- `POST /admin/reshard:fence` (#1396 R2) — arms or clears a bounded write
  pause on a set of virtual buckets: `{"virtual_bucket_count": N, "buckets":
  [0, 3, ...], "ttl_secs": 300}`; an empty `buckets` array clears the fence
  instead of arming it. A write routed to a currently-fenced bucket is
  rejected with a retryable `503 bucket_write_paused` rather than being
  silently dropped or applied against a map that is about to change. `ttl_secs`
  defaults to 300 and is capped at 3600 (a request outside `1..=3600` is
  rejected with `400 invalid_ttl_secs`); expiry is enforced on the *serving*
  pod independent of the caller, so a caller that dies between arming and
  clearing can never leave a bucket permanently unwritable. This is a
  **driver-owned** verb: the reshard driver (`service_k8s::reshard_driver::
  advance_catching_up`) arms it over exactly the buckets its final
  `CatchingUp` migration pass is about to copy, immediately before that pass,
  and always clears it (`buckets: []`) on every exit path of that tick —
  success or `Blocked` — re-arming a fresh deadline every tick it still needs
  one (`reshard_driver::WRITE_FENCE_TTL_SECS`, 120s). Calling it manually
  outside driver-orchestrated cutover risks a real write outage: an operator
  who arms it and forgets to clear it (or races the driver's own arm/clear
  cycle) pauses writes to those buckets until the TTL lapses.
- `POST /admin/reshard:prune` (#1457 R1) — accumulates one byte-capped
  `ReshardPruneChunk` of a final migration pass's authoritative "keep" id
  set for one `(bucket, collection_id)` pair, keyed by `(to_map_version,
  bucket, collection_id, total_chunks)`, and prunes any document this shard
  holds that routes to that bucket but is absent from the accumulated set
  once every chunk has arrived. Unlike `/admin/reshard:apply` (always
  purely additive), this is what makes the final, fenced `CatchingUp` pass
  authoritative: a document deleted on the source during the split is
  absent from the accumulated keep set and is pruned here instead of
  surviving as a stale copy from an earlier additive pass. Independently
  byte-capped from `/admin/reshard:apply`'s own batches, so a bucket whose
  id set alone would exceed the body limit still converges via multiple
  chunks rather than ever producing an over-limit request. Idempotent both
  per chunk (safe to retry after a 413) and as a whole group (safe to
  re-send every chunk after a driver restart — re-running an
  already-completed group's accumulate-then-apply sequence is a no-op
  against already-pruned state).
- `POST /admin/checkpoint` — forces a synchronous, awaited durability
  checkpoint of the live engine state, bypassing the periodic
  `LUMEN_SNAPSHOT_SECS` cadence. `/admin/reshard:apply` and
  `/admin/reshard:evict` mutate engine state directly rather than through
  `WriteCoordinator`/the AOF, so without this verb their effects are only
  captured by the next periodic segment checkpoint — a window a pod restart
  can land inside and silently lose (target: the whole batch; source: the
  eviction, i.e. `documents_indexed` reverting upward). The reshard phase
  driver (`advance_catching_up`,
  `src/operator/reshard_driver.rs`) calls this on every shard touched by a
  split — every old shard plus the new one — and awaits success on all of
  them before patching `spec.shardMap` and triggering the cutover rolling
  restart, so a batch or eviction is only ever counted "migrated" once it
  can survive that restart. Returns `{"persisted": bool}`: `true` when a
  real durable store was actually written, `false` when no durable store is
  configured (e.g. tests, or a deployment running without segment
  persistence) — a vacuous success, not an error, so the verb is always safe
  to call.

  Two designs were considered for this durability gap: (a) route
  `apply`/`evict` through the AOF/`WriteCoordinator` as new log-entry types,
  or (b) the explicit synchronous checkpoint step described above, invoked
  and awaited by the driver per touched shard before cutover. (b) was
  chosen: it reuses `SegmentRdbStore::save` exactly as the periodic
  snapshotter already does — a full atomic re-seal of the current engine
  state, independent of which code path produced that state — with no new
  WAL record shape, apply-loop branch, or distinct idempotency reasoning.
  (a) would require a new `ReshardBatch`-shaped log entry that doesn't fit
  the existing single-mutation entry variants, plus a second, different
  notion of "already applied" alongside `merge_snapshot_delta`'s own
  idempotent merge semantics.

These six verbs are the data-plane building blocks for a reshard; only
`/admin/checkpoint`'s ordering relative to cutover is sequenced by the
operator phase driver — the rest do not sequence a migration end to end or
decide *when* to cut over. `/admin/reshard:fence` is armed/cleared by that
same driver around the `CatchingUp` pass — it is not an independent step an
operator sequences by hand.

### Direct CLI data movement: `dump` / `export` / `load` / `import`
For ad hoc SnapshotV1 movement from a shell, use the direct CLI wrappers:

```
lumen export --url http://localhost:7373 --out snapshot.json
lumen import --url http://localhost:7373 --file snapshot.json
```

`lumen dump` is an alias of `export`, and `lumen load` is an alias of
`import`. With no `--out`, dump/export write the exact SnapshotV1 JSON bytes to
stdout; with no `--file`, load/import read SnapshotV1 JSON from stdin. These
verbs do not add a new format, merge mode, or partial import semantics:
load/import still replace all engine state via `/admin/restore`. Neither verb
acquires a credential of its own: the metadata-server ID token they used to
mint is gone (#2871), because a Google-issued token is not something the
Kubernetes-native verifier can ever accept.

### Required production scheduled backup: `spec.serving.backup`
Production Lumen CRs set `spec.serving.backup` so the operator renders a
`<name>-backup` `batch/v1` CronJob that runs `lumen backup` on a schedule and
writes the snapshot to object storage. This adds no new snapshot mechanism — it
only *schedules and transports* the same `GET /admin/backup` bytes above to a
destination:

```yaml
spec:
  serving:
    backup:
      schedule: "0 * * * *"        # CronJob.spec.schedule
      destination: "gs://my-bucket/lumen-backups"  # file:// | s3:// | gs://
      retentionSecs: 604800        # optional; drop objects older than this
      adminTokenSecret: lumen-backup-token  # deprecated no-op; kept so pre-#2764 CRs still apply
```

Use `s3://` or `gs://` for production object-storage snapshots. GCS accepts an
explicit `GOOGLE_OAUTH_ACCESS_TOKEN`/`GCS_ACCESS_TOKEN` and otherwise resolves
the GCE/GKE metadata-server token used by Workload Identity. `file://` remains
a local-dev, migration, or break-glass sink and does not satisfy the production
service archetype.

`adminTokenSecret` (the Secret name above) is deprecated and has no effect as of
#2764; setting it generates no error but no bearer token is injected. The
backup runner authenticates as its own ServiceAccount, with a projected
audience-bound token the cluster mints for it — there is no Secret, and no
Google-issued token, anywhere on that path.

Omitting `spec.serving.backup` renders no CronJob. That is acceptable for local
development or manual recovery exercises, but a production Lumen instance is not
service-archetype complete until the scheduled object snapshot is configured.

### `lumen backup` CLI verb
The CronJob (and any ad hoc invocation) drives the same verb:

```
lumen backup --url http://<name>.<namespace>.svc.cluster.local:7373 \
  --dest s3://my-bucket/lumen-backups \
  [--retention-secs 604800]
```

`--url` points at the serving Service (not a specific pod). There is no
credential flag: #2871 removed the metadata-server ID token this used to mint,
and #2873 removed the bearer flag that was left, because a credential passed as
an argument is a credential in `ps` and in `kubectl describe`. The request
carries no `Authorization` header, which is correct against today's only
startable mode (`auth: disabled`); the projected, audience-bound ServiceAccount
token the backup runner will present instead is #2877. The verb GETs
`/admin/backup`, hands the
bytes to the `libs/service-backup` destination sink named by `--dest`, prunes
by `--retention-secs` if given, and prints the resulting `BackupRunResult` as
JSON. It needs the `backup` Cargo feature (pulled in transitively by
`operator`; the published image includes both).

## Resizing `raftStorage` (#809)
`spec.serving.raftStorage` is baked into the StatefulSet's
`volumeClaimTemplates` at first apply. Kubernetes treats
`volumeClaimTemplates` as **immutable** after creation, so editing
`spec.serving.raftStorage` on a live CR and letting the operator reconcile
does **not** resize anything — the StatefulSet `apply` is a silent no-op for
that field, and the pods' existing PVCs stay at their original size. This is
true for every `replicasPerShard` value, including the default
`replicasPerShard: 1` single-member topology.

Growing storage requires patching each per-pod PVC directly:

```
kubectl patch pvc raft-<name>-<n> --type merge \
  -p '{"spec":{"resources":{"requests":{"storage":"<new size>"}}}}'
```

This only succeeds if the PVC's bound `StorageClass` has
`allowVolumeExpansion: true`; otherwise the API server rejects the patch.
Kubernetes does not support shrinking a bound PVC — a smaller
`raftStorage` value only affects newly created PVCs (a fresh instance or a
recreated pod), never an existing one.

### `lumen k8s operator resize-storage` CLI verb
Rather than patching PVCs by hand, run the automated form of the same
procedure:

```
lumen k8s operator resize-storage --namespace <ns> --name <name> [--dry-run]
```

This fetches the named `Lumen` CR's declared `spec.serving.raftStorage`,
lists that instance's live `raft-<name>-<n>` PVCs, and for each PVC whose
current size is smaller: checks the bound `StorageClass.allowVolumeExpansion`
and, when it's `true`, patches only `spec.resources.requests.storage`
(`Patch::Merge`, no other PVC field touched) — unless `--dry-run` is given,
in which case it reports what it would do without patching anything. PVCs
already at the desired size, PVCs whose `StorageClass` does not allow
expansion, and shrink requests are reported but never mutated. It needs the
`operator` Cargo feature (`--features operator`), the same feature gate as
`lumen k8s operator run`.

## Choosing an SSD-backed StorageClass for `raftStorage` (#810)
`spec.serving.raftStorageClass` (`ServingSpec.raft_storage_class` in
`crd.rs`) is a free-text Kubernetes StorageClass name. Leaving it unset does
not mean "no StorageClass" — it means "cluster default," and on most managed
Kubernetes offerings **the cluster default is not SSD-backed**. Raft/WAL
write latency is sensitive to disk performance, so a deployer who cares
about that latency should set `raftStorageClass` explicitly rather than
relying on whatever the cluster's default happens to be.

There is no `serving.ssd` boolean toggle and no operator-side
cloud-provider detection — `raftStorageClass` is the sole mechanism, by
design (see Non-goals below). The table below is informational reference
only; verify the actual StorageClass names available on your cluster
(`kubectl get storageclass`) before setting this field, since names and
defaults vary by provider, region, and cluster version.

| Provider | Common default (usually NOT SSD) | Example SSD-backed class(es) |
|----------|-----------------------------------|-------------------------------|
| GKE | `standard-rwo` (pd-balanced) | `premium-rwo`, `pd-ssd` |
| EKS | `gp2` (older clusters) | `gp3` (tune `iops`/`throughput` parameters) |
| AKS | `default`/`managed-csi` (Standard SSD tier) | `managed-csi-premium` |
| Self-hosted / on-prem | varies by CSI driver — no universal default | ask your cluster operator; there is no cross-cluster naming convention |

```yaml
spec:
  serving:
    raftStorageClass: premium-rwo   # example: GKE SSD-backed class
```

### Non-goals: no `serving.ssd` toggle, no provider-detection
A `serving.ssd: true` boolean that maps to a hard-coded per-provider
StorageClass name was considered and explicitly rejected: cloud-provider
SSD class names change and vary across regions/versions, a hard-coded
mapping cannot know a given cluster's actual class names, it would not
cover on-prem/self-hosted Kubernetes at all, and a silently-wrong guess is
worse for a raft/WAL workload than no guess. A second toggle field
competing with the existing free-text `raftStorageClass` would also add
CRD validation ambiguity (which one wins if both are set?) for no real
gain. `raftStorageClass` already lets a deployer set any StorageClass name
they want — the fix here is this guidance, not new API surface.
"#
    .to_string();
    out.push_str("\n## Shared backup primitive\n");
    out.push_str(service_backup::llm::topic().body);
    out.push_str("\n## Shared raft-runtime primitive\n");
    out.push_str(raft_runtime::llm::topic().body);
    out
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/spec.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Canonical lossless source unit for Lumen's offline specification and
      agent documentation. Historical #809, #1298, #1321, and #1387 changes,
      plus later documentation contracts, are regenerated together above.
```
