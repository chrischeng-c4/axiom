---
id: libs-compass-src-gen-python-shield-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/gen/python/shield.rs`.
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

# Standardized libs/compass/src/gen/python/shield.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/gen/python/shield.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ShieldGenerator` | libs/compass/src/gen/python/shield.rs | struct | pub | 10 | pub struct ShieldGenerator; |
| `new` | libs/compass/src/gen/python/shield.rs | function | pub | 13 | pub fn new() -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! cclab.shield code generator
//!
//! Generates BaseModel subclasses with Field constraints.

use crate::gen::python::{format_to_python_type, type_to_python};
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{DataModelSpec, EnumDef, FieldDef, ModelDef, StringFormat};

/// Shield code generator
pub struct ShieldGenerator;

impl ShieldGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a single model class
    fn generate_model(&self, model: &ModelDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Class docstring
        if ctx.generate_docs {
            if let Some(desc) = &model.description {
                lines.push(format!("class {}(BaseModel):", model.name));
                lines.push(format!("{}\"\"\"{}\"\"\"", ctx.indent, desc));
            } else {
                lines.push(format!("class {}(BaseModel):", model.name));
            }
        } else {
            lines.push(format!("class {}(BaseModel):", model.name));
        }

        // Generate fields
        if model.fields.is_empty() {
            lines.push(format!("{}pass", ctx.indent));
        } else {
            for field in &model.fields {
                lines.push(self.generate_field(field, ctx));
            }
        }

        lines.join("\n")
    }

    /// Generate a single field
    fn generate_field(&self, field: &FieldDef, ctx: &GenContext) -> String {
        let type_str = self.get_field_type(field);
        let field_args = self.get_field_args(field, ctx);

        if field_args.is_empty() {
            // Simple field without constraints
            if !field.required && field.default.is_none() {
                format!("{}{}: {} = None", ctx.indent, field.name, type_str)
            } else if let Some(default) = &field.default {
                format!("{}{}: {} = {}", ctx.indent, field.name, type_str, default)
            } else {
                format!("{}{}: {}", ctx.indent, field.name, type_str)
            }
        } else {
            // Field with constraints
            format!(
                "{}{}: {} = Field({})",
                ctx.indent,
                field.name,
                type_str,
                field_args.join(", ")
            )
        }
    }

    /// Get the Python type for a field
    fn get_field_type(&self, field: &FieldDef) -> String {
        // Check for special format types
        if let Some(format) = &field.constraints.format {
            let format_type = format_to_python_type(format);
            if format_type != "str" {
                if field.required {
                    return format_type.to_string();
                } else {
                    return format!("Optional[{}]", format_type);
                }
            }
        }

        let base_type = type_to_python(&field.ty);

        if !field.required && !matches!(&field.ty, crate::type_inference::Type::Optional(_)) {
            format!("Optional[{}]", base_type)
        } else {
            base_type
        }
    }

    /// Get Field() arguments
    fn get_field_args(&self, field: &FieldDef, ctx: &GenContext) -> Vec<String> {
        let mut args = Vec::new();
        let c = &field.constraints;

        // Default value
        if let Some(default) = &field.default {
            args.push(format!("default={}", default));
        } else if !field.required {
            args.push("default=None".to_string());
        }

        // String constraints
        if let Some(min) = c.min_length {
            args.push(format!("min_length={}", min));
        }
        if let Some(max) = c.max_length {
            args.push(format!("max_length={}", max));
        }
        if let Some(pattern) = &c.pattern {
            args.push(format!("pattern=r\"{}\"", pattern));
        }

        // Numeric constraints
        if let Some(min) = c.minimum {
            args.push(format!("ge={}", format_number(min)));
        }
        if let Some(max) = c.maximum {
            args.push(format!("le={}", format_number(max)));
        }
        if let Some(min) = c.exclusive_minimum {
            args.push(format!("gt={}", format_number(min)));
        }
        if let Some(max) = c.exclusive_maximum {
            args.push(format!("lt={}", format_number(max)));
        }
        if let Some(mult) = c.multiple_of {
            args.push(format!("multiple_of={}", format_number(mult)));
        }

        // Array constraints
        if let Some(min) = c.min_items {
            args.push(format!("min_items={}", min));
        }
        if let Some(max) = c.max_items {
            args.push(format!("max_items={}", max));
        }

        // Description
        if ctx.generate_docs {
            if let Some(desc) = &field.description {
                args.push(format!("description=\"{}\"", escape_string(desc)));
            }
        }

        // Alias
        if let Some(alias) = &field.alias {
            args.push(format!("alias=\"{}\"", alias));
        }

        args
    }

    /// Generate an enum class
    fn generate_enum(&self, enum_def: &EnumDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Check if all variants are strings
        let is_str_enum = enum_def
            .variants
            .iter()
            .all(|v| matches!(&v.value, Some(crate::spec::ir::EnumValue::String(_))));

        let base_class = if is_str_enum { "str, Enum" } else { "Enum" };

        if ctx.generate_docs {
            if let Some(desc) = &enum_def.description {
                lines.push(format!("class {}({}):", enum_def.name, base_class));
                lines.push(format!("{}\"\"\"{}\"\"\"", ctx.indent, desc));
            } else {
                lines.push(format!("class {}({}):", enum_def.name, base_class));
            }
        } else {
            lines.push(format!("class {}({}):", enum_def.name, base_class));
        }

        for variant in &enum_def.variants {
            let value_str = match &variant.value {
                Some(crate::spec::ir::EnumValue::String(s)) => format!("\"{}\"", s),
                Some(crate::spec::ir::EnumValue::Int(i)) => i.to_string(),
                None => format!("\"{}\"", variant.name.to_lowercase()),
            };
            lines.push(format!("{}{} = {}", ctx.indent, variant.name, value_str));
        }

        lines.join("\n")
    }

    /// Generate imports
    fn generate_imports(&self, spec: &DataModelSpec) -> Vec<String> {
        let mut imports = vec![
            "from typing import Optional, List, Dict, Any, Union".to_string(),
            "from cclab.shield import BaseModel, Field".to_string(),
        ];

        // Check if we need Enum
        if !spec.enums.is_empty() {
            imports.push("from enum import Enum".to_string());
        }

        // Check for special types
        let mut needs_uuid = false;
        let mut needs_datetime = false;
        let mut needs_email = false;
        let mut needs_url = false;

        for model in &spec.models {
            for field in &model.fields {
                if let Some(format) = &field.constraints.format {
                    match format {
                        StringFormat::Uuid => needs_uuid = true,
                        StringFormat::DateTime | StringFormat::Date | StringFormat::Time => {
                            needs_datetime = true
                        }
                        StringFormat::Email => needs_email = true,
                        StringFormat::Uri | StringFormat::Url => needs_url = true,
                        _ => {}
                    }
                }
            }
        }

        if needs_uuid {
            imports.push("from uuid import UUID".to_string());
        }
        if needs_datetime {
            imports.push("from datetime import datetime, date, time".to_string());
        }
        if needs_email {
            imports.push("from cclab.shield import EmailStr".to_string());
        }
        if needs_url {
            imports.push("from cclab.shield import HttpUrl".to_string());
        }

        imports
    }
}

