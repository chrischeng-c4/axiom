//! cclab.titan code generator
//!
//! Generates PostgreSQL ORM models using cclab.titan.

use crate::gen::python::type_to_python;
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{
    DataModelSpec, FieldDef, ModelDef, RelationType, Relationship, StringFormat,
};
use crate::type_inference::Type;

/// Titan (PostgreSQL ORM) code generator
pub struct TitanGenerator;

impl TitanGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a single model class
    fn generate_model(&self, model: &ModelDef, spec: &DataModelSpec, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Get table name
        let table_name = model
            .table_name
            .clone()
            .unwrap_or_else(|| to_snake_case(&model.name) + "s");

        // Class definition
        if ctx.generate_docs {
            if let Some(desc) = &model.description {
                lines.push(format!("class {}(Model):", model.name));
                lines.push(format!("{}\"\"\"{}\"\"\"", ctx.indent, desc));
            } else {
                lines.push(format!("class {}(Model):", model.name));
            }
        } else {
            lines.push(format!("class {}(Model):", model.name));
        }

        // Meta class
        lines.push(format!("{}class Meta:", ctx.indent));
        lines.push(format!(
            "{}{}table_name = \"{}\"",
            ctx.indent, ctx.indent, table_name
        ));

        // Schema if specified
        if let Some(schema) = &ctx.db_schema {
            lines.push(format!(
                "{}{}schema = \"{}\"",
                ctx.indent, ctx.indent, schema
            ));
        }

        lines.push(String::new());

        // Generate fields
        for field in &model.fields {
            lines.push(self.generate_field(field, ctx));
        }

        // Generate relationships
        let rels = self.get_model_relationships(&model.name, spec);
        for rel in rels {
            lines.push(self.generate_relationship(&rel, ctx));
        }

        lines.join("\n")
    }

    /// Generate a single field
    fn generate_field(&self, field: &FieldDef, ctx: &GenContext) -> String {
        let column_type = self.get_column_type(field);
        let mut args = Vec::new();

        // Column name if different
        if let Some(col_name) = &field.column_name {
            args.push(format!("column_name=\"{}\"", col_name));
        }

        // Primary key
        if field.primary_key {
            args.push("primary_key=True".to_string());
        }

        // Nullable
        if !field.required && !field.primary_key {
            args.push("nullable=True".to_string());
        }

        // Unique
        if field.unique {
            args.push("unique=True".to_string());
        }

        // Indexed
        if field.indexed {
            args.push("index=True".to_string());
        }

        // Default value
        if let Some(default) = &field.default {
            args.push(format!("default={}", default));
        }

        // String length constraints
        if let Some(max_len) = field.constraints.max_length {
            if matches!(field.ty, Type::Str) {
                args.push(format!("max_length={}", max_len));
            }
        }

        // Foreign key
        if let Some(fk) = &field.foreign_key {
            let fk_str = format!("{}.{}", to_snake_case(&fk.model), fk.field);
            args.push(format!("foreign_key=\"{}\"", fk_str));
        }

        let args_str = if args.is_empty() {
            String::new()
        } else {
            format!("({})", args.join(", "))
        };

        let type_annotation = self.get_type_annotation(field);

        format!(
            "{}{}: {} = {}{}",
            ctx.indent, field.name, type_annotation, column_type, args_str
        )
    }

    /// Get PostgreSQL column type
    fn get_column_type(&self, field: &FieldDef) -> &'static str {
        // Check for special formats first
        if let Some(format) = &field.constraints.format {
            match format {
                StringFormat::Uuid => return "UUIDColumn",
                StringFormat::DateTime => return "DateTimeColumn",
                StringFormat::Date => return "DateColumn",
                StringFormat::Time => return "TimeColumn",
                StringFormat::Email | StringFormat::Url => return "StringColumn",
                _ => {}
            }
        }

        // Check primary key for auto-increment
        if field.primary_key && matches!(field.ty, Type::Int) {
            return "SerialColumn";
        }

        match &field.ty {
            Type::Bool => "BoolColumn",
            Type::Int => "IntColumn",
            Type::Float => "FloatColumn",
            Type::Str => {
                if field.constraints.max_length.is_some() {
                    "VarcharColumn"
                } else {
                    "TextColumn"
                }
            }
            Type::Bytes => "ByteaColumn",
            Type::List(_) => "ArrayColumn",
            Type::Dict(_, _) => "JsonbColumn",
            Type::Optional(inner) => {
                // Get inner type's column
                let inner_field = FieldDef {
                    ty: (**inner).clone(),
                    ..field.clone()
                };
                self.get_column_type(&inner_field)
            }
            Type::Instance { .. } => {
                // Reference to another model - this is a foreign key
                "IntColumn" // FK column type, usually int
            }
            _ => "TextColumn",
        }
    }

    /// Get Python type annotation
    fn get_type_annotation(&self, field: &FieldDef) -> String {
        // Check for special formats
        if let Some(format) = &field.constraints.format {
            let base = match format {
                StringFormat::Uuid => "UUID",
                StringFormat::DateTime => "datetime",
                StringFormat::Date => "date",
                StringFormat::Time => "time",
                _ => return type_to_python(&field.ty),
            };

            return if field.required {
                base.to_string()
            } else {
                format!("Optional[{}]", base)
            };
        }

        type_to_python(&field.ty)
    }

    /// Get relationships for a model
    fn get_model_relationships<'a>(
        &self,
        model_name: &str,
        spec: &'a DataModelSpec,
    ) -> Vec<&'a Relationship> {
        spec.relationships
            .iter()
            .filter(|r| r.from_model == model_name || r.to_model == model_name)
            .collect()
    }

    /// Generate relationship field
    fn generate_relationship(&self, rel: &Relationship, ctx: &GenContext) -> String {
        let (rel_type, related_model) = match rel.rel_type {
            RelationType::OneToOne => ("OneToOne", &rel.to_model),
            RelationType::OneToMany => ("OneToMany", &rel.to_model),
            RelationType::ManyToOne => ("ManyToOne", &rel.to_model),
            RelationType::ManyToMany => ("ManyToMany", &rel.to_model),
        };

        let field_name = to_snake_case(related_model);

        format!(
            "{}{}: {} = {}(\"{}\")",
            ctx.indent, field_name, related_model, rel_type, related_model
        )
    }

    /// Generate imports
    fn generate_imports(&self, spec: &DataModelSpec) -> Vec<String> {
        let mut imports = vec![
            "from typing import Optional, List".to_string(),
            "from cclab.titan import Model".to_string(),
        ];

        // Collect column types needed
        let mut column_types = std::collections::HashSet::new();
        let mut needs_uuid = false;
        let mut needs_datetime = false;
        let mut rel_types = std::collections::HashSet::new();

        for model in &spec.models {
            for field in &model.fields {
                let col_type = self.get_column_type(field);
                column_types.insert(col_type);

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

        for rel in &spec.relationships {
            match rel.rel_type {
                RelationType::OneToOne => rel_types.insert("OneToOne"),
                RelationType::OneToMany => rel_types.insert("OneToMany"),
                RelationType::ManyToOne => rel_types.insert("ManyToOne"),
                RelationType::ManyToMany => rel_types.insert("ManyToMany"),
            };
        }

        // Add column imports
        let col_types: Vec<_> = column_types.into_iter().collect();
        if !col_types.is_empty() {
            imports.push(format!("from cclab.titan import {}", col_types.join(", ")));
        }

        // Add relationship imports
        let rel_types: Vec<_> = rel_types.into_iter().collect();
        if !rel_types.is_empty() {
            imports.push(format!("from cclab.titan import {}", rel_types.join(", ")));
        }

        if needs_uuid {
            imports.push("from uuid import UUID".to_string());
        }
        if needs_datetime {
            imports.push("from datetime import datetime, date, time".to_string());
        }

        imports
    }
}

impl Default for TitanGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for TitanGenerator {
    fn name(&self) -> &str {
        "titan"
    }

    fn generate_data_models(
        &self,
        spec: &DataModelSpec,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let mut content_parts = Vec::new();

        // Generate models
        for model in &spec.models {
            content_parts.push(self.generate_model(model, spec, ctx));
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
    fn test_generate_model() {
        let mut model = ModelDef::new("User");
        model.description = Some("User table".to_string());

        let mut id_field = FieldDef::new("id", Type::Int);
        id_field.primary_key = true;
        model.fields.push(id_field);

        let mut email_field = FieldDef::new("email", Type::Str);
        email_field.unique = true;
        email_field.constraints.max_length = Some(255);
        model.fields.push(email_field);

        let spec = DataModelSpec {
            models: vec![model],
            enums: vec![],
            relationships: vec![],
        };

        let gen = TitanGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_data_models(&spec, &ctx).unwrap();

        let code = &result[0].content;
        assert!(code.contains("class User(Model):"));
        assert!(code.contains("table_name = \"users\""));
        assert!(code.contains("id: int = SerialColumn(primary_key=True)"));
        assert!(code.contains("email: str = VarcharColumn(unique=True, max_length=255)"));
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
        assert_eq!(to_snake_case("HTTPRequest"), "h_t_t_p_request");
        assert_eq!(to_snake_case("user"), "user");
    }
}
