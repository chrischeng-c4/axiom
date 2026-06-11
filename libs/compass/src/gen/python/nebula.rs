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
