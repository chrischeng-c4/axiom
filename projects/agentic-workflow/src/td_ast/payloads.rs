// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#source
// CODEGEN-BEGIN
//! Typed payload structs for `TypedBody` variants — Stage 1B of the
//! TD AST migration.
//!
//! Replaces the six opaque `serde_yaml::Value` carriers in [`super::types::TypedBody`]
//! (`JsonSchema`, `OpenRpc`, `OpenApi`, `AsyncApi`, `CliManifest`,
//! `ConfigManifest`) with strongly-typed Rust structs that mirror the
//! canonical schema family each section type uses.
//!
//! Each payload exposes the fields that downstream consumers
//! (entity walkers, validators, query API) need so they can stop
//! falling back to `Value::as_mapping()` lookups.
//!
//! @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

/// JSON Schema 2020-12 document body. Preserves `definitions` / `$defs`
/// as typed maps so entity walkers can iterate keys directly.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsonSchemaPayload {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(rename = "$id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub definitions: BTreeMap<String, PayloadTypeDef>,
    #[serde(rename = "$defs", default, skip_serializing_if = "BTreeMap::is_empty")]
    pub defs: BTreeMap<String, PayloadTypeDef>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// OpenRPC 1.3 document body. `methods[].name` is the precise replacement
/// for the heuristic OpenRPC walk in `entities.rs`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenRpcPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openrpc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<RpcMethod>,
    /// `components.schemas` flattened into a top-level map. Populated by
    /// `OpenRpcPayload::from_value` (custom deserialiser path) since serde
    /// itself cannot project nested keys inline.
    #[serde(skip)]
    pub components_schemas: BTreeMap<String, PayloadTypeDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Value>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// OpenAPI 3.1 document body.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenApiPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openapi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info: Option<Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub paths: BTreeMap<String, OpenApiPathItem>,
    #[serde(skip)]
    pub components_schemas: BTreeMap<String, PayloadTypeDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Value>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// AsyncAPI 2.6 document body.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsyncApiPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asyncapi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info: Option<Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub channels: BTreeMap<String, AsyncApiChannel>,
    #[serde(skip)]
    pub components_schemas: BTreeMap<String, PayloadTypeDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Value>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// CLI manifest document body. `commands[].name` powers the typed CLI walk.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliManifestPayload {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<CliCommandDef>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// Config manifest document body. `keys[].name` powers the typed config walk.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigManifestPayload {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<ConfigKeyDef>,
    /// Some specs use `config:` mapping form instead of `keys:` sequence —
    /// captured as raw value so `config_entities` can walk either shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// Shared helper for a named type definition (Schema/OpenRPC/OpenAPI/AsyncAPI
/// components). Only `extra` plus the structured fields below are consulted
/// by entity walks — author-supplied keys round-trip via `extra`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PayloadTypeDef {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// One method declaration inside an OpenRPC document.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMethod {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<RpcParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// One param declaration on an `RpcMethod`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcParam {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// One entry in OpenAPI `paths`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenApiPathItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub get: Option<OpenApiOperation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<OpenApiOperation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub put: Option<OpenApiOperation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete: Option<OpenApiOperation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch: Option<OpenApiOperation>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// One HTTP-verb operation on an `OpenApiPathItem`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenApiOperation {
    #[serde(
        rename = "operationId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub operation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// One channel entry in an AsyncAPI document.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsyncApiChannel {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// One command declaration inside a CLI manifest. Distinct from the
/// `CliCommand` type in `cli_subcommand.rs` — Stage 2 unifies them.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommandDef {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<CliArgDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<CliArgDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subcommands: Vec<CliCommandDef>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// One arg or flag declaration on a `CliCommandDef`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliArgDef {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// One key declaration inside a config manifest.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigKeyDef {
    pub name: String,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "value_is_empty_mapping")]
    pub extra: Value,
}

/// Discriminant of `TdParseError`. Stage 1B introduces `TypedPayloadParse`
/// so callers can distinguish typed-payload deserialisation failures from
/// frontmatter / IO failures.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TdParseErrorKind {
    /// Frontmatter YAML failed to parse or had no closing marker.
    Frontmatter,
    /// A SectionKind dispatch attempted to parse the section's fenced block
    /// as its expected typed payload and the deserialiser returned an error.
    /// `TdParseError::section_type` carries the `expected_type` from R2.
    TypedPayloadParse,
    /// Catch-all for non-payload parse failures (file IO, etc.).
    Generic,
}

