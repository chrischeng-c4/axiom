//! AsyncAPI 2.x/3.x to EventApiSpec parser
//!
//! Parses AsyncAPI specifications for event-driven APIs.

use std::collections::HashMap;

use serde_json::Value;

use crate::spec::ir::{
    ChannelDef, DataModelSpec, EventApiSpec, FieldConstraints, FieldDef, ModelDef, OperationDef,
    StringFormat,
};
use crate::type_inference::Type;

/// Error type for AsyncAPI parsing
#[derive(Debug)]
pub enum AsyncApiError {
    /// Invalid JSON/YAML
    ParseError(String),
    /// Missing required field
    MissingField(String),
    /// Unsupported AsyncAPI version
    UnsupportedVersion(String),
    /// Invalid reference
    InvalidRef(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for AsyncApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsyncApiError::ParseError(s) => write!(f, "parse error: {}", s),
            AsyncApiError::MissingField(s) => write!(f, "missing field: {}", s),
            AsyncApiError::UnsupportedVersion(s) => write!(f, "unsupported version: {}", s),
            AsyncApiError::InvalidRef(s) => write!(f, "invalid reference: {}", s),
            AsyncApiError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for AsyncApiError {}

/// AsyncAPI parser
pub struct AsyncApiParser {
    /// Parsed components for reference resolution
    components: HashMap<String, Value>,
    /// AsyncAPI version (2.x or 3.x)
    version_major: u8,
}

impl AsyncApiParser {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            version_major: 2,
        }
    }

    /// Parse AsyncAPI from JSON string
    pub fn parse_json(&mut self, json: &str) -> Result<EventApiSpec, AsyncApiError> {
        let value: Value =
            serde_json::from_str(json).map_err(|e| AsyncApiError::ParseError(e.to_string()))?;
        self.parse_value(&value)
    }

    /// Parse AsyncAPI from YAML string
    pub fn parse_yaml(&mut self, yaml: &str) -> Result<EventApiSpec, AsyncApiError> {
        let value: Value =
            serde_yaml::from_str(yaml).map_err(|e| AsyncApiError::ParseError(e.to_string()))?;
        self.parse_value(&value)
    }

    /// Parse AsyncAPI from Value
    pub fn parse_value(&mut self, value: &Value) -> Result<EventApiSpec, AsyncApiError> {
        // Check version
        let version = value
            .get("asyncapi")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AsyncApiError::MissingField("asyncapi".into()))?;

        self.version_major = if version.starts_with("3.") {
            3
        } else if version.starts_with("2.") {
            2
        } else {
            return Err(AsyncApiError::UnsupportedVersion(version.to_string()));
        };

        // Extract components for reference resolution
        self.extract_components(value);

        // Parse info
        let info = value
            .get("info")
            .ok_or_else(|| AsyncApiError::MissingField("info".into()))?;

