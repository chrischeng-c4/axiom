// HANDWRITE-BEGIN gap="missing-generator:logic:9d2fdaf7" tracker="#777" reason="New offline spec module: openapi_json/openapi_yaml/json_schema_json reuse http::routes::openapi (the /openapi.json accessor); request_shapes is a keep operation cookbook and value_catalog mirrors the KvValue enum."
//! Offline, machine-readable self-description for agent integration.
//!
//! `keep spec` emits everything an agent needs to wire keep into a pipeline —
//! the OpenAPI document (the offline twin of the served `/openapi.json`), the
//! component JSON-Schema, a request-shape cookbook, and the value-type catalog
//! — straight from the installed binary, with no running server and no network.
//! Every emitter reuses the exact accessor the server mounts at `/openapi.json`
//! ([`crate::http::routes::openapi`]), so the offline document and the served
//! one can never drift.
//!
//! @spec projects/keep/tech-design/interfaces/cli/deploy-cli-keep-spec-spec-gen-dockerfile-render.md

use serde_json::{json, Value};

/// The full OpenAPI 3 document as pretty JSON — the offline twin of the served
/// `/openapi.json` (same source accessor: [`crate::http::routes::openapi`]).
pub fn openapi_json() -> String {
    crate::http::routes::openapi()
        .to_pretty_json()
        .expect("OpenApi serializes to JSON")
}

/// The full OpenAPI 3 document as YAML, for LLM/agent reading.
pub fn openapi_yaml() -> String {
    serde_yaml::to_string(&crate::http::routes::openapi()).expect("OpenApi serializes to YAML")
}

/// Just the component schemas (the request/response data types) as pretty JSON —
/// the JSON-Schema view an agent uses to build/validate request bodies.
pub fn json_schema_json() -> String {
    let api = crate::http::routes::openapi();
    serde_json::to_string_pretty(&json!({ "components": api.components }))
        .expect("components serialize to JSON")
}

/// A cookbook of canonical request shapes for keep's HTTP surface. Each entry is
/// a ready-to-send `{name, description, request:{method, path, body?}}` for one
/// operation family, using the exact route + DTO wire form.
pub fn request_shapes() -> Value {
    json!({
        "base_url": "http://localhost:7117",
        "note": "Structured values are native JSON (application/json); opaque claim-check blobs use application/octet-stream on the value path and never round-trip through JSON.",
        "shapes": [
            { "name": "set", "description": "store a scalar/JSON value (optional TTL in ms)",
              "request": { "method": "PUT", "path": "/v1/kv/{key}", "body": { "value": "hello", "ttl_ms": 60000 } } },
            { "name": "get", "description": "read a value",
              "request": { "method": "GET", "path": "/v1/kv/{key}" } },
            { "name": "delete", "description": "remove a key",
              "request": { "method": "DELETE", "path": "/v1/kv/{key}" } },
            { "name": "incr", "description": "atomic integer add (negative delta decrements)",
              "request": { "method": "POST", "path": "/v1/kv/{key}/incr", "body": { "delta": 1 } } },
            { "name": "cas", "description": "compare-and-swap: write `new` iff the current value equals `expected`",
              "request": { "method": "POST", "path": "/v1/kv/{key}/cas", "body": { "expected": "old", "new": "new" } } },
            { "name": "setnx", "description": "set only if the key is absent",
              "request": { "method": "POST", "path": "/v1/kv/{key}/setnx", "body": { "value": "once" } } },
            { "name": "mset", "description": "batch set",
              "request": { "method": "POST", "path": "/v1/kv:mset", "body": { "entries": { "a": 1, "b": 2 } } } },
            { "name": "mget", "description": "batch get (parallel to keys; null where absent)",
              "request": { "method": "POST", "path": "/v1/kv:mget", "body": { "keys": ["a", "b"] } } },
            { "name": "mdel", "description": "batch delete",
              "request": { "method": "POST", "path": "/v1/kv:mdel", "body": { "keys": ["a", "b"] } } },
            { "name": "scan", "description": "prefix scan (keys only)",
              "request": { "method": "GET", "path": "/v1/kv?prefix=user:&limit=100" } },
            { "name": "expire", "description": "set a TTL on any existing key (seconds or ms)",
              "request": { "method": "POST", "path": "/v1/kv/{key}/expire", "body": { "ms": 60000 } } },
            { "name": "lock", "description": "acquire an owner+TTL advisory lease",
              "request": { "method": "POST", "path": "/v1/locks/{name}", "body": { "owner": "worker-1", "ttl_ms": 30000 } } },
            { "name": "lpush", "description": "prepend values to a list",
              "request": { "method": "POST", "path": "/v1/lists/{key}/lpush", "body": { "values": ["a", "b"] } } },
            { "name": "hset", "description": "write hash fields",
              "request": { "method": "POST", "path": "/v1/hashes/{key}", "body": { "fields": { "name": "ada" } } } },
            { "name": "sadd", "description": "add set members",
              "request": { "method": "POST", "path": "/v1/sets/{key}", "body": { "members": ["x", "y"] } } },
            { "name": "zadd", "description": "add scored members to a sorted set",
              "request": { "method": "POST", "path": "/v1/zsets/{key}", "body": { "members": [ { "member": "alice", "score": 10.0 } ] } } },
            { "name": "claim_check_input", "description": "worker job input payload by message id (bytes-first)",
              "request": { "method": "PUT", "path": "/v1/inputs/{id}", "body": "<application/octet-stream bytes>" } },
            { "name": "claim_check_result", "description": "worker job result payload by message id (bytes-first)",
              "request": { "method": "PUT", "path": "/v1/results/{id}", "body": "<application/octet-stream bytes>" } }
        ]
    })
}

/// The value-type catalog — the engine value kinds keep stores and returns,
/// mirroring the [`crate::types::KvValue`] enum, plus the collection endpoint
/// that holds each aggregate type.
pub fn value_catalog() -> Value {
    json!({
        "note": "Scalars are set/read on /v1/kv/{key}; aggregates use their typed collection endpoints. JSON integers map to int, fractional numbers to float, and bool to int.",
        "value_types": [
            { "type": "int", "purpose": "64-bit signed integer; target of incr / cas" },
            { "type": "float", "purpose": "64-bit floating point" },
            { "type": "decimal", "purpose": "128-bit fixed-point decimal (financial precision); rendered as a string in JSON" },
            { "type": "string", "purpose": "UTF-8 string" },
            { "type": "bytes", "purpose": "opaque binary blob via the application/octet-stream value path (claim-check)" },
            { "type": "list", "purpose": "ordered list", "endpoint": "/v1/lists/{key}" },
            { "type": "map", "purpose": "field -> value hash", "endpoint": "/v1/hashes/{key}" },
            { "type": "set", "purpose": "unordered unique strings", "endpoint": "/v1/sets/{key}" },
            { "type": "sorted_set", "purpose": "scored members in score order", "endpoint": "/v1/zsets/{key}" },
            { "type": "null", "purpose": "explicit null / absent value" }
        ]
    })
}
// HANDWRITE-END
