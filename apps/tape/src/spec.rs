// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-src-spec-rs.md#schema
// <HANDWRITE gap="missing-generator:logic:tape-bootstrap" tracker="#768" reason="Initial offline route/spec/LLM contract before Tape has generated OpenAPI artifacts.">
use serde_json::{json, Value};

pub fn openapi_json() -> String {
    serde_json::to_string_pretty(&openapi()).expect("Tape OpenAPI serializes")
}

pub fn openapi_yaml() -> String {
    serde_yaml::to_string(&openapi()).expect("Tape OpenAPI serializes as YAML")
}

pub fn json_schema_json() -> String {
    serde_json::to_string_pretty(&json!({
        "components": {
            "schemas": schemas(),
        }
    }))
    .expect("Tape schemas serialize")
}

pub fn routes_json() -> String {
    serde_json::to_string_pretty(&json!({
        "project": "tape",
        "routes": [
            {"method": "GET", "path": "/healthz", "purpose": "liveness"},
            {"method": "GET", "path": "/readyz", "purpose": "readiness"},
            {"method": "GET", "path": "/metrics", "purpose": "Prometheus metrics"},
            {"method": "GET", "path": "/openapi.json", "purpose": "machine OpenAPI"},
            {"method": "GET", "path": "/docs", "purpose": "Swagger UI"},
            {"method": "POST", "path": "/topics/{topic}/append", "purpose": "append event envelopes"},
            {"method": "GET", "path": "/topics/{topic}/replay", "purpose": "replay by offset or timestamp"},
            {"method": "GET", "path": "/topics/{topic}/replay/stream", "purpose": "compact read-only h2c bulk replay"},
            {"method": "POST", "path": "/topics/{topic}/subscriptions", "purpose": "create a named pull cursor"},
            {"method": "GET", "path": "/topics/{topic}/subscriptions", "purpose": "list named pull cursors"},
            {"method": "GET", "path": "/topics/{topic}/subscriptions/{subscription}", "purpose": "inspect one named pull cursor"},
            {"method": "DELETE", "path": "/topics/{topic}/subscriptions/{subscription}", "purpose": "delete named pull cursor metadata"},
            {"method": "POST", "path": "/topics/{topic}/subscriptions/{subscription}/pull", "purpose": "read a bounded pull subscription window"},
            {"method": "POST", "path": "/topics/{topic}/subscriptions/{subscription}/ack", "purpose": "advance a pull subscription cursor"},
            {"method": "PUT", "path": "/topics/{topic}/consumers/{consumer}/checkpoint", "purpose": "advance a durable consumer cursor"},
            {"method": "GET", "path": "/topics/{topic}/consumers/{consumer}/checkpoint", "purpose": "read a durable consumer cursor"},
            {"method": "PUT", "path": "/topics/{topic}/retention", "purpose": "configure retention windows"}
        ]
    }))
    .expect("Tape route list serializes")
}

pub const fn llm_workflow_md() -> &'static str {
    r#"# tape workflow

Tape is the historical topic replay journal. Relay owns online broker delivery;
Tape owns append-only topic history, offset/time replay, consumer checkpoints,
retention, and backfill/audit reads.

Start with:

```text
tape append orders --payload '{"id":"o1"}'
tape replay orders --from-offset 0
tape checkpoint put orders worker-a --offset 1
tape subscription create orders worker-a
```

Subscription creation is intrinsically caller-driven pull; Tape has no push,
lease, consumer-group, or bidirectional consume mode. Serving supports durable
Raft replication, operator-managed Kubernetes deployment, and external replay
benchmark gates through their dedicated commands and evidence.
"#
}

pub const fn llm_api_md() -> &'static str {
    r#"# tape API

The initial service contract is intentionally compact:

