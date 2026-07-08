---
id: libs-compass-src-gen-rust-sqlx-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/gen/rust/sqlx.rs`.
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

# Standardized libs/compass/src/gen/rust/sqlx.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/gen/rust/sqlx.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SqlxGenerator` | libs/compass/src/gen/rust/sqlx.rs | struct | pub | 11 | pub struct SqlxGenerator; |
| `new` | libs/compass/src/gen/rust/sqlx.rs | function | pub | 14 | pub fn new() -> Self { |
| `TABLE` | libs/compass/src/gen/rust/sqlx.rs | const | pub | 136 | "{}pub const TABLE: &'static str = \"{}\";", |
| `find_by_id` | libs/compass/src/gen/rust/sqlx.rs | function | pub | 145 | "{}pub async fn find_by_id(pool: &sqlx::PgPool, {}: {}) -> sqlx::Result<Option<Self>> {{", |
| `find_all` | libs/compass/src/gen/rust/sqlx.rs | function | pub | 167 | "{}pub async fn find_all(pool: &sqlx::PgPool) -> sqlx::Result<Vec<Self>> {{", |
| `insert` | libs/compass/src/gen/rust/sqlx.rs | function | pub | 191 | "{}pub async fn insert(&self, pool: &sqlx::PgPool) -> sqlx::Result<Self> {{", |
| `User` | libs/compass/src/gen/rust/sqlx.rs | struct | pub | 345 | assert!(code.contains("pub struct User {")); |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! sqlx code generator
//!
//! Generates Rust structs with sqlx derives for database operations.

use crate::gen::rust::{format_to_rust_type, type_to_rust};
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{DataModelSpec, FieldDef, ModelDef, StringFormat};
use crate::type_inference::Type;

/// Sqlx code generator
pub struct SqlxGenerator;

impl SqlxGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a single struct with sqlx derives
    fn generate_struct(&self, model: &ModelDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Doc comment
        if ctx.generate_docs {
            if let Some(desc) = &model.description {
                lines.push(format!("/// {}", desc));
            }
        }

        // Derive attributes
        lines.push("#[derive(Debug, Clone, sqlx::FromRow)]".to_string());

        lines.push(format!("pub struct {} {{", model.name));

        // Generate fields
        for field in &model.fields {
            lines.push(self.generate_field(field, ctx));
        }

        lines.push("}".to_string());

        // Generate impl block with query methods
        lines.push(String::new());
        lines.push(self.generate_impl(model, ctx));

        lines.join("\n")
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

        // sqlx attributes
        let sqlx_attrs = self.get_field_sqlx_attrs(field);
        if !sqlx_attrs.is_empty() {
            lines.push(format!("{}#[sqlx({})]", ctx.indent, sqlx_attrs.join(", ")));
        }

        // Field definition
        let type_str = self.get_field_type(field);
        lines.push(format!("{}pub {}: {},", ctx.indent, field.name, type_str));

        lines.join("\n")
    }

    /// Get field-level sqlx attributes
    fn get_field_sqlx_attrs(&self, field: &FieldDef) -> Vec<String> {
        let mut attrs = Vec::new();

        // Rename for database column
        if let Some(col_name) = &field.column_name {
            attrs.push(format!("rename = \"{}\"", col_name));
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

        // Map types to sqlx-compatible types
        let base_type = match &field.ty {
            Type::Int => {
                if field.primary_key {
                    "i64".to_string() // Use i64 for primary keys (BIGSERIAL)
                } else {
                    "i32".to_string() // Use i32 for regular integers
                }
            }
            Type::List(inner) => {
                format!("Vec<{}>", type_to_rust(inner))
            }
            _ => type_to_rust(&field.ty),
        };

        if !field.required && !matches!(&field.ty, Type::Optional(_)) {
            format!("Option<{}>", base_type)
        } else {
            base_type
        }
    }

    /// Generate impl block with query methods
    fn generate_impl(&self, model: &ModelDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        let table_name = model
            .table_name
            .clone()
            .unwrap_or_else(|| to_snake_case(&model.name) + "s");

        // Find primary key field
        let pk_field = model.fields.iter().find(|f| f.primary_key);

        lines.push(format!("impl {} {{", model.name));

        // Table name constant
        lines.push(format!(
            "{}pub const TABLE: &'static str = \"{}\";",
            ctx.indent, table_name
        ));
        lines.push(String::new());

        // find_by_id method
        if let Some(pk) = pk_field {
            let pk_type = self.get_field_type(pk);
            lines.push(format!(
                "{}pub async fn find_by_id(pool: &sqlx::PgPool, {}: {}) -> sqlx::Result<Option<Self>> {{",
                ctx.indent, pk.name, pk_type
            ));
            lines.push(format!(
                "{}{}sqlx::query_as::<_, Self>(\"SELECT * FROM {} WHERE {} = $1\")",
                ctx.indent, ctx.indent, table_name, pk.name
            ));
            lines.push(format!(
                "{}{}{}.bind({})",
                ctx.indent, ctx.indent, ctx.indent, pk.name
            ));
            lines.push(format!(
                "{}{}{}.fetch_optional(pool)",
                ctx.indent, ctx.indent, ctx.indent
            ));
            lines.push(format!("{}{}{}.await", ctx.indent, ctx.indent, ctx.indent));
            lines.push(format!("{}}}", ctx.indent));
            lines.push(String::new());
        }

        // find_all method
        lines.push(format!(
            "{}pub async fn find_all(pool: &sqlx::PgPool) -> sqlx::Result<Vec<Self>> {{",
            ctx.indent
        ));
        lines.push(format!(
            "{}{}sqlx::query_as::<_, Self>(\"SELECT * FROM {}\")",
            ctx.indent, ctx.indent, table_name
        ));
        lines.push(format!(
            "{}{}{}.fetch_all(pool)",
            ctx.indent, ctx.indent, ctx.indent
        ));
        lines.push(format!("{}{}{}.await", ctx.indent, ctx.indent, ctx.indent));
        lines.push(format!("{}}}", ctx.indent));
        lines.push(String::new());

        // insert method
        let non_pk_fields: Vec<_> = model.fields.iter().filter(|f| !f.primary_key).collect();
        if !non_pk_fields.is_empty() {
            let field_names: Vec<_> = non_pk_fields.iter().map(|f| f.name.as_str()).collect();
            let placeholders: Vec<_> = (1..=non_pk_fields.len())
                .map(|i| format!("${}", i))
                .collect();

            lines.push(format!(
                "{}pub async fn insert(&self, pool: &sqlx::PgPool) -> sqlx::Result<Self> {{",
                ctx.indent
            ));
            lines.push(format!(
                "{}{}sqlx::query_as::<_, Self>(",
                ctx.indent, ctx.indent
            ));
            lines.push(format!(
                "{}{}{}\"INSERT INTO {} ({}) VALUES ({}) RETURNING *\"",
                ctx.indent,
                ctx.indent,
                ctx.indent,
                table_name,
                field_names.join(", "),
                placeholders.join(", ")
            ));
            lines.push(format!("{}{})", ctx.indent, ctx.indent));

            for field in &non_pk_fields {
                lines.push(format!(
                    "{}{}{}.bind(&self.{})",
                    ctx.indent, ctx.indent, ctx.indent, field.name
                ));
            }

            lines.push(format!(
                "{}{}{}.fetch_one(pool)",
                ctx.indent, ctx.indent, ctx.indent
            ));
            lines.push(format!("{}{}{}.await", ctx.indent, ctx.indent, ctx.indent));
            lines.push(format!("{}}}", ctx.indent));
        }

        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generate imports
    fn generate_imports(&self, spec: &DataModelSpec) -> Vec<String> {
        let mut imports = Vec::new();

        let mut needs_uuid = false;
        let mut needs_chrono = false;

        for model in &spec.models {
            for field in &model.fields {
                if let Some(format) = &field.constraints.format {
                    match format {
                        StringFormat::Uuid => needs_uuid = true,
                        StringFormat::DateTime | StringFormat::Date | StringFormat::Time => {
                            needs_chrono = true
                        }
                        _ => {}
                    }
                }
            }
        }

        if needs_uuid {
            imports.push("use uuid::Uuid;".to_string());
        }
        if needs_chrono {
            imports.push("use chrono::{DateTime, NaiveDate, NaiveTime, Utc};".to_string());
        }

        imports
    }
}

impl Default for SqlxGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for SqlxGenerator {
    fn name(&self) -> &str {
        "sqlx"
    }

    fn generate_data_models(
        &self,
        spec: &DataModelSpec,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let mut content_parts = Vec::new();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_struct_with_pk() {
        let mut model = ModelDef::new("User");
        model.table_name = Some("users".to_string());

        let mut id_field = FieldDef::new("id", Type::Int);
        id_field.primary_key = true;
        model.fields.push(id_field);

        model.fields.push(FieldDef::new("name", Type::Str));

        let mut email = FieldDef::new("email", Type::Str);
        email.required = false;
        model.fields.push(email);

        let spec = DataModelSpec {
            models: vec![model],
            enums: vec![],
            relationships: vec![],
        };

        let gen = SqlxGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_data_models(&spec, &ctx).unwrap();

        let code = &result[0].content;
        assert!(code.contains("#[derive(Debug, Clone, sqlx::FromRow)]"));
        assert!(code.contains("pub struct User {"));
        assert!(code.contains("pub id: i64,"));
        assert!(code.contains("pub name: String,"));
        assert!(code.contains("pub email: Option<String>,"));
        assert!(code.contains("pub const TABLE: &'static str = \"users\";"));
        assert!(code.contains("pub async fn find_by_id"));
        assert!(code.contains("pub async fn find_all"));
        assert!(code.contains("pub async fn insert"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/gen/rust/sqlx.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/gen/rust/sqlx.rs` captured during libs codegen standardization.
```
