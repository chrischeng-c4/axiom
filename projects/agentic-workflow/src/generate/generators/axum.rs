// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/axum_preamble.md#source
// CODEGEN-BEGIN
//! Axum code generator

use super::common::{
    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::schema::{JsonSchema, SchemaType};
use serde::Serialize;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/axum.md#schema
// CODEGEN-BEGIN
/// Axum generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/axum.md#schema
pub struct AxumGenerator;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/axum_runtime.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/axum_runtime.md#source
impl AxumGenerator {
    pub fn new() -> Self {
        Self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/axum_runtime.md#source
impl Default for AxumGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for Axum templates
#[derive(Debug, Serialize)]
struct AxumContext {
    project_name: String,
    version: String,
    models: Vec<ModelContext>,
    routes: Vec<RouteContext>,
}

#[derive(Debug, Serialize)]
struct ModelContext {
    name: String,
    fields: Vec<FieldContext>,
}

#[derive(Debug, Serialize)]
struct FieldContext {
    name: String,
    rust_type: String,
    is_optional: bool,
    serde_rename: Option<String>,
}

#[derive(Debug, Serialize)]
struct RouteContext {
    path: String,
    method: String,
    handler_name: String,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/axum_runtime.md#source
impl Generator for AxumGenerator {
    fn template_dir(&self) -> &'static str {
        "axum"
    }

    fn generate(
        &self,
        schema: &JsonSchema,
        settings: &GeneratorSettings,
        engine: &TemplateEngine,
    ) -> Result<Manifest, GeneratorError> {
        let mut manifest = Manifest::new();

        // Build context from schema
        let context = build_context(schema, settings)?;

        // Define files to generate
        let files = vec![
            ("main.rs.j2", "src/main.rs"),
            ("lib.rs.j2", "src/lib.rs"),
            ("models.rs.j2", "src/models.rs"),
            ("handlers.rs.j2", "src/handlers.rs"),
            ("Cargo.toml.j2", "Cargo.toml"),
        ];

        for (template, output) in files {
            let template_name = format!("{}/{}", self.template_dir(), template);
            let output_path = settings.output_dir.join(output);

            // Check overwrite policy
            if output_path.exists() {
                match settings.overwrite_policy {
                    OverwritePolicy::Error => {
                        return Err(GeneratorError::OverwriteNotAllowed(output_path));
                    }
                    OverwritePolicy::Skip => {
                        manifest.add(GeneratedFile::skipped(output_path));
                        continue;
                    }
                    OverwritePolicy::Overwrite => {}
                }
            }

            // Check if template exists
            if !engine.has_template(&template_name) {
                let content = generate_inline(&context, output)?;
                manifest.add(GeneratedFile::written(output_path, &content));
                continue;
            }

            match engine.render(&template_name, &context) {
                Ok(content) => {
                    manifest.add(GeneratedFile::written(output_path, &content));
                }
                Err(e) => {
                    return Err(GeneratorError::TemplateRenderError {
                        template: template_name,
                        message: e.to_string(),
                    });
                }
            }
        }

        Ok(manifest)
    }
}

fn build_context(
    schema: &JsonSchema,
    settings: &GeneratorSettings,
) -> Result<AxumContext, GeneratorError> {
    let mut models = Vec::new();

    // Extract models from definitions
    for (name, def_schema) in schema.all_definitions() {
        if let Some(model) = extract_model(&name, def_schema) {
            models.push(model);
        }
    }

    // If root is an object, also create a model for it
    if let Some(title) = &schema.title {
        if let Some(model) = extract_model(title, schema) {
            models.push(model);
        }
    }

    Ok(AxumContext {
        project_name: settings.name.clone(),
        version: settings.version.clone(),
        models,
        routes: Vec::new(),
    })
}

fn extract_model(name: &str, schema: &JsonSchema) -> Option<ModelContext> {
    let effective_type = schema.effective_type();
    if effective_type != Some(SchemaType::Object) {
        return None;
    }

    let properties = schema.properties.as_ref()?;
    let required_fields: std::collections::HashSet<_> = schema
        .required
        .as_ref()
        .map(|r| r.iter().collect())
        .unwrap_or_default();

    let fields: Vec<_> = properties
        .iter()
        .map(|(field_name, field_schema)| {
            let is_optional = !required_fields.contains(field_name);
            let rust_type = schema_to_rust_type(field_schema, is_optional);

            // Check if field name needs serde rename (snake_case vs camelCase)
            let serde_rename =
                if field_name.contains('-') || field_name.chars().any(|c| c.is_uppercase()) {
                    Some(field_name.clone())
                } else {
                    None
                };

            FieldContext {
                name: to_snake_case(field_name),
                rust_type,
                is_optional,
                serde_rename,
            }
        })
        .collect();

    Some(ModelContext {
        name: to_pascal_case(name),
        fields,
    })
}

fn schema_to_rust_type(schema: &JsonSchema, wrap_option: bool) -> String {
    let base_type = schema_to_rust_type_inner(schema);
    if wrap_option {
        format!("Option<{}>", base_type)
    } else {
        base_type
    }
}

fn schema_to_rust_type_inner(schema: &JsonSchema) -> String {
    // Handle $ref
    if let Some(ref_path) = &schema.ref_ {
        if let Some(name) = ref_path.strip_prefix("#/definitions/") {
            return to_pascal_case(name);
        }
        if let Some(name) = ref_path.strip_prefix("#/$defs/") {
            return to_pascal_case(name);
        }
    }

    match schema.effective_type() {
        Some(SchemaType::String) => "String".to_string(),
        Some(SchemaType::Integer) => "i64".to_string(),
        Some(SchemaType::Number) => "f64".to_string(),
        Some(SchemaType::Boolean) => "bool".to_string(),
        Some(SchemaType::Array) => {
            if let Some(items) = &schema.items {
                format!("Vec<{}>", schema_to_rust_type_inner(items))
            } else {
                "Vec<serde_json::Value>".to_string()
            }
        }
        Some(SchemaType::Object) => "serde_json::Value".to_string(),
        Some(SchemaType::Null) => "()".to_string(),
        None => "serde_json::Value".to_string(),
    }
}

fn to_snake_case(s: &str) -> String {
    use heck::ToSnakeCase;
    s.to_snake_case()
}

fn to_pascal_case(s: &str) -> String {
    use heck::ToPascalCase;
    s.to_pascal_case()
}

/// Generate code inline when templates are not available
fn generate_inline(context: &AxumContext, output: &str) -> Result<String, GeneratorError> {
    match output {
        "src/main.rs" => Ok(generate_main_rs(context)),
        "src/lib.rs" => Ok(generate_lib_rs(context)),
        "src/models.rs" => Ok(generate_models_rs(context)),
        "src/handlers.rs" => Ok(generate_handlers_rs(context)),
        "Cargo.toml" => Ok(generate_cargo_toml(context)),
        _ => Err(GeneratorError::TemplateRenderError {
            template: output.to_string(),
            message: "Unknown output file".to_string(),
        }),
    }
}

fn generate_main_rs(context: &AxumContext) -> String {
    format!(
        r#"//! {} - Axum Application
//! Generated by sdd

use {name}::create_router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {{
    let app = create_router();
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}}
"#,
        context.project_name,
        name = context.project_name.replace('-', "_")
    )
}

fn generate_lib_rs(context: &AxumContext) -> String {
    format!(
        r#"//! {} library
//! Generated by sdd

pub mod handlers;
pub mod models;

use axum::{{routing::get, Router}};
use handlers::*;

pub fn create_router() -> Router {{
    Router::new()
        .route("/", get(root))
}}
"#,
        context.project_name
    )
}

fn generate_models_rs(context: &AxumContext) -> String {
    let mut output = String::from(
        r#"//! Data models
//! Generated by sdd

use serde::Deserialize;

"#,
    );

    for model in &context.models {
        output.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        output.push_str(&format!("pub struct {} {{\n", model.name));
        for field in &model.fields {
            if let Some(rename) = &field.serde_rename {
                output.push_str(&format!("    #[serde(rename = \"{}\")]\n", rename));
            }
            output.push_str(&format!("    pub {}: {},\n", field.name, field.rust_type));
        }
        output.push_str("}\n\n");
    }

    output
}

fn generate_handlers_rs(context: &AxumContext) -> String {
    format!(
        r#"//! Request handlers
//! Generated by sdd

use axum::Json;
use serde_json::{{json, Value}};

pub async fn root() -> Json<Value> {{
    Json(json!({{
        "message": "Hello from {}"
    }}))
}}
"#,
        context.project_name
    )
}

fn generate_cargo_toml(context: &AxumContext) -> String {
    format!(
        r#"[package]
name = "{}"
version = "{}"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
        context.project_name.replace('-', "_"),
        context.version
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_to_rust_type() {
        let string_schema = JsonSchema::string();
        assert_eq!(schema_to_rust_type_inner(&string_schema), "String");

        let int_schema = JsonSchema::integer();
        assert_eq!(schema_to_rust_type_inner(&int_schema), "i64");

        let array_schema = JsonSchema::array(JsonSchema::string());
        assert_eq!(schema_to_rust_type_inner(&array_schema), "Vec<String>");
    }

    #[test]
    fn test_optional_wrapping() {
        let string_schema = JsonSchema::string();
        assert_eq!(schema_to_rust_type(&string_schema, true), "Option<String>");
        assert_eq!(schema_to_rust_type(&string_schema, false), "String");
    }
}
// CODEGEN-END