- `POST /topics/{topic}/append` appends an event envelope and returns its offset.
- `GET /topics/{topic}/replay?from_offset=N&from_timestamp_ms=T&limit=L` replays history.
- `GET /topics/{topic}/replay/stream?from_offset=N&from_timestamp_ms=T&limit=L` downloads the same read-only history as compact validated frames over h2c.
- `POST`/`GET /topics/{topic}/subscriptions` declare topic delivery resources.
- `GET`/`DELETE /topics/{topic}/subscriptions/{subscription}` declare one resource.
- `POST /topics/{topic}/subscriptions/{subscription}/pull` declares bounded pull reads.
- `POST /topics/{topic}/subscriptions/{subscription}/ack` declares explicit cursor advance.
- `PUT /topics/{topic}/consumers/{consumer}/checkpoint` advances a replay cursor.
- `GET /topics/{topic}/consumers/{consumer}/checkpoint` reads the replay cursor.
- `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, and `/docs` are the standard service endpoints.

Use `tape spec --format routes` for the route inventory and `tape spec --format
openapi-yaml` when an agent needs a readable OpenAPI-shaped contract.
"#
}

pub const fn llm_boundaries_md() -> &'static str {
    r#"# tape boundaries

- Relay owns low-latency online pub/sub, work-queue leasing, ack, and redelivery.
- Tape owns historical topic replay, backfill, retention, audit, and checkpoints.
- Loom may write workflow events into Tape, but workflow decisions remain Loom state.
- Keep stores payload/result bytes; Tape stores event envelopes and claim-check refs.
"#
}

fn openapi() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Tape API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Topic replay journal API for append, replay, subscription resource inventory, checkpoints, retention, and standard service endpoints."
        },
        "paths": {
            "/healthz": {"get": {"summary": "Liveness probe", "responses": ok_text()}},
            "/readyz": {"get": {"summary": "Readiness probe", "responses": ok_text()}},
            "/metrics": {"get": {"summary": "Prometheus metrics", "responses": ok_text()}},
            "/openapi.json": {"get": {"summary": "OpenAPI document", "responses": ok_json()}},
            "/docs": {"get": {"summary": "Swagger UI", "responses": ok_text()}},
            "/topics/{topic}/append": {
                "post": {
                    "summary": "Append an event envelope to a topic journal",
                    "parameters": [topic_param()],
                    "requestBody": json_body("AppendEventRequest"),
                    "responses": ok_schema("TapeEvent")
                }
            },
            "/topics/{topic}/replay": {
                "get": {
                    "summary": "Replay topic history by offset or timestamp",
                    "parameters": [
                        topic_param(),
                        query_param("from_offset", "integer"),
                        query_param("from_timestamp_ms", "integer"),
                        query_param("limit", "integer")
                    ],
                    "responses": ok_schema("ReplayResponse")
                }
            },
            "/topics/{topic}/replay/stream": {
                "get": {
                    "summary": "Download compact read-only topic replay frames",
                    "parameters": [
                        topic_param(),
                        query_param("from_offset", "integer"),
                        query_param("from_timestamp_ms", "integer"),
                        query_param("limit", "integer")
                    ],
                    "responses": {
                        "200": {
                            "description": "Length-framed Tape replay stream",
                            "content": {
                                "application/vnd.tape.replay.v1": {
                                    "schema": {"type": "string", "contentEncoding": "binary"}
                                }
                            }
                        }
                    }
                }
            },
            "/topics/{topic}/subscriptions": {
                "post": {
                    "summary": "Create a topic delivery resource",
                    "parameters": [topic_param()],
                    "requestBody": json_body("SubscriptionCreateRequest"),
                    "responses": ok_schema("Subscription")
                },
                "get": {
                    "summary": "List topic delivery resources",
                    "parameters": [topic_param()],
                    "responses": ok_schema("SubscriptionListResponse")
                }
            },
            "/topics/{topic}/subscriptions/{subscription}": {
                "get": {
                    "summary": "Inspect one topic delivery resource",
                    "parameters": [topic_param(), subscription_param()],
                    "responses": ok_schema("Subscription")
                },
                "delete": {
                    "summary": "Delete topic delivery resource metadata",
                    "parameters": [topic_param(), subscription_param()],
                    "responses": ok_schema("Subscription")
                }
            },
            "/topics/{topic}/subscriptions/{subscription}/pull": {
                "post": {
                    "summary": "Read a bounded pull subscription window",
                    "parameters": [topic_param(), subscription_param()],
                    "requestBody": json_body("PullSubscriptionRequest"),
                    "responses": ok_schema("PullSubscriptionBatch")
                }
            },
            "/topics/{topic}/subscriptions/{subscription}/ack": {
                "post": {
                    "summary": "Advance a pull subscription cursor",
                    "parameters": [topic_param(), subscription_param()],
                    "requestBody": json_body("PullSubscriptionAckRequest"),
                    "responses": ok_schema("ConsumerCheckpoint")
                }
            },
            "/topics/{topic}/consumers/{consumer}/checkpoint": {
                "get": {
                    "summary": "Read a consumer replay checkpoint",
                    "parameters": [topic_param(), consumer_param()],
                    "responses": ok_schema("ConsumerCheckpoint")
                },
                "put": {
                    "summary": "Advance a consumer replay checkpoint",
                    "parameters": [topic_param(), consumer_param()],
                    "requestBody": json_body("CheckpointRequest"),
                    "responses": ok_schema("ConsumerCheckpoint")
                }
            },
            "/topics/{topic}/retention": {
                "put": {
                    "summary": "Configure a topic retention window",
                    "parameters": [topic_param()],
                    "requestBody": json_body("RetentionPolicy"),
                    "responses": ok_schema("RetentionPolicy")
                }
            }
        },
        "components": {
            "schemas": schemas()
        }
    })
}

fn schemas() -> Value {
    json!({
        "AppendEventRequest": {
            "type": "object",
            "required": ["payload"],
            "properties": {
                "key": {"type": "string"},
                "timestamp_ms": {"type": "integer", "minimum": 0},
                "payload": {"description": "Caller-owned event envelope or claim-check reference"}
            }
        },
        "TapeEvent": {
            "type": "object",
            "required": ["topic", "offset", "timestamp_ms", "payload"],
            "properties": {
                "topic": {"type": "string"},
                "offset": {"type": "integer", "minimum": 0},
                "timestamp_ms": {"type": "integer", "minimum": 0},
                "key": {"type": "string"},
                "payload": {}
            }
        },
        "ReplayResponse": {
            "type": "object",
            "required": ["events"],
            "properties": {
                "events": {"type": "array", "items": {"$ref": "#/components/schemas/TapeEvent"}}
            }
        },
        "SubscriptionCreateRequest": {
            "type": "object",
            "additionalProperties": false,
            "required": ["name"],
            "properties": {
                "name": {"type": "string"}
            }
        },
        "Subscription": {
            "type": "object",
            "required": ["topic", "name"],
            "properties": {
                "topic": {"type": "string"},
                "name": {"type": "string"}
            }
        },
        "SubscriptionListResponse": {
            "type": "object",
            "required": ["subscriptions"],
            "properties": {
                "subscriptions": {"type": "array", "items": {"$ref": "#/components/schemas/Subscription"}}
            }
        },
        "PullSubscriptionRequest": {
            "type": "object",
            "properties": {
                "limit": {"type": "integer", "minimum": 0, "maximum": 1000, "default": 100}
            }
        },
        "PullSubscriptionBatch": {
            "type": "object",
            "required": ["topic", "subscription", "cursor", "limit", "next_offset", "events"],
            "properties": {
                "topic": {"type": "string"},
                "subscription": {"type": "string"},
                "cursor": {"type": "integer", "minimum": 0},
                "limit": {"type": "integer", "minimum": 0, "maximum": 1000},
                "next_offset": {"type": "integer", "minimum": 0},
                "events": {"type": "array", "items": {"$ref": "#/components/schemas/TapeEvent"}}
            }
        },
        "PullSubscriptionAckRequest": {
            "type": "object",
            "required": ["offset"],
            "properties": {
                "offset": {"type": "integer", "minimum": 0}
            }
        },
        "CheckpointRequest": {
            "type": "object",
            "required": ["offset"],
            "properties": {
                "offset": {"type": "integer", "minimum": 0}
            }
        },
        "ConsumerCheckpoint": {
            "type": "object",
            "required": ["topic", "consumer", "offset", "updated_at_ms"],
            "properties": {
                "topic": {"type": "string"},
                "consumer": {"type": "string"},
                "offset": {"type": "integer", "minimum": 0},
                "updated_at_ms": {"type": "integer", "minimum": 0}
            }
        },
        "RetentionPolicy": {
            "type": "object",
            "properties": {
                "min_offset": {"type": "integer", "minimum": 0},
                "max_age_seconds": {"type": "integer", "minimum": 0},
                "protected_consumers": {"type": "array", "items": {"type": "string"}}
            }
        }
    })
}

fn topic_param() -> Value {
    path_param("topic", "Topic name")
}

fn consumer_param() -> Value {
    path_param("consumer", "Consumer checkpoint name")
}

fn subscription_param() -> Value {
    path_param("subscription", "Topic delivery resource name")
}

fn path_param(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "in": "path",
        "required": true,
        "description": description,
        "schema": {"type": "string"}
    })
}

fn query_param(name: &str, kind: &str) -> Value {
    json!({
        "name": name,
        "in": "query",
        "required": false,
        "schema": {"type": kind}
    })
}

fn json_body(schema: &str) -> Value {
    json!({
        "required": true,
        "content": {
            "application/json": {
                "schema": {"$ref": format!("#/components/schemas/{schema}")}
            }
        }
    })
}

fn ok_schema(schema: &str) -> Value {
    json!({
        "200": {
            "description": "ok",
            "content": {
                "application/json": {
                    "schema": {"$ref": format!("#/components/schemas/{schema}")}
                }
            }
        }
    })
}

fn ok_json() -> Value {
    json!({"200": {"description": "ok", "content": {"application/json": {"schema": {"type": "object"}}}}})
}

fn ok_text() -> Value {
    json!({"200": {"description": "ok", "content": {"text/plain": {"schema": {"type": "string"}}}}})
}
// </HANDWRITE>
