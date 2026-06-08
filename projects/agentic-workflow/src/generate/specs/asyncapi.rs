// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/specs/asyncapi.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