        let title = info
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AsyncApiError::MissingField("info.title".into()))?;

        let api_version = info
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AsyncApiError::MissingField("info.version".into()))?;

        let description = info
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Parse channels
        let channels = self.parse_channels(value)?;

        // Parse message schemas
        let messages = self.parse_message_schemas(value)?;

        Ok(EventApiSpec {
            title: title.to_string(),
            version: api_version.to_string(),
            description,
            channels,
            messages,
        })
    }

    /// Extract components for reference resolution
    fn extract_components(&mut self, value: &Value) {
        if let Some(components) = value.get("components") {
            // Schemas
            if let Some(schemas) = components.get("schemas").and_then(|s| s.as_object()) {
                for (name, schema) in schemas {
                    self.components
                        .insert(format!("#/components/schemas/{}", name), schema.clone());
                }
            }
            // Messages
            if let Some(messages) = components.get("messages").and_then(|m| m.as_object()) {
                for (name, message) in messages {
                    self.components
                        .insert(format!("#/components/messages/{}", name), message.clone());
                }
            }
        }
    }

    /// Parse channels
    fn parse_channels(&self, value: &Value) -> Result<Vec<ChannelDef>, AsyncApiError> {
        let channels_obj = value
            .get("channels")
            .and_then(|c| c.as_object())
            .ok_or_else(|| AsyncApiError::MissingField("channels".into()))?;

        let mut channels = Vec::new();

        for (name, channel_value) in channels_obj {
            let description = channel_value
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);

            let subscribe = self.parse_operation(channel_value, "subscribe")?;
            let publish = self.parse_operation(channel_value, "publish")?;

            channels.push(ChannelDef {
                name: name.clone(),
                description,
                subscribe,
                publish,
            });
        }

        Ok(channels)
    }

    /// Parse a channel operation (subscribe/publish)
    fn parse_operation(
        &self,
        channel: &Value,
        op_type: &str,
    ) -> Result<Option<OperationDef>, AsyncApiError> {
        let op = match channel.get(op_type) {
            Some(o) => o,
            None => return Ok(None),
        };

        let operation_id = op
            .get("operationId")
            .and_then(|v| v.as_str())
            .map(String::from);
        let summary = op.get("summary").and_then(|v| v.as_str()).map(String::from);
        let description = op
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Parse message
        let message = self.parse_operation_message(op)?;

        Ok(Some(OperationDef {
            operation_id,
            summary,
            description,
            message,
        }))
    }

    /// Parse operation message schema
    fn parse_operation_message(&self, op: &Value) -> Result<Type, AsyncApiError> {
        let message = match op.get("message") {
            Some(m) => m,
            None => return Ok(Type::Any),
        };

        // Handle $ref
        if let Some(ref_str) = message.get("$ref").and_then(|r| r.as_str()) {
            return self.resolve_message_ref(ref_str);
        }

        // Handle oneOf (multiple message types)
        if let Some(one_of) = message.get("oneOf").and_then(|o| o.as_array()) {
            let types: Vec<Type> = one_of
                .iter()
                .filter_map(|m| self.parse_message_schema(m).ok())
                .collect();
            return Ok(if types.len() == 1 {
                types.into_iter().next().unwrap()
            } else {
                Type::Union(types)
            });
        }

        self.parse_message_schema(message)
    }

    /// Parse a single message schema
    fn parse_message_schema(&self, message: &Value) -> Result<Type, AsyncApiError> {
        // Handle $ref
        if let Some(ref_str) = message.get("$ref").and_then(|r| r.as_str()) {
            return self.resolve_message_ref(ref_str);
        }

        // Get payload
        let payload = message.get("payload").unwrap_or(message);

        self.schema_to_type(payload)
    }

    /// Resolve a message reference
    fn resolve_message_ref(&self, ref_str: &str) -> Result<Type, AsyncApiError> {
        // Extract name from reference
        let name = ref_str
            .rsplit('/')
            .next()
            .ok_or_else(|| AsyncApiError::InvalidRef(ref_str.to_string()))?;

        // Check if we have it in components
        if let Some(component) = self.components.get(ref_str) {
            // If it's a message, get its payload
            if let Some(payload) = component.get("payload") {
                return self.schema_to_type(payload);
            }
            // Otherwise treat as schema
            return self.schema_to_type(component);
        }

        // Return as Instance type (will be defined elsewhere)
        Ok(Type::Instance {
            name: name.to_string(),
            module: None,
            type_args: vec![],
        })
    }

    /// Parse message schemas into DataModelSpec
    fn parse_message_schemas(&self, value: &Value) -> Result<DataModelSpec, AsyncApiError> {
        let mut spec = DataModelSpec::new();

        // Parse schemas from components
        if let Some(components) = value.get("components") {
            // Schemas
            if let Some(schemas) = components.get("schemas").and_then(|s| s.as_object()) {
                for (name, schema) in schemas {
                    if let Some(model) = self.parse_schema_as_model(name, schema)? {
                        spec.add_model(model);
                    }
                }
            }

            // Messages (extract their payloads as models)
            if let Some(messages) = components.get("messages").and_then(|m| m.as_object()) {
                for (name, message) in messages {
                    if let Some(payload) = message.get("payload") {
                        let model_name = format!("{}Message", name);
                        if let Some(model) = self.parse_schema_as_model(&model_name, payload)? {
                            spec.add_model(model);
                        }
                    }
                }
            }
        }

        Ok(spec)
    }

    /// Parse a schema as a model
    fn parse_schema_as_model(
        &self,
        name: &str,
        schema: &Value,
    ) -> Result<Option<ModelDef>, AsyncApiError> {
        let schema_type = schema.get("type").and_then(|t| t.as_str());

        // Only create models for object types
        if schema_type != Some("object") {
            return Ok(None);
        }

        let mut model = ModelDef {
            name: name.to_string(),
            description: schema
                .get("description")
                .and_then(|d| d.as_str())
                .map(String::from),
            fields: Vec::new(),
            methods: Vec::new(),
            extends: Vec::new(),
            type_params: Vec::new(),
            is_abstract: false,
            table_name: None,
            collection_name: None,
        };

        // Parse properties
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            let required_fields: Vec<&str> = schema
                .get("required")
                .and_then(|r| r.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();

            for (field_name, field_schema) in properties {
                let ty = self.schema_to_type(field_schema)?;
                let required = required_fields.contains(&field_name.as_str());

                let field_type = if required {
                    ty
                } else {
                    Type::Optional(Box::new(ty))
                };

                model.fields.push(FieldDef {
                    name: field_name.clone(),
                    ty: field_type,
                    default: field_schema.get("default").map(|v| v.to_string()),
                    description: field_schema
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(String::from),
                    required,
                    constraints: self.parse_constraints(field_schema),
                    column_name: None,
                    primary_key: false,
                    unique: false,
                    indexed: false,
                    foreign_key: None,
                    alias: None,
                });
            }
        }

        Ok(Some(model))
    }

    /// Convert JSON Schema to Type
    fn schema_to_type(&self, schema: &Value) -> Result<Type, AsyncApiError> {
        // Handle $ref
        if let Some(ref_str) = schema.get("$ref").and_then(|r| r.as_str()) {
            let name = ref_str.rsplit('/').next().unwrap_or(ref_str);
            return Ok(Type::Instance {
                name: name.to_string(),
                module: None,
                type_args: vec![],
            });
        }

        let schema_type = schema.get("type").and_then(|t| t.as_str());

        match schema_type {
            Some("string") => {
                // Check for format
                if let Some(format) = schema.get("format").and_then(|f| f.as_str()) {
                    let sf = match format {
                        "date-time" => Some(StringFormat::DateTime),
                        "date" => Some(StringFormat::Date),
                        "time" => Some(StringFormat::Time),
                        "email" => Some(StringFormat::Email),
                        "uuid" => Some(StringFormat::Uuid),
                        "uri" | "url" => Some(StringFormat::Uri),
                        _ => None,
                    };
                    if let Some(string_format) = sf {
                        return Ok(Type::Instance {
                            name: format!("Formatted<{:?}>", string_format),
                            module: None,
                            type_args: vec![],
                        });
                    }
                }
                // Check for enum
                if let Some(enum_values) = schema.get("enum").and_then(|e| e.as_array()) {
                    let values: Vec<Type> = enum_values
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| {
                            Type::Literal(crate::type_inference::LiteralValue::Str(s.to_string()))
                        })
                        .collect();
                    return Ok(Type::Union(values));
                }
                Ok(Type::Str)
            }
            Some("integer") => Ok(Type::Int),
            Some("number") => Ok(Type::Float),
            Some("boolean") => Ok(Type::Bool),
            Some("array") => {
                let items_type = if let Some(items) = schema.get("items") {
                    self.schema_to_type(items)?
                } else {
                    Type::Any
                };
                Ok(Type::List(Box::new(items_type)))
            }
            Some("object") => {
                // Check for additionalProperties (dict)
                if let Some(additional) = schema.get("additionalProperties") {
                    if additional.is_boolean() || additional.is_object() {
                        let value_type = if additional.is_object() {
                            self.schema_to_type(additional)?
                        } else {
                            Type::Any
                        };
                        return Ok(Type::Dict(Box::new(Type::Str), Box::new(value_type)));
                    }
                }
                // Check for inline object with properties
                if schema.get("properties").is_some() {
                    // This should be a named model, but for inline we use Any
                    return Ok(Type::Any);
                }
                Ok(Type::Dict(Box::new(Type::Str), Box::new(Type::Any)))
            }
            Some("null") => Ok(Type::None),
            None => {
                // Check for oneOf/anyOf/allOf
                if let Some(one_of) = schema.get("oneOf").and_then(|o| o.as_array()) {
                    let types: Vec<Type> = one_of
                        .iter()
                        .filter_map(|s| self.schema_to_type(s).ok())
                        .collect();
                    return Ok(Type::Union(types));
                }
                if let Some(any_of) = schema.get("anyOf").and_then(|o| o.as_array()) {
                    let types: Vec<Type> = any_of
                        .iter()
                        .filter_map(|s| self.schema_to_type(s).ok())
                        .collect();
                    return Ok(Type::Union(types));
                }
                Ok(Type::Any)
            }
            _ => Ok(Type::Any),
        }
    }

    /// Parse field constraints
    fn parse_constraints(&self, schema: &Value) -> FieldConstraints {
        FieldConstraints {
            min_length: schema
                .get("minLength")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize),
            max_length: schema
                .get("maxLength")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize),
            pattern: schema
                .get("pattern")
                .and_then(|v| v.as_str())
                .map(String::from),
            format: schema
                .get("format")
                .and_then(|v| v.as_str())
                .and_then(|f| match f {
                    "email" => Some(StringFormat::Email),
                    "uri" | "url" => Some(StringFormat::Uri),
                    "uuid" => Some(StringFormat::Uuid),
                    "date-time" => Some(StringFormat::DateTime),
                    "date" => Some(StringFormat::Date),
                    "time" => Some(StringFormat::Time),
                    _ => None,
                }),
            minimum: schema.get("minimum").and_then(|v| v.as_f64()),
            maximum: schema.get("maximum").and_then(|v| v.as_f64()),
            exclusive_minimum: schema.get("exclusiveMinimum").and_then(|v| v.as_f64()),
            exclusive_maximum: schema.get("exclusiveMaximum").and_then(|v| v.as_f64()),
            multiple_of: schema.get("multipleOf").and_then(|v| v.as_f64()),
            min_items: schema
                .get("minItems")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize),
            max_items: schema
                .get("maxItems")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize),
            unique_items: schema
                .get("uniqueItems")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }
}

