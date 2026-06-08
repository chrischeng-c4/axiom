// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/express_preamble.md#source
// CODEGEN-BEGIN
//! Express.js code generator

use super::common::{
    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::schema::{JsonSchema, SchemaType};
use serde::Serialize;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/express.md#schema
// CODEGEN-BEGIN
/// Express generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/express.md#schema
pub struct ExpressGenerator;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/express_runtime.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/express_runtime.md#source
impl ExpressGenerator {
    pub fn new() -> Self {
        Self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/express_runtime.md#source
impl Default for ExpressGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for Express templates
#[derive(Debug, Serialize)]
struct ExpressContext {
    project_name: String,
    version: String,
    lang: String,
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
    ts_type: String,
    required: bool,
}

#[derive(Debug, Serialize)]
struct RouteContext {
    path: String,
    method: String,
    handler_name: String,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/express_runtime.md#source
impl Generator for ExpressGenerator {
    fn template_dir(&self) -> &'static str {
        "express"
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

        // Define files to generate based on language
        let files = if settings.lang == "ts" || settings.lang.is_empty() {
            vec![
                ("index.ts.j2", "src/index.ts"),
                ("types.ts.j2", "src/types/index.ts"),
                ("package.json.j2", "package.json"),
                ("tsconfig.json.j2", "tsconfig.json"),
            ]
        } else {
            vec![
                ("index.js.j2", "src/index.js"),
                ("package.json.j2", "package.json"),
            ]
        };

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
                    OverwritePolicy::Overwrite => {
                        // Continue to generate
                    }
                }
            }

            // Check if template exists
            if !engine.has_template(&template_name) {
                // Generate inline if template missing
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
) -> Result<ExpressContext, GeneratorError> {
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

    Ok(ExpressContext {
        project_name: settings.name.clone(),
        version: settings.version.clone(),
        lang: if settings.lang.is_empty() {
            "ts".to_string()
        } else {
            settings.lang.clone()
        },
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
            let ts_type = schema_to_ts_type(field_schema);
            FieldContext {
                name: field_name.clone(),
                ts_type,
                required: required_fields.contains(field_name),
            }
        })
        .collect();

    Some(ModelContext {
        name: name.to_string(),
        fields,
    })
}

fn schema_to_ts_type(schema: &JsonSchema) -> String {
    // Handle $ref
    if let Some(ref_path) = &schema.ref_ {
        if let Some(name) = ref_path.strip_prefix("#/definitions/") {
            return name.to_string();
        }
        if let Some(name) = ref_path.strip_prefix("#/$defs/") {
            return name.to_string();
        }
    }

    match schema.effective_type() {
        Some(SchemaType::String) => "string".to_string(),
        Some(SchemaType::Integer) | Some(SchemaType::Number) => "number".to_string(),
        Some(SchemaType::Boolean) => "boolean".to_string(),
        Some(SchemaType::Array) => {
            if let Some(items) = &schema.items {
                format!("{}[]", schema_to_ts_type(items))
            } else {
                "unknown[]".to_string()
            }
        }
        Some(SchemaType::Object) => "Record<string, unknown>".to_string(),
        Some(SchemaType::Null) => "null".to_string(),
        None => "unknown".to_string(),
    }
}

/// Generate code inline when templates are not available
fn generate_inline(context: &ExpressContext, output: &str) -> Result<String, GeneratorError> {
    match output {
        "src/index.ts" => Ok(generate_index_ts(context)),
        "src/types/index.ts" => Ok(generate_types_ts(context)),
        "package.json" => Ok(generate_package_json(context)),
        "tsconfig.json" => Ok(generate_tsconfig_json()),
        "src/index.js" => Ok(generate_index_js(context)),
        _ => Err(GeneratorError::TemplateRenderError {
            template: output.to_string(),
            message: "Unknown output file".to_string(),
        }),
    }
}

fn generate_index_ts(context: &ExpressContext) -> String {
    format!(
        r#"/**
 * {} - Express Application
 * Generated by sdd
 */
import express from 'express';
import {{ json }} from 'express';

const app = express();
app.use(json());

const PORT = process.env.PORT || 3000;

app.get('/', (req, res) => {{
  res.json({{ message: 'Hello from {}' }});
}});

app.listen(PORT, () => {{
  console.log(`Server running on port ${{PORT}}`);
}});

export default app;
"#,
        context.project_name, context.project_name
    )
}

fn generate_index_js(context: &ExpressContext) -> String {
    format!(
        r#"/**
 * {} - Express Application
 * Generated by sdd
 */
const express = require('express');

const app = express();
app.use(express.json());

const PORT = process.env.PORT || 3000;

app.get('/', (req, res) => {{
  res.json({{ message: 'Hello from {}' }});
}});

app.listen(PORT, () => {{
  console.log(`Server running on port ${{PORT}}`);
}});

module.exports = app;
"#,
        context.project_name, context.project_name
    )
}

fn generate_types_ts(context: &ExpressContext) -> String {
    let mut output = String::from(
        r#"/**
 * Type definitions
 * Generated by sdd
 */

"#,
    );

    for model in &context.models {
        output.push_str(&format!("export interface {} {{\n", model.name));
        for field in &model.fields {
            let optional = if field.required { "" } else { "?" };
            output.push_str(&format!(
                "  {}{}: {};\n",
                field.name, optional, field.ts_type
            ));
        }
        output.push_str("}\n\n");
    }

    output
}

fn generate_package_json(context: &ExpressContext) -> String {
    if context.lang == "ts" {
        format!(
            r#"{{
  "name": "{}",
  "version": "{}",
  "main": "dist/index.js",
  "scripts": {{
    "build": "tsc",
    "start": "node dist/index.js",
    "dev": "ts-node src/index.ts"
  }},
  "dependencies": {{
    "express": "^4.18.2"
  }},
  "devDependencies": {{
    "@types/express": "^4.17.21",
    "@types/node": "^20.10.0",
    "typescript": "^5.3.0",
    "ts-node": "^10.9.0"
  }}
}}
"#,
            context.project_name, context.version
        )
    } else {
        format!(
            r#"{{
  "name": "{}",
  "version": "{}",
  "main": "src/index.js",
  "scripts": {{
    "start": "node src/index.js"
  }},
  "dependencies": {{
    "express": "^4.18.2"
  }}
}}
"#,
            context.project_name, context.version
        )
    }
}

fn generate_tsconfig_json() -> String {
    r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_to_ts_type() {
        let string_schema = JsonSchema::string();
        assert_eq!(schema_to_ts_type(&string_schema), "string");

        let int_schema = JsonSchema::integer();
        assert_eq!(schema_to_ts_type(&int_schema), "number");

        let array_schema = JsonSchema::array(JsonSchema::string());
        assert_eq!(schema_to_ts_type(&array_schema), "string[]");
    }
}
// CODEGEN-END