/// True for empty / null `serde_yaml::Value`s — used by the `flatten`
/// catch-all field's `skip_serializing_if`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn value_is_empty_mapping(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::Mapping(m) => m.is_empty(),
        _ => false,
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#source
impl OpenRpcPayload {
    /// Parse from raw YAML, lifting `components.schemas` into the typed
    /// `components_schemas` field. Used by `parse.rs` instead of plain
    /// `serde_yaml::from_str`.
    ///
    /// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
    pub fn from_yaml_str(s: &str) -> Result<Self, serde_yaml::Error> {
        let mut p: Self = serde_yaml::from_str(s)?;
        p.components_schemas = extract_components_schemas(p.components.as_ref());
        Ok(p)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#source
impl OpenApiPayload {
    /// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
    pub fn from_yaml_str(s: &str) -> Result<Self, serde_yaml::Error> {
        let mut p: Self = serde_yaml::from_str(s)?;
        p.components_schemas = extract_components_schemas(p.components.as_ref());
        Ok(p)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#source
impl AsyncApiPayload {
    /// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
    pub fn from_yaml_str(s: &str) -> Result<Self, serde_yaml::Error> {
        let mut p: Self = serde_yaml::from_str(s)?;
        p.components_schemas = extract_components_schemas(p.components.as_ref());
        Ok(p)
    }
}

/// Project `components.schemas` from a raw `Value` into a typed map.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn extract_components_schemas(components: Option<&Value>) -> BTreeMap<String, PayloadTypeDef> {
    let mut out = BTreeMap::new();
    let Some(c) = components else { return out };
    let Some(schemas) = c.get("schemas").and_then(Value::as_mapping) else {
        return out;
    };
    for (k, v) in schemas {
        let Some(name) = k.as_str() else { continue };
        if let Ok(def) = serde_yaml::from_value::<PayloadTypeDef>(v.clone()) {
            out.insert(name.to_string(), def);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_schema_payload_roundtrip() {
        let raw = "$schema: \"https://json-schema.org/draft/2020-12/schema\"\n$id: example#schema\ndefinitions:\n  Foo:\n    type: object\n    description: foo\n  Bar:\n    type: string\n";
        let p: JsonSchemaPayload = serde_yaml::from_str(raw).unwrap();
        assert_eq!(p.id.as_deref(), Some("example#schema"));
        assert_eq!(p.definitions.len(), 2);
        assert!(p.definitions.contains_key("Foo"));
        let back = serde_yaml::to_string(&p).unwrap();
        let p2: JsonSchemaPayload = serde_yaml::from_str(&back).unwrap();
        assert_eq!(p2.definitions.len(), 2);
        assert_eq!(p2.definitions["Foo"].ty.as_deref(), Some("object"));
    }

    #[test]
    fn openrpc_payload_roundtrip() {
        let raw = "openrpc: \"1.3.0\"\ninfo:\n  title: example\n  version: 1.0\nmethods:\n  - name: tools/list\n    summary: list\n  - name: tools/call\n    params:\n      - name: tool\n        required: true\ncomponents:\n  schemas:\n    Tool:\n      type: object\n";
        let p = OpenRpcPayload::from_yaml_str(raw).unwrap();
        assert_eq!(p.openrpc.as_deref(), Some("1.3.0"));
        assert_eq!(p.methods.len(), 2);
        assert_eq!(p.methods[0].name, "tools/list");
        assert_eq!(p.methods[1].params[0].name, "tool");
        assert_eq!(p.components_schemas.len(), 1);
        assert!(p.components_schemas.contains_key("Tool"));
    }

    #[test]
    fn openapi_payload_roundtrip() {
        let raw = "openapi: \"3.1.0\"\npaths:\n  /users:\n    get:\n      operationId: listUsers\n      summary: list users\n    post:\n      operationId: createUser\ncomponents:\n  schemas:\n    User:\n      type: object\n";
        let p = OpenApiPayload::from_yaml_str(raw).unwrap();
        assert_eq!(p.openapi.as_deref(), Some("3.1.0"));
        let users = &p.paths["/users"];
        assert_eq!(
            users.get.as_ref().unwrap().operation_id.as_deref(),
            Some("listUsers")
        );
        assert_eq!(
            users.post.as_ref().unwrap().operation_id.as_deref(),
            Some("createUser")
        );
        assert!(p.components_schemas.contains_key("User"));
    }

    #[test]
    fn asyncapi_payload_roundtrip() {
        let raw = "asyncapi: \"2.6.0\"\nchannels:\n  user/created:\n    description: emitted on user creation\n  user/deleted:\n    description: emitted on user deletion\ncomponents:\n  schemas:\n    UserEvent:\n      type: object\n";
        let p = AsyncApiPayload::from_yaml_str(raw).unwrap();
        assert_eq!(p.asyncapi.as_deref(), Some("2.6.0"));
        assert_eq!(p.channels.len(), 2);
        assert!(p.channels.contains_key("user/created"));
        assert!(p.components_schemas.contains_key("UserEvent"));
    }

    #[test]
    fn cli_manifest_payload_roundtrip() {
        let raw = "commands:\n  - name: build\n    description: build the project\n    flags:\n      - name: release\n        type: bool\n  - name: test\n    args:\n      - name: filter\n        required: false\n";
        let p: CliManifestPayload = serde_yaml::from_str(raw).unwrap();
        assert_eq!(p.commands.len(), 2);
        assert_eq!(p.commands[0].name, "build");
        assert_eq!(p.commands[0].flags[0].name, "release");
        assert_eq!(p.commands[1].args[0].name, "filter");
    }

    #[test]
    fn config_manifest_payload_roundtrip() {
        let raw = "keys:\n  - name: log_level\n    type: string\n    default: info\n    env: LOG_LEVEL\n  - name: max_workers\n    type: integer\n    default: 4\n";
        let p: ConfigManifestPayload = serde_yaml::from_str(raw).unwrap();
        assert_eq!(p.keys.len(), 2);
        assert_eq!(p.keys[0].name, "log_level");
        assert_eq!(p.keys[0].env.as_deref(), Some("LOG_LEVEL"));
    }

    #[test]
    fn json_schema_payload_preserves_extra_keys() {
        let raw = "definitions:\n  Foo:\n    type: object\nrequirementsTraceMatrix:\n  R1: Foo\n";
        let p: JsonSchemaPayload = serde_yaml::from_str(raw).unwrap();
        assert!(matches!(p.extra, Value::Mapping(_)));
        let back = serde_yaml::to_string(&p).unwrap();
        // Extra key survives the round trip.
        assert!(back.contains("requirementsTraceMatrix"));
    }
}

// CODEGEN-END