impl Default for AsyncApiParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_asyncapi_2x() {
        let yaml = r#"
asyncapi: 2.6.0
info:
  title: User Service
  version: 1.0.0
  description: User events service

channels:
  user/created:
    description: User creation events
    subscribe:
      operationId: onUserCreated
      summary: Handle user created event
      message:
        payload:
          type: object
          properties:
            userId:
              type: string
            email:
              type: string
              format: email
            createdAt:
              type: string
              format: date-time

  user/updated:
    publish:
      operationId: publishUserUpdated
      message:
        $ref: '#/components/messages/UserUpdated'

components:
  messages:
    UserUpdated:
      payload:
        type: object
        properties:
          userId:
            type: string
          changes:
            type: object
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
        name:
          type: string
        email:
          type: string
"#;

        let mut parser = AsyncApiParser::new();
        let spec = parser.parse_yaml(yaml).unwrap();

        assert_eq!(spec.title, "User Service");
        assert_eq!(spec.version, "1.0.0");
        assert_eq!(spec.channels.len(), 2);

        // Check user/created channel
        let created_channel = spec
            .channels
            .iter()
            .find(|c| c.name == "user/created")
            .unwrap();
        assert!(created_channel.subscribe.is_some());
        let sub = created_channel.subscribe.as_ref().unwrap();
        assert_eq!(sub.operation_id, Some("onUserCreated".to_string()));

