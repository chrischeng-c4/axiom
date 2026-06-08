---
id: sdd-td-ast-entities-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# TD AST Entity Enumeration Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/td_ast/entities.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `EntityRef` | projects/agentic-workflow/src/td_ast/entities.rs | struct | pub | 27 |  |
| `new` | projects/agentic-workflow/src/td_ast/entities.rs | function | pub | 39 | new(kind: impl Into<String>, id: impl Into<String>) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/td_ast/entities.rs -->
```rust
//! Per-`TypedBody`-variant entity enumeration.
//!
//! Walks the native structure of each parsed body (JSON Schema definitions,
//! Mermaid state machine nodes, OpenRPC methods, etc.) and surfaces a flat
//! list of `EntityRef`s for downstream tooling (entity graph, drift audit,
//! cross-spec link checking).
//!
//! @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::payloads::{
    AsyncApiPayload, CliManifestPayload, ConfigManifestPayload, JsonSchemaPayload, OpenApiPayload,
    OpenRpcPayload,
};
use super::types::{MermaidPlusPayload, TypedBody};

/// One entity reference produced by walking a `TypedBody`. `kind` names the
/// schema family (e.g. `"json-schema:definition"`, `"mermaid:node"`,
/// `"openrpc:method"`); `id` is the entity identifier within that family.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef {
    /// Family-qualified kind, e.g. `"json-schema:definition"`.
    pub kind: String,
    /// Identifier within the family, e.g. a JSON Schema definition name.
    pub id: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/entities.md#source
impl EntityRef {
    /// Construct a fresh `EntityRef`.
    ///
    /// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
    pub fn new(kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
        }
    }
}

/// Trait implemented by every `TypedBody` variant: walks its native structure
/// and returns the entities it declares (R5).
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
pub trait SectionEntities {
    /// Enumerate the entities declared by this body. Order is structural.
    fn entities(&self) -> Vec<EntityRef>;
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#trait_impl
impl SectionEntities for TypedBody {
    fn entities(&self) -> Vec<EntityRef> {
        match self {
            TypedBody::MermaidPlus(p) => mermaid_entities(p),
            TypedBody::JsonSchema(p) => json_schema_entities(p),
            TypedBody::OpenRpc(p) => openrpc_entities(p),
            TypedBody::OpenApi(p) => openapi_entities(p),
            TypedBody::AsyncApi(p) => asyncapi_entities(p),
            TypedBody::CliManifest(p) => cli_entities(p),
            TypedBody::ConfigManifest(p) => config_entities(p),
            TypedBody::Markdown(_) => Vec::new(),
            TypedBody::Placeholder => Vec::new(),
            TypedBody::Unsupported(_) => Vec::new(),
        }
    }
}

/// Walk a Mermaid Plus payload's frontmatter and surface declared nodes.
///
/// Recognised shapes:
///  - `nodes:` map (state-machine, logic, interaction)
///  - `types:` map (dependency)
///  - `tables:` map (db-model)
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
fn mermaid_entities(p: &MermaidPlusPayload) -> Vec<EntityRef> {
    let mut out = Vec::new();
    let fm = &p.frontmatter;

    if let Some(id) = fm.get("id").and_then(Value::as_str) {
        out.push(EntityRef::new("mermaid:diagram", id));
    }

    for (key, kind) in [
        ("nodes", "mermaid:node"),
        ("types", "mermaid:type"),
        ("tables", "mermaid:table"),
        ("methods", "mermaid:method"),
    ] {
        if let Some(map) = fm.get(key).and_then(Value::as_mapping) {
            for k in map.keys() {
                if let Some(name) = k.as_str() {
                    out.push(EntityRef::new(kind, name));
                }
            }
        }
    }
    out
}

/// Walk a typed `JsonSchemaPayload` and surface `definitions.*` and `$defs.*`.
///
/// Stage 1B: walks typed `BTreeMap<String, PayloadTypeDef>` directly — no
/// `Value::as_mapping()` lookups (R3). Falls back to `extra` only for
/// Changes sections that round-trip through `JsonSchemaPayload` and use
/// non-`definitions` shapes.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn json_schema_entities(p: &JsonSchemaPayload) -> Vec<EntityRef> {
    let mut out = Vec::new();
    for name in p.definitions.keys() {
        out.push(EntityRef::new("json-schema:definition", name));
    }
    for name in p.defs.keys() {
        out.push(EntityRef::new("json-schema:definition", name));
    }
    // Changes sections route through JsonSchemaPayload and put their entries
    // under a top-level `changes:` key in `extra`. Surface them as
    // changes:target so the orphan-changes-target validator can find them.
    if let Some(changes) = p.extra.get("changes").and_then(Value::as_sequence) {
        for entry in changes {
            if let Some(path) = entry.get("path").and_then(Value::as_str) {
                out.push(EntityRef::new("changes:target", path));
            }
        }
    }
    out
}

/// Walk a typed `OpenRpcPayload` and surface declared methods +
/// `components.schemas`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn openrpc_entities(p: &OpenRpcPayload) -> Vec<EntityRef> {
    let mut out = Vec::new();
    for m in &p.methods {
        out.push(EntityRef::new("openrpc:method", &m.name));
    }
    for name in p.components_schemas.keys() {
        out.push(EntityRef::new("openrpc:schema", name));
    }
    out
}

/// Walk a typed `OpenApiPayload` and surface declared paths +
/// `components.schemas`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn openapi_entities(p: &OpenApiPayload) -> Vec<EntityRef> {
    let mut out = Vec::new();
    for name in p.paths.keys() {
        out.push(EntityRef::new("openapi:path", name));
    }
    for name in p.components_schemas.keys() {
        out.push(EntityRef::new("openapi:schema", name));
    }
    out
}

/// Walk a typed `AsyncApiPayload` and surface declared channels +
/// `components.schemas`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn asyncapi_entities(p: &AsyncApiPayload) -> Vec<EntityRef> {
    let mut out = Vec::new();
    for name in p.channels.keys() {
        out.push(EntityRef::new("asyncapi:channel", name));
    }
    for name in p.components_schemas.keys() {
        out.push(EntityRef::new("asyncapi:schema", name));
    }
    out
}

/// Walk a typed `CliManifestPayload` and surface declared commands.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn cli_entities(p: &CliManifestPayload) -> Vec<EntityRef> {
    let mut out = Vec::new();
    for cmd in &p.commands {
        out.push(EntityRef::new("cli:command", &cmd.name));
    }
    out
}

/// Walk a typed `ConfigManifestPayload` and surface declared config keys.
///
/// Stage 1B: prefers the typed `keys:` sequence; falls back to the raw
/// `config:` mapping (still carried as `Option<Value>` to support the
/// alternate spec shape).
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn config_entities(p: &ConfigManifestPayload) -> Vec<EntityRef> {
    let mut out = Vec::new();
    for key in &p.keys {
        out.push(EntityRef::new("config:key", &key.name));
    }
    if let Some(cfg) = &p.config {
        if let Some(map) = cfg.as_mapping() {
            for k in map.keys() {
                if let Some(name) = k.as_str() {
                    out.push(EntityRef::new("config:key", name));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value;

    #[test]
    fn json_schema_definitions_are_listed() {
        let p: JsonSchemaPayload = serde_yaml::from_str(
            "definitions:\n  Foo: { type: object }\n  Bar: { type: string }\n",
        )
        .unwrap();
        let body = TypedBody::JsonSchema(p);
        let names: Vec<_> = body.entities().into_iter().map(|e| e.id).collect();
        assert!(names.contains(&"Foo".to_string()));
        assert!(names.contains(&"Bar".to_string()));
    }

    #[test]
    fn mermaid_nodes_are_listed() {
        let fm: Value = serde_yaml::from_str(
            "id: my-sm\nnodes:\n  start: { kind: start }\n  done: { kind: terminal }\n",
        )
        .unwrap();
        let payload = MermaidPlusPayload {
            frontmatter: fm,
            frontmatter_raw: String::new(),
            rendered_body: String::new(),
        };
        let body = TypedBody::MermaidPlus(payload);
        let entities = body.entities();
        let kinds: Vec<_> = entities.iter().map(|e| e.kind.as_str()).collect();
        assert!(kinds.contains(&"mermaid:diagram"));
        assert!(kinds.contains(&"mermaid:node"));
    }

    #[test]
    fn placeholder_yields_no_entities() {
        assert!(TypedBody::Placeholder.entities().is_empty());
    }

    #[test]
    fn openrpc_methods_are_listed() {
        let p =
            OpenRpcPayload::from_yaml_str("methods:\n  - name: tools/list\n  - name: tools/call\n")
                .unwrap();
        let body = TypedBody::OpenRpc(p);
        let names: Vec<_> = body.entities().into_iter().map(|e| e.id).collect();
        assert_eq!(names, vec!["tools/list", "tools/call"]);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/td_ast/entities.rs
    action: modify
    section: source
    impl_mode: codegen
    description: >
      Promote the entity enumeration module into a single source template,
      preserving EntityRef, SectionEntities, typed-payload walkers, and tests
      while removing nested HANDWRITE and partial CODEGEN markers.
```