impl Default for ShieldGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for ShieldGenerator {
    fn name(&self) -> &str {
        "shield"
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

        // Generate models
        for model in &spec.models {
            content_parts.push(self.generate_model(model, ctx));
        }

        let imports = self.generate_imports(spec);
        let content = content_parts.join("\n\n\n");

        let name = ctx
            .module_name
            .clone()
            .unwrap_or_else(|| "models".to_string());

        Ok(vec![
            GeneratedCode::new(name, content, Language::Python).with_imports(imports)
        ])
    }
}

/// Format a number for Python (remove trailing .0 for integers)
fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

/// Escape string for Python
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ir::{EnumValue, EnumVariant, FieldConstraints};
    use crate::type_inference::Type;

    #[test]
    fn test_generate_simple_model() {
        let mut model = ModelDef::new("User");
        model.description = Some("User account".to_string());
        model.fields.push(FieldDef::new("id", Type::Int));
        model.fields.push(FieldDef::new("name", Type::Str));

        let mut email_field = FieldDef::new("email", Type::Str);
        email_field.required = false;
        model.fields.push(email_field);

        let spec = DataModelSpec {
            models: vec![model],
            enums: vec![],
            relationships: vec![],
        };

        let gen = ShieldGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_data_models(&spec, &ctx).unwrap();

        assert_eq!(result.len(), 1);
        let code = &result[0].content;
        assert!(code.contains("class User(BaseModel):"));
        assert!(code.contains("id: int"));
        assert!(code.contains("name: str"));
        assert!(code.contains("email: Optional[str]"));
    }

    #[test]
    fn test_generate_field_with_constraints() {
        let mut field = FieldDef::new("age", Type::Int);
        field.constraints = FieldConstraints {
            minimum: Some(0.0),
            maximum: Some(150.0),
            ..Default::default()
        };

        let gen = ShieldGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_field(&field, &ctx);

        assert!(result.contains("Field("));
        assert!(result.contains("ge=0"));
        assert!(result.contains("le=150"));
    }

    #[test]
    fn test_generate_enum() {
        let enum_def = EnumDef {
            name: "Status".to_string(),
            description: Some("Order status".to_string()),
            variants: vec![
                EnumVariant {
                    name: "PENDING".to_string(),
                    value: Some(EnumValue::String("pending".to_string())),
                    description: None,
                },
                EnumVariant {
                    name: "COMPLETED".to_string(),
                    value: Some(EnumValue::String("completed".to_string())),
                    description: None,
                },
            ],
        };

        let gen = ShieldGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_enum(&enum_def, &ctx);

        assert!(result.contains("class Status(str, Enum):"));
        assert!(result.contains("PENDING = \"pending\""));
        assert!(result.contains("COMPLETED = \"completed\""));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/gen/python/shield.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/gen/python/shield.rs` captured during libs codegen standardization.
```
