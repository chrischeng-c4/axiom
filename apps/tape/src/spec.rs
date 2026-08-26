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
            {"method": "GET", "path": "/admin/backup", "purpose": "admin-gated whole-journal snapshot"},
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
            {"method": "PUT", "path": "/topics/{topic}/retention", "purpose": "configure retention windows"},
            {"method": "GET", "path": "/topics/{topic}/retention", "purpose": "read the current retention window"}
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
- `GET /admin/backup` returns an admin-gated whole-journal snapshot.
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

/// Prose companion to the `TopicSection::Generated` section in
/// [`LLM_BACKUP_TOPICS`] — everything about Tape's own backup usage except
/// the destination scheme list. #2483 found three hand-copied
/// `file://`+`s3://`-only claims (CLI help, deployment-handoff.md, README)
/// that had drifted stale against the unconditionally-shipped `gs://`
/// scheme; #2494 is the durable fix: derive the scheme list from
/// `service_backup::SUPPORTED_SCHEMES` at call time instead of freezing a
/// copy into a `&'static str`.
const LLM_BACKUP_INTRO: &str = r#"# tape backup

`tape backup` (feature `backup`) fetches `GET /admin/backup`'s exact
whole-journal `JournalSnapshot` over HTTP and ships it to a
`libs/service-backup` destination sink; `tape serve --bootstrap-seed-uri`
reads the same object back to cold-seed a fresh, empty replica PVC before
Raft starts. It is never a live in-place restore of a running node. Tape
keeps no destination scheme list of its own — the contract below is
composed live from the shared `service-backup` library so it can never go
stale independent of what the linked build actually accepts."#;

/// Render the shared `service-backup` destination contract at call time via
/// `cli_std::llm::render_sectioned` instead of hand-copying its scheme list
/// into a Tape-owned `&'static str` (#2483, #2494). Uses
/// `service_backup::llm::sectioned_topic()` for the topic id so this file
/// never hand-copies that literal either.
fn llm_backup_destination_contract() -> String {
    let backing = service_backup::llm::sectioned_topic();
    cli_std::llm::render_sectioned(
        "tape",
        env!("CARGO_PKG_VERSION"),
        service_backup::llm::SECTIONED_TOPICS,
        backing.id,
        cli_std::llm::Format::Md,
    )
    .unwrap_or_else(|e| {
        panic!(
            "service_backup::llm::SECTIONED_TOPICS failed to render its `{}` topic: {e}",
            backing.id
        )
    })
}

/// Tape's backup-destination LLM topic, composed from
/// [`service_backup::llm::sectioned_topic`] (#2483, #2494): the destination
/// scheme list is a `TopicSection::Generated` section rendered from
/// `service_backup::SUPPORTED_SCHEMES` at call time, so it always names
/// whatever schemes the linked `service-backup` build actually accepts
/// instead of a hand-copied `file://`/`s3://`-only literal that drifted
/// stale (the exact class #2483 found and fixed).
///
/// Not yet wired into `tape llm`'s dispatch: `LLM_TOPICS`/`llm()` in
/// `src/bin/tape.rs` still speak the static `&[cli_std::llm::Topic]` shape
/// and render via `cli_std::llm::render`. Adding a `backup` topic to that
/// dispatch needs `cli_std::llm::render_sectioned` (or a mixed dispatch)
/// and is `src/bin/tape.rs`-owned follow-up CLI-surface work, out of this
/// file's scope.
pub const LLM_BACKUP_TOPICS: &[cli_std::llm::SectionedTopic] = &[cli_std::llm::SectionedTopic {
    id: "backup",
    summary: "backup destination schemes, feature gating, and cold-seed restore, composed live from the shared service-backup contract",
    sections: &[
        cli_std::llm::TopicSection::Prose(LLM_BACKUP_INTRO),
        cli_std::llm::TopicSection::Generated {
            id: "service-backup-destination-contract",
            render: llm_backup_destination_contract,
        },
    ],
}];