        // Check user/updated channel
        let updated_channel = spec
            .channels
            .iter()
            .find(|c| c.name == "user/updated")
            .unwrap();
        assert!(updated_channel.publish.is_some());

        // Check messages
        assert!(!spec.messages.models.is_empty());
    }

    #[test]
    fn test_parse_message_ref() {
        let yaml = r#"
asyncapi: 2.6.0
info:
  title: Order Service
  version: 1.0.0

channels:
  orders/placed:
    subscribe:
      message:
        $ref: '#/components/messages/OrderPlaced'

components:
  messages:
    OrderPlaced:
      payload:
        type: object
        properties:
          orderId:
            type: string
          total:
            type: number
"#;

        let mut parser = AsyncApiParser::new();
        let spec = parser.parse_yaml(yaml).unwrap();

        assert_eq!(spec.channels.len(), 1);
        let channel = &spec.channels[0];
        assert!(channel.subscribe.is_some());
    }

    #[test]
    fn test_parse_oneof_messages() {
        let yaml = r#"
asyncapi: 2.6.0
info:
  title: Notification Service
  version: 1.0.0

channels:
  notifications:
    subscribe:
      message:
        oneOf:
          - $ref: '#/components/messages/EmailNotification'
          - $ref: '#/components/messages/SmsNotification'

components:
  messages:
    EmailNotification:
      payload:
        type: object
        properties:
          email:
            type: string
          subject:
            type: string
    SmsNotification:
      payload:
        type: object
        properties:
          phone:
            type: string
          text:
            type: string
"#;

        let mut parser = AsyncApiParser::new();
        let spec = parser.parse_yaml(yaml).unwrap();

        let channel = &spec.channels[0];
        let sub = channel.subscribe.as_ref().unwrap();

        // Should be a Union type
        match &sub.message {
            Type::Union(types) => assert_eq!(types.len(), 2),
            _ => panic!("Expected Union type"),
        }
    }
}
