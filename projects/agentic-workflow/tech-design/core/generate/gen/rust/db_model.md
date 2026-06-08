---
id: sdd-generate-gen-rust-db-model
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# DB Model Structural Generator

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/db_model.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DbModelGenOutput` | projects/agentic-workflow/src/generate/gen/rust/db_model.rs | struct | pub | 9 |  |
| `generate_db_model` | projects/agentic-workflow/src/generate/gen/rust/db_model.rs | function | pub | 20 | generate_db_model(erd_yaml: &Value, config: &RustConfig) -> DbModelGenOutput |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/db_model.rs -->
```rust
//! DB-model structural generator.
//!
//! Produces Rust struct with sqlx attributes from ERD YAML frontmatter.
//! 100% deterministic coverage for db-model section types.

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R4

use crate::generate::types::{parse_abstract_type, RustConfig, RustTypeTranslator, TypeTranslator};
use serde_yaml::Value;

/// Output from DB-model code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/db_model_types.md#schema
#[derive(Debug, Clone)]
pub struct DbModelGenOutput {
    /// The generated Rust struct(s) with sqlx derives.
    pub code: String,
}

/// Generate Rust sqlx structs from ERD YAML.
///
/// Each entity in the ERD becomes a Rust struct with `#[derive(sqlx::FromRow)]`.
/// Field types are translated via the abstract type system.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R4
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R6
pub fn generate_db_model(erd_yaml: &Value, config: &RustConfig) -> DbModelGenOutput {
    let config = config.merge_overrides(erd_yaml);
    let translator = RustTypeTranslator;
    let vis = config.vis_prefix();

    let entities = erd_yaml
        .get("entities")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();

    let mut all_lines = Vec::new();

    for (entity_key, entity_value) in &entities {
        let entity_name = entity_key.as_str().unwrap_or("Entity");
        let struct_name = to_pascal_case(entity_name);

        let fields = entity_value
            .get("fields")
            .or_else(|| entity_value.get("attributes"))
            .and_then(|v| v.as_sequence())
            .cloned()
            .unwrap_or_default();

        // Doc comment
        if let Some(desc) = entity_value.get("description").and_then(|v| v.as_str()) {
            all_lines.push(format!("/// {}", desc));
        }

        all_lines
            .push("#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]".to_string());
        all_lines.push(format!("#[sqlx(rename_all = \"snake_case\")]"));
        all_lines.push(format!("{}struct {} {{", vis, struct_name));

        for field in &fields {
            let field_name = field.get("name").and_then(|v| v.as_str()).unwrap_or("id");
            let field_type_str = field
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("string");
            let is_pk = field.get("pk").and_then(|v| v.as_bool()).unwrap_or(false);
            let nullable = field
                .get("nullable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Type mapping
            let rust_type = if let Ok(at) = parse_abstract_type(field_type_str) {
                translator.translate(&at)
            } else {
                match field_type_str {
                    "uuid" => "uuid::Uuid".to_string(),
                    "timestamp" | "datetime" => "chrono::DateTime<chrono::Utc>".to_string(),
                    "jsonb" | "json" => "serde_json::Value".to_string(),
                    _ => "String".to_string(),
                }
            };

            let final_type = if nullable {
                format!("Option<{}>", rust_type)
            } else {
                rust_type
            };

            let rust_field_name = to_snake_case(field_name);

            if is_pk {
                all_lines.push("    #[sqlx(rename = \"id\")]".to_string());
            }
            if nullable {
                all_lines.push("    #[serde(default)]".to_string());
            }
            all_lines.push(format!("    {}{}: {},", vis, rust_field_name, final_type));
        }

        all_lines.push("}".to_string());
        all_lines.push(String::new());
    }

    DbModelGenOutput {
        code: all_lines.join("\n").trim_end().to_string(),
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else if c == '-' {
            result.push('_');
        } else {
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
    }
    result
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/db_model.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete canonical DB-model generator module.
```

# Reviews

