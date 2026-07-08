---
id: libs-compass-src-gen-rust-serde-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/gen/rust/serde.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/gen/rust/serde.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/gen/rust/serde.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SerdeGenerator` | libs/compass/src/gen/rust/serde.rs | struct | pub | 11 | pub struct SerdeGenerator; |
| `new` | libs/compass/src/gen/rust/serde.rs | function | pub | 14 | pub fn new() -> Self { |
| `User` | libs/compass/src/gen/rust/serde.rs | struct | pub | 310 | assert!(code.contains("pub struct User {")); |
| `Status` | libs/compass/src/gen/rust/serde.rs | enum | pub | 339 | assert!(result.contains("pub enum Status {")); |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! serde code generator
//!
//! Generates Rust structs with serde derives.

use crate::gen::rust::{format_to_rust_type, type_to_rust};
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{DataModelSpec, EnumDef, EnumValue, FieldDef, ModelDef, StringFormat};
use crate::type_inference::Type;

/// Serde code generator
pub struct SerdeGenerator;

impl SerdeGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a single struct
    fn generate_struct(&self, model: &ModelDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Doc comment
        if ctx.generate_docs {
            if let Some(desc) = &model.description {
                lines.push(format!("/// {}", desc));
            }
        }

        // Derive attributes
        lines.push("#[derive(Debug, Clone, Serialize, Deserialize)]".to_string());

        // Add serde attributes if needed
        let serde_attrs = self.get_struct_serde_attrs(model);
        if !serde_attrs.is_empty() {
            lines.push(format!("#[serde({})]", serde_attrs.join(", ")));
        }

        lines.push(format!("pub struct {} {{", model.name));

        // Generate fields
        for field in &model.fields {
            lines.push(self.generate_field(field, ctx));
        }

        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Get struct-level serde attributes
    fn get_struct_serde_attrs(&self, _model: &ModelDef) -> Vec<String> {
        let mut attrs = Vec::new();
        attrs.push("rename_all = \"camelCase\"".to_string());
        attrs
    }

    /// Generate a single field
    fn generate_field(&self, field: &FieldDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Doc comment
        if ctx.generate_docs {
            if let Some(desc) = &field.description {
                lines.push(format!("{}/// {}", ctx.indent, desc));
            }
        }

        // Serde attributes
        let serde_attrs = self.get_field_serde_attrs(field);
        if !serde_attrs.is_empty() {
            lines.push(format!(
                "{}#[serde({})]",
                ctx.indent,
                serde_attrs.join(", ")
            ));
        }

        // Field definition
        let type_str = self.get_field_type(field);
        lines.push(format!("{}pub {}: {},", ctx.indent, field.name, type_str));

        lines.join("\n")
    }

    /// Get field-level serde attributes
    fn get_field_serde_attrs(&self, field: &FieldDef) -> Vec<String> {
        let mut attrs = Vec::new();

        // Alias
        if let Some(alias) = &field.alias {
            attrs.push(format!("alias = \"{}\"", alias));
        }

        // Skip serializing None
        if !field.required {
            attrs.push("skip_serializing_if = \"Option::is_none\"".to_string());
        }

        // Default for optional fields
        if !field.required && field.default.is_none() {
            attrs.push("default".to_string());
        }

        attrs
    }

    /// Get Rust type for a field
    fn get_field_type(&self, field: &FieldDef) -> String {
        // Check for special formats
        if let Some(format) = &field.constraints.format {
            let format_type = format_to_rust_type(format);
            if format_type != "String" {
                return if field.required {
                    format_type.to_string()
                } else {
                    format!("Option<{}>", format_type)
                };
            }
        }

        let base_type = type_to_rust(&field.ty);

        if !field.required && !matches!(&field.ty, Type::Optional(_)) {
            format!("Option<{}>", base_type)
        } else {
            base_type
        }
    }

    /// Generate an enum
    fn generate_enum(&self, enum_def: &EnumDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Doc comment
        if ctx.generate_docs {
            if let Some(desc) = &enum_def.description {
                lines.push(format!("/// {}", desc));
            }
        }

        // Check if string enum
        let is_str_enum = enum_def
            .variants
            .iter()
            .all(|v| matches!(&v.value, Some(EnumValue::String(_)) | None));

        if is_str_enum {
            lines
                .push("#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]".to_string());
            lines.push(format!("pub enum {} {{", enum_def.name));

            for variant in &enum_def.variants {
                if let Some(EnumValue::String(s)) = &variant.value {
                    if s != &variant.name.to_lowercase() {
                        lines.push(format!("{}#[serde(rename = \"{}\")]", ctx.indent, s));
                    }
                }
                lines.push(format!("{}{},", ctx.indent, variant.name));
            }
        } else {
            lines.push(
                "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]"
                    .to_string(),
            );
            lines.push("#[repr(i64)]".to_string());
            lines.push(format!("pub enum {} {{", enum_def.name));

            for variant in &enum_def.variants {
                let value = match &variant.value {
                    Some(EnumValue::Int(i)) => i.to_string(),
                    _ => continue,
                };
                lines.push(format!("{}{} = {},", ctx.indent, variant.name, value));
            }
        }

        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generate imports
    fn generate_imports(&self, spec: &DataModelSpec) -> Vec<String> {
        let mut imports = vec!["use serde::{Serialize, Deserialize};".to_string()];

        let mut needs_hashmap = false;
        let mut needs_uuid = false;
        let mut needs_chrono = false;
        let mut needs_url = false;
        let mut needs_repr = false;

        for model in &spec.models {
            for field in &model.fields {
                if matches!(&field.ty, Type::Dict(_, _)) {
                    needs_hashmap = true;
                }

                if let Some(format) = &field.constraints.format {
                    match format {
                        StringFormat::Uuid => needs_uuid = true,
                        StringFormat::DateTime | StringFormat::Date | StringFormat::Time => {
                            needs_chrono = true
                        }
                        StringFormat::Uri | StringFormat::Url => needs_url = true,
                        _ => {}
                    }
                }
            }
        }

        for enum_def in &spec.enums {
            let has_int_values = enum_def
                .variants
                .iter()
                .any(|v| matches!(&v.value, Some(EnumValue::Int(_))));
            if has_int_values {
                needs_repr = true;
            }
        }

        if needs_hashmap {
            imports.push("use std::collections::HashMap;".to_string());
        }
        if needs_uuid {
            imports.push("use uuid::Uuid;".to_string());
        }
        if needs_chrono {
            imports.push("use chrono::{DateTime, NaiveDate, NaiveTime, Utc};".to_string());
        }
        if needs_url {
            imports.push("use url::Url;".to_string());
        }
        if needs_repr {
            imports.push("use serde_repr::{Serialize_repr, Deserialize_repr};".to_string());
        }

        imports
    }
}

impl Default for SerdeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for SerdeGenerator {
    fn name(&self) -> &str {
        "serde"
    }

    fn generate_data_models(
        &self,
        spec: &DataModelSpec,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let mut content_parts = Vec::new();

        // Generate enums first
        for enum_def in &spec.enums {
            content_parts.push(self.generate_enum(enum_def, ctx));
        }

        // Generate structs
        for model in &spec.models {
            content_parts.push(self.generate_struct(model, ctx));
        }

        let imports = self.generate_imports(spec);
        let content = content_parts.join("\n\n");

        let name = ctx
            .module_name
            .clone()
            .unwrap_or_else(|| "models".to_string());

        Ok(vec![
            GeneratedCode::new(name, content, Language::Rust).with_imports(imports)
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ir::EnumVariant;

    #[test]
    fn test_generate_struct() {
        let mut model = ModelDef::new("User");
        model.description = Some("User data".to_string());
        model.fields.push(FieldDef::new("id", Type::Int));
        model.fields.push(FieldDef::new("name", Type::Str));

        let mut email = FieldDef::new("email", Type::Str);
        email.required = false;
        model.fields.push(email);

        let spec = DataModelSpec {
            models: vec![model],
            enums: vec![],
            relationships: vec![],
        };

        let gen = SerdeGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_data_models(&spec, &ctx).unwrap();

        let code = &result[0].content;
        assert!(code.contains("pub struct User {"));
        assert!(code.contains("pub id: i64,"));
        assert!(code.contains("pub name: String,"));
        assert!(code.contains("pub email: Option<String>,"));
    }

    #[test]
    fn test_generate_enum() {
        let enum_def = EnumDef {
            name: "Status".to_string(),
            description: Some("Order status".to_string()),
            variants: vec![
                EnumVariant {
                    name: "Pending".to_string(),
                    value: Some(EnumValue::String("pending".to_string())),
                    description: None,
                },
                EnumVariant {
                    name: "Completed".to_string(),
                    value: Some(EnumValue::String("completed".to_string())),
                    description: None,
                },
            ],
        };

        let gen = SerdeGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_enum(&enum_def, &ctx);

        assert!(result.contains("pub enum Status {"));
        assert!(result.contains("Pending,"));
        assert!(result.contains("Completed,"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/gen/rust/serde.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/gen/rust/serde.rs` captured during libs codegen standardization.
```