/// Return Tape's backup-destination topic in
/// [`cli_std::llm::SectionedTopic`] form for CLI composition.
pub fn llm_backup_sectioned_topic() -> &'static cli_std::llm::SectionedTopic {
    &LLM_BACKUP_TOPICS[0]
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
            "/admin/backup": {
                "get": {
                    "summary": "Download an admin-gated whole-journal snapshot",
                    "responses": {
                        "200": {
                            "description": "JournalSnapshot JSON containing the journal at the applied Raft index",
                            "content": {
                                "application/json": {
                                    "schema": {"type": "object"}
                                }
                            }
                        }
                    }
                }
            },
            "/topics/{topic}/append": {
                "post": {
                    "summary": "Append an event envelope to a topic journal",
                    "parameters": [topic_param()],
                    "requestBody": json_body("AppendEventRequest"),
                    "responses": mutating_schema("TapeEvent")
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
                    "responses": mutating_schema("Subscription")
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
                    "responses": mutating_schema("Subscription")
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
                    "responses": mutating_schema("ConsumerCheckpoint")
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
                    "responses": mutating_schema("ConsumerCheckpoint")
                }
            },
            "/topics/{topic}/retention": {
                "get": {
                    "summary": "Read a topic's current retention window",
                    "parameters": [topic_param()],
                    "responses": ok_schema("RetentionPolicy")
                },
                "put": {
                    "summary": "Configure a topic retention window",
                    "parameters": [topic_param()],
                    "requestBody": json_body("RetentionPolicy"),
                    "responses": mutating_schema("RetentionPolicy")
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

/// #2573: the response set for an operation that writes through the journal
/// persist path.
///
/// This document otherwise lists only success responses, and 4xx stays out of
/// it deliberately — a client already handles those by status class. `507` is
/// different in kind: it is the one status where the correct client behavior
/// (stop, surface the condition, retry on a human timescale) differs from what
/// a generic retry policy would do with a 5xx, so a generated client that does
/// not know it exists will hammer a full disk. Read-only operations never
/// reach the persist path and so never carry it.
fn mutating_schema(schema: &str) -> Value {
    let mut responses = ok_schema(schema);
    responses["507"] = json!({
        "description": "Node is in ENOSPC degraded read-only mode (error kind `storage_full`); \
                        reads keep serving and the node re-probes its store to recover itself"
    });
    responses
}

fn ok_json() -> Value {
    json!({"200": {"description": "ok", "content": {"application/json": {"schema": {"type": "object"}}}}})
}

fn ok_text() -> Value {
    json!({"200": {"description": "ok", "content": {"text/plain": {"schema": {"type": "string"}}}}})
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drift gate for #2483/#2494: the sectioned topic must render cleanly
    /// through `cli_std::llm::assert_topics_render`, exactly how
    /// `libs/cli-std`'s and `libs/service-backup`'s own tests exercise the
    /// same helper.
    #[test]
    fn llm_backup_topic_conforms() {
        cli_std::llm::assert_topics_render(LLM_BACKUP_TOPICS);
    }

    /// Drift gate for #2483: fails the moment Tape's backup topic stops
    /// naming every scheme `service_backup::SUPPORTED_SCHEMES` accepts —
    /// the canonical fact source `libs/service-backup/src/destination.rs`
    /// derives from `cfg!`-evaluated feature flags, so a scheme addition or
    /// a feature-gating change trips this without a hand edit anywhere.
    #[test]
    fn llm_backup_topic_lists_every_supported_scheme() {
        let rendered = llm_backup_destination_contract();
        for info in service_backup::SUPPORTED_SCHEMES {
            assert!(
                rendered.contains(info.scheme),
                "tape's backup llm topic is missing scheme `{}` from service_backup::SUPPORTED_SCHEMES — #2483 drift is back",
                info.scheme
            );
        }
    }

    #[test]
    fn llm_backup_sectioned_topic_matches_id() {
        assert_eq!(llm_backup_sectioned_topic().id, "backup");
        assert_eq!(LLM_BACKUP_TOPICS.len(), 1);
    }
}
// </HANDWRITE>
