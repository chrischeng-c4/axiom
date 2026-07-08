---
id: libs-compass-src-gen-python-nebula-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/gen/python/nebula.rs`.
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

# Standardized libs/compass/src/gen/python/nebula.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/gen/python/nebula.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `NebulaGenerator` | libs/compass/src/gen/python/nebula.rs | struct | pub | 11 | pub struct NebulaGenerator; |
| `new` | libs/compass/src/gen/python/nebula.rs | function | pub | 14 | pub fn new() -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! cclab.nebula code generator
//!
//! Generates MongoDB Document classes using cclab.nebula.

use crate::gen::python::type_to_python;
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{DataModelSpec, FieldDef, ModelDef, StringFormat};
use crate::type_inference::Type;

/// Nebula (MongoDB ORM) code generator
pub struct NebulaGenerator;

impl NebulaGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a single document class
    fn generate_document(&self, model: &ModelDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Get collection name
        let collection_name = model
            .collection_name
            .clone()
            .unwrap_or_else(|| to_snake_case(&model.name) + "s");

        // Class definition
        if ctx.generate_docs {
            if let Some(desc) = &model.description {
                lines.push(format!("class {}(Document):", model.name));
                lines.push(format!("{}\"\"\"{}\"\"\"", ctx.indent, desc));
            } else {
                lines.push(format!("class {}(Document):", model.name));
            }
        } else {
            lines.push(format!("class {}(Document):", model.name));
        }

        // Settings class
        lines.push(format!("{}class Settings:", ctx.indent));
        lines.push(format!(
            "{}{}name = \"{}\"",
            ctx.indent, ctx.indent, collection_name
        ));

        // Add indexes if any fields are indexed
        let indexed_fields: Vec<_> = model
            .fields
            .iter()
            .filter(|f| f.indexed || f.unique)
            .collect();

        if !indexed_fields.is_empty() {
            lines.push(format!("{}{}indexes = [", ctx.indent, ctx.indent));
            for field in indexed_fields {
                let unique_str = if field.unique { ", unique=True" } else { "" };
                lines.push(format!(
                    "{}{}{}Index(\"{}\"){}",
                    ctx.indent, ctx.indent, ctx.indent, field.name, unique_str
                ));
            }
            lines.push(format!("{}{}]", ctx.indent, ctx.indent));
        }

        lines.push(String::new());

        // Generate fields
        // MongoDB documents always have _id, but we skip if there's a field named 'id'
        let has_id_field = model
            .fields
            .iter()
            .any(|f| f.name == "id" || f.name == "_id");
        if !has_id_field {
            lines.push(format!("{}id: Optional[ObjectId] = None", ctx.indent));
        }

        for field in &model.fields {
            if field.name != "_id" {
                lines.push(self.generate_field(field, ctx));
            }
        }

        lines.join("\n")
    }

    /// Generate a single field
    fn generate_field(&self, field: &FieldDef, ctx: &GenContext) -> String {
        let type_str = self.get_field_type(field);

        // Determine if we need a Field() call
        let needs_field_call = field.alias.is_some()
            || field.description.is_some()
            || field.default.is_some()
            || !field.required;

        if needs_field_call {
            let args = self.get_field_args(field, ctx);
            format!(
                "{}{}: {} = Field({})",
                ctx.indent,
                field.name,
                type_str,
                args.join(", ")
            )
        } else {
            format!("{}{}: {}", ctx.indent, field.name, type_str)
        }
    }

    /// Get Python type for a field
    fn get_field_type(&self, field: &FieldDef) -> String {
        // Check for special MongoDB types
        if let Some(format) = &field.constraints.format {
            let format_type = match format {
                StringFormat::Uuid => "UUID",
                StringFormat::DateTime => "datetime",
                StringFormat::Date => "date",
                _ => return self.default_type(field),
            };

            return if field.required {
                format_type.to_string()
            } else {
                format!("Optional[{}]", format_type)
            };
        }

        self.default_type(field)
    }

    /// Get default Python type
    fn default_type(&self, field: &FieldDef) -> String {
        let base = type_to_python(&field.ty);

        if !field.required && !matches!(&field.ty, Type::Optional(_)) {
            format!("Optional[{}]", base)
        } else {
            base
        }
    }

    /// Get Field() arguments
    fn get_field_args(&self, field: &FieldDef, ctx: &GenContext) -> Vec<String> {
        let mut args = Vec::new();

        // Default value
        if let Some(default) = &field.default {
            args.push(format!("default={}", default));
        } else if !field.required {
            args.push("default=None".to_string());
        }

        // Alias (MongoDB field name)
        if let Some(alias) = &field.alias {
            args.push(format!("alias=\"{}\"", alias));
        }

        // Description
        if ctx.generate_docs {
            if let Some(desc) = &field.description {
                args.push(format!("description=\"{}\"", escape_string(desc)));
            }
        }

        args
    }

    /// Generate imports
    fn generate_imports(&self, spec: &DataModelSpec) -> Vec<String> {
        let mut imports = vec![
            "from typing import Optional, List, Dict, Any".to_string(),
            "from cclab.nebula import Document, Field".to_string(),
            "from bson import ObjectId".to_string(),
        ];

        let mut needs_uuid = false;
        let mut needs_datetime = false;
        let mut needs_index = false;

        for model in &spec.models {
            for field in &model.fields {
                if field.indexed || field.unique {
                    needs_index = true;
                }

                if let Some(format) = &field.constraints.format {
                    match format {
                        StringFormat::Uuid => needs_uuid = true,
                        StringFormat::DateTime | StringFormat::Date | StringFormat::Time => {
                            needs_datetime = true
                        }
                        _ => {}
                    }
                }
            }
        }

        if needs_index {
            imports.push("from cclab.nebula import Index".to_string());
        }
        if needs_uuid {
            imports.push("from uuid import UUID".to_string());
        }
        if needs_datetime {
            imports.push("from datetime import datetime, date".to_string());
        }

        imports
    }
}

impl Default for NebulaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for NebulaGenerator {
    fn name(&self) -> &str {
        "nebula"
    }

    fn generate_data_models(
        &self,
        spec: &DataModelSpec,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let mut content_parts = Vec::new();

        // Generate documents
        for model in &spec.models {
            content_parts.push(self.generate_document(model, ctx));
        }

        let imports = self.generate_imports(spec);
        let content = content_parts.join("\n\n\n");

        let name = ctx
            .module_name
            .clone()
            .unwrap_or_else(|| "documents".to_string());

        Ok(vec![
            GeneratedCode::new(name, content, Language::Python).with_imports(imports)
        ])
    }
}

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Escape string for Python
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_document() {
        let mut model = ModelDef::new("User");
        model.description = Some("User document".to_string());
        model.collection_name = Some("users".to_string());

        model.fields.push(FieldDef::new("name", Type::Str));

        let mut email_field = FieldDef::new("email", Type::Str);
        email_field.unique = true;
        email_field.indexed = true;
        model.fields.push(email_field);

        let mut age_field = FieldDef::new("age", Type::Int);
        age_field.required = false;
        model.fields.push(age_field);

        let spec = DataModelSpec {
            models: vec![model],
            enums: vec![],
            relationships: vec![],
        };

        let gen = NebulaGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_data_models(&spec, &ctx).unwrap();

        let code = &result[0].content;
        assert!(code.contains("class User(Document):"));
        assert!(code.contains("name = \"users\""));
        assert!(code.contains("id: Optional[ObjectId]"));
        assert!(code.contains("name: str"));
        assert!(code.contains("email: str"));
        assert!(code.contains("age: Optional[int]"));
    }

    #[test]
    fn test_generate_with_index() {
        let mut model = ModelDef::new("Product");

        let mut sku_field = FieldDef::new("sku", Type::Str);
        sku_field.unique = true;
        model.fields.push(sku_field);

        let spec = DataModelSpec {
            models: vec![model],
            enums: vec![],
            relationships: vec![],
        };

        let gen = NebulaGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_data_models(&spec, &ctx).unwrap();

        let code = &result[0].content;
        assert!(code.contains("indexes = ["));
        assert!(code.contains("Index(\"sku\")"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/gen/python/nebula.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/gen/python/nebula.rs` captured during libs codegen standardization.
```
