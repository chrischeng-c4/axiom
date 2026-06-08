---
id: sdd-specs-asyncapi-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# AsyncAPI Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/specs/asyncapi.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AsyncApiInfo` | projects/agentic-workflow/src/generate/specs/asyncapi.rs | struct | pub | 39 |  |
| `AsyncApiInput` | projects/agentic-workflow/src/generate/specs/asyncapi.rs | struct | pub | 88 |  |
| `AsyncApiServer` | projects/agentic-workflow/src/generate/specs/asyncapi.rs | struct | pub | 49 |  |
| `AsyncMessage` | projects/agentic-workflow/src/generate/specs/asyncapi.rs | struct | pub | 59 |  |
| `Channel` | projects/agentic-workflow/src/generate/specs/asyncapi.rs | struct | pub | 74 |  |
| `OperationType` | projects/agentic-workflow/src/generate/specs/asyncapi.rs | enum | pub | 31 |  |
| `ServerProtocol` | projects/agentic-workflow/src/generate/specs/asyncapi.rs | enum | pub | 17 |  |
| `generate_asyncapi` | projects/agentic-workflow/src/generate/specs/asyncapi.rs | function | pub | 98 | generate_asyncapi(input: &AsyncApiInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ServerProtocol:
    type: string
    enum: [Kafka, Amqp, Mqtt, Ws, Wss, Nats, Redis]
    description: Server protocol.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: lowercase

  OperationType:
    type: string
    enum: [Publish, Subscribe]
    description: Operation type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: lowercase

  AsyncApiInfo:
    type: object
    required: [title, version]
    description: API info.
    properties:
      title: { type: string }
      version: { type: string }
      description: { type: string }
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  AsyncApiServer:
    type: object
    required: [url, protocol]
    description: Server definition.
    properties:
      url: { type: string }
      protocol: { $ref: "#/definitions/ServerProtocol" }
      description: { type: string }
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  AsyncMessage:
    type: object
    description: Message definition.
    properties:
      name: { type: string }
      title: { type: string }
      summary: { type: string }
      content_type: { type: string }
      payload:
        type: object
        x-rust-type: "Option<Value>"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Channel:
    type: object
    required: [name, operation]
    description: Channel definition.
    properties:
      name: { type: string }
      operation: { $ref: "#/definitions/OperationType" }
      summary: { type: string }
      description: { type: string }
      message:
        $ref: "#/definitions/AsyncMessage"
        x-rust-type: "Option<AsyncMessage>"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  AsyncApiInput:
    type: object
    required: [info, channels, servers, schemas]
    description: Input for AsyncAPI generation.
    properties:
      info: { $ref: "#/definitions/AsyncApiInfo" }
      channels:
        type: array
        items: { $ref: "#/definitions/Channel" }
      servers:
        type: object
        x-rust-type: "HashMap<String, AsyncApiServer>"
        x-serde-default: true
      schemas:
        type: object
        x-rust-type: "HashMap<String, Value>"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/specs/asyncapi.rs -->
```rust
//! AsyncAPI 2.6 Specification Generation
//!
//! Generates AsyncAPI 2.6 specifications for event-driven APIs.

use crate::generate::{GenerateError, Result};
use serde_json::{json, Value};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Server protocol.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/asyncapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerProtocol {
    Kafka,
    Amqp,
    Mqtt,
    Ws,
    Wss,
    Nats,
    Redis,
}

/// Operation type.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/asyncapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationType {
    Publish,
    Subscribe,
}

/// API info.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/asyncapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncApiInfo {
    pub title: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Server definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/asyncapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncApiServer {
    pub url: String,
    pub protocol: ServerProtocol,
    #[serde(default)]
    pub description: Option<String>,
}

/// Message definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/asyncapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncMessage {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
    pub payload: Option<Value>,
}

/// Channel definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/asyncapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub operation: OperationType,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub message: Option<AsyncMessage>,
}

/// Input for AsyncAPI generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/asyncapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncApiInput {
    pub info: AsyncApiInfo,
    pub channels: Vec<Channel>,
    #[serde(default)]
    pub servers: HashMap<String, AsyncApiServer>,
    #[serde(default)]
    pub schemas: HashMap<String, Value>,
}
/// Generate an AsyncAPI 2.6 specification
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/asyncapi.md#source
pub fn generate_asyncapi(input: &AsyncApiInput) -> Result<String> {
    if input.channels.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one channel required".to_string(),
        ));
    }

    let mut asyncapi = json!({
        "asyncapi": "2.6.0",
        "info": {
            "title": input.info.title,
            "version": input.info.version
        }
    });

    if let Some(ref desc) = input.info.description {
        asyncapi["info"]["description"] = json!(desc);
    }

    // Servers
    if !input.servers.is_empty() {
        let mut servers_obj = serde_json::Map::new();
        for (name, server) in &input.servers {
            let protocol = match server.protocol {
                ServerProtocol::Kafka => "kafka",
                ServerProtocol::Amqp => "amqp",
                ServerProtocol::Mqtt => "mqtt",
                ServerProtocol::Ws => "ws",
                ServerProtocol::Wss => "wss",
                ServerProtocol::Nats => "nats",
                ServerProtocol::Redis => "redis",
            };
            let mut server_obj = json!({
                "url": server.url,
                "protocol": protocol
            });
            if let Some(ref desc) = server.description {
                server_obj["description"] = json!(desc);
            }
            servers_obj.insert(name.clone(), server_obj);
        }
        asyncapi["servers"] = Value::Object(servers_obj);
    }

    // Channels
    let mut channels_obj = serde_json::Map::new();
    for channel in &input.channels {
        let op_key = match channel.operation {
            OperationType::Publish => "publish",
            OperationType::Subscribe => "subscribe",
        };

        let mut op_obj = json!({});
        if let Some(ref summary) = channel.summary {
            op_obj["summary"] = json!(summary);
        }
        if let Some(ref desc) = channel.description {
            op_obj["description"] = json!(desc);
        }
        if let Some(ref msg) = channel.message {
            let mut msg_obj = json!({});
            if let Some(ref name) = msg.name {
                msg_obj["name"] = json!(name);
            }
            if let Some(ref title) = msg.title {
                msg_obj["title"] = json!(title);
            }
            if let Some(ref summary) = msg.summary {
                msg_obj["summary"] = json!(summary);
            }
            if let Some(ref ct) = msg.content_type {
                msg_obj["contentType"] = json!(ct);
            }
            if let Some(ref payload) = msg.payload {
                msg_obj["payload"] = payload.clone();
            }
            op_obj["message"] = msg_obj;
        }

        channels_obj.insert(channel.name.clone(), json!({ op_key: op_obj }));
    }
    asyncapi["channels"] = Value::Object(channels_obj);

    // Components/schemas
    if !input.schemas.is_empty() {
        asyncapi["components"] = json!({ "schemas": input.schemas });
    }

    serde_yaml::to_string(&asyncapi).map_err(|e| GenerateError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_asyncapi() {
        let input = AsyncApiInput {
            info: AsyncApiInfo {
                title: "User Events".to_string(),
                version: "1.0.0".to_string(),
                description: Some("User event API".to_string()),
            },
            channels: vec![Channel {
                name: "user.created".to_string(),
                operation: OperationType::Publish,
                summary: Some("User created event".to_string()),
                description: None,
                message: Some(AsyncMessage {
                    name: Some("UserCreated".to_string()),
                    title: None,
                    summary: None,
                    content_type: Some("application/json".to_string()),
                    payload: Some(json!({"type": "object"})),
                }),
            }],
            servers: HashMap::new(),
            schemas: HashMap::new(),
        };

        let result = generate_asyncapi(&input).unwrap();
        assert!(result.contains("asyncapi: 2.6.0"));
        assert!(result.contains("user.created"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/specs/asyncapi.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete AsyncAPI specification generation module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
