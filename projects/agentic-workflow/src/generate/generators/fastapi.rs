// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/fastapi_preamble.md#source
// CODEGEN-BEGIN
//! FastAPI code generator
//!
//! Generates a standard FastAPI project layout from a JSON Schema / OpenAPI input:
//!
//! | Output file      | Source section | Description |
//! |------------------|----------------|-------------|
//! | `models.py`      | schema         | Pydantic `BaseModel` definitions |
//! | `schemas.py`     | schema         | Create/Update/Response wrappers (cross-section) |
//! | `routes.py`      | rest-api × schema | `APIRouter` with typed handlers |
//! | `app.py`         | project config | FastAPI app entry-point |
//! | `requirements.txt` | project config | Python dependencies |
//!
//! Cross-section composition (Phase 2): route handlers reference both the base
//! models (`models.py`) and the request/response schemas (`schemas.py`), tying
//! the rest-api and schema sections together.

use super::common::{
    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::schema::{JsonSchema, SchemaType};
use serde::Serialize;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// FastAPI code generator
// ---------------------------------------------------------------------------
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/fastapi.md#schema
// CODEGEN-BEGIN
/// FastAPI generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/fastapi.md#schema
pub struct FastAPIGenerator;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/fastapi_runtime.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/fastapi_runtime.md#source
impl FastAPIGenerator {
    pub fn new() -> Self {
        Self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/fastapi_runtime.md#source
impl Default for FastAPIGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Context types (Serialize so they can be used with Tera templates too)
// ---------------------------------------------------------------------------

/// Top-level template context for all FastAPI templates.
#[derive(Debug, Serialize)]
struct FastAPIContext {
    project_name: String,
    version: String,
    /// Sorted list of models (one per schema object definition).
    models: Vec<ModelContext>,
    /// Sorted list of route groups (one per model).
    routes: Vec<RouteContext>,
}

/// Context for a single Pydantic model.
#[derive(Debug, Serialize)]
struct ModelContext {
    /// PascalCase model name, e.g. `User`
    name: String,
    /// kebab-case URL slug, e.g. `user`
    slug: String,
    fields: Vec<FieldContext>,
}

/// A single model field.
#[derive(Debug, Serialize)]
struct FieldContext {
    name: String,
    python_type: String,
    required: bool,
    default: Option<String>,
    description: Option<String>,
}

/// Route group for a single resource.
#[derive(Debug, Serialize)]
struct RouteContext {
    /// The model name this route is for, e.g. `User`
    model_name: String,
    /// URL path segment, e.g. `users`
    path_prefix: String,
    /// Create schema name, e.g. `UserCreate`
    create_schema: String,
    /// Update schema name, e.g. `UserUpdate`
    update_schema: String,
    /// Response schema name, e.g. `UserResponse`
    response_schema: String,
}

// ---------------------------------------------------------------------------
// Generator impl
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/fastapi_runtime.md#source
impl Generator for FastAPIGenerator {
    fn template_dir(&self) -> &'static str {
        "fastapi"
    }

    fn generate(
        &self,
        schema: &JsonSchema,
        settings: &GeneratorSettings,
        engine: &TemplateEngine,
    ) -> Result<Manifest, GeneratorError> {
        let mut manifest = Manifest::new();

        // Build context (deterministically sorted).
        let context = build_context(schema, settings)?;

        // Define output files: (template_name, output_file, generator_fn)
        let files: &[(&str, &str, fn(&FastAPIContext) -> String)] = &[
            ("app.py.j2", "app.py", generate_app_py),
            ("models.py.j2", "models.py", generate_models_py),
            ("schemas.py.j2", "schemas.py", generate_schemas_py),
            ("routes.py.j2", "routes.py", generate_routes_py),
            (
                "requirements.txt.j2",
                "requirements.txt",
                generate_requirements_txt,
            ),
        ];

        for (template, output, inline_gen) in files {
            let output_path = settings.output_dir.join(output);

            // Overwrite policy check.
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

            let template_name = format!("{}/{}", self.template_dir(), template);

            let content = if engine.has_template(&template_name) {
                engine.render(&template_name, &context).map_err(|e| {
                    GeneratorError::TemplateRenderError {
                        template: template_name.clone(),
                        message: e.to_string(),
                    }
                })?
            } else {
                // Inline fallback — no template file needed.
                inline_gen(&context)
            };

            manifest.add(GeneratedFile::written(output_path, &content));
        }

        Ok(manifest)
    }
}

// ---------------------------------------------------------------------------
// Context builder
// ---------------------------------------------------------------------------

fn build_context(
    schema: &JsonSchema,
    settings: &GeneratorSettings,
) -> Result<FastAPIContext, GeneratorError> {
    // Collect object models from definitions, sorted by name for determinism.
    let raw_defs: BTreeMap<String, &JsonSchema> = schema.all_definitions().into_iter().collect();

    let mut models: Vec<ModelContext> = raw_defs
        .iter()
        .filter_map(|(name, def)| extract_model(name, def))
        .collect();

    // Also include root-level object if it has a title and properties.
    if let Some(title) = &schema.title {
        if let Some(model) = extract_model(title, schema) {
            // Avoid duplicates.
            if !models.iter().any(|m| m.name == model.name) {
                models.push(model);
            }
        }
    }

    // Sort for deterministic output.
    models.sort_by(|a, b| a.name.cmp(&b.name));

    // Build route groups — one per model (cross-section composition:
    // route handler = rest-api × schema).
    let routes: Vec<RouteContext> = models
        .iter()
        .map(|m| RouteContext {
            model_name: m.name.clone(),
            path_prefix: format!("{}s", m.slug),
            create_schema: format!("{}Create", m.name),
            update_schema: format!("{}Update", m.name),
            response_schema: format!("{}Response", m.name),
        })
        .collect();

    Ok(FastAPIContext {
        project_name: settings.name.clone(),
        version: settings.version.clone(),
        models,
        routes,
    })
}

// ---------------------------------------------------------------------------
// Model extraction helpers
// ---------------------------------------------------------------------------

fn extract_model(name: &str, schema: &JsonSchema) -> Option<ModelContext> {
    if schema.effective_type() != Some(SchemaType::Object) {
        return None;
    }

    let properties = schema.properties.as_ref()?;
    let required_set: std::collections::HashSet<&str> = schema
        .required
        .as_ref()
        .map(|r| r.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();

    // Sort fields deterministically.
    let mut sorted_props: Vec<(&String, &Box<JsonSchema>)> = properties.iter().collect();
    sorted_props.sort_by_key(|(k, _)| k.as_str());

    let fields: Vec<FieldContext> = sorted_props
        .iter()
        .map(|(field_name, field_schema)| FieldContext {
            name: field_name.to_string(),
            python_type: schema_to_python_type(field_schema),
            required: required_set.contains(field_name.as_str()),
            default: field_schema.default.as_ref().map(|v| v.to_string()),
            description: field_schema.description.clone(),
        })
        .collect();

    Some(ModelContext {
        name: to_pascal_case(name),
        slug: to_snake_case(name),
        fields,
    })
}

fn schema_to_python_type(schema: &JsonSchema) -> String {
    // Handle $ref.
    if let Some(ref_path) = &schema.ref_ {
        if let Some(n) = ref_path.strip_prefix("#/definitions/") {
            return to_pascal_case(n);
        }
        if let Some(n) = ref_path.strip_prefix("#/$defs/") {
            return to_pascal_case(n);
        }
    }

    match schema.effective_type() {
        Some(SchemaType::String) => "str".to_string(),
        Some(SchemaType::Integer) => "int".to_string(),
        Some(SchemaType::Number) => "float".to_string(),
        Some(SchemaType::Boolean) => "bool".to_string(),
        Some(SchemaType::Array) => {
            let inner = schema
                .items
                .as_ref()
                .map(|i| schema_to_python_type(i))
                .unwrap_or_else(|| "Any".to_string());
            format!("list[{}]", inner)
        }
        Some(SchemaType::Object) => "dict".to_string(),
        Some(SchemaType::Null) => "None".to_string(),
        None => "Any".to_string(),
    }
}

fn to_pascal_case(s: &str) -> String {
    use heck::ToPascalCase;
    s.to_pascal_case()
}

fn to_snake_case(s: &str) -> String {
    use heck::ToSnakeCase;
    s.to_snake_case()
}

// ---------------------------------------------------------------------------
// Inline code generators (used when Tera templates are not present)
// ---------------------------------------------------------------------------

/// `app.py` — FastAPI application entry-point.
fn generate_app_py(ctx: &FastAPIContext) -> String {
    let router_imports: String = ctx
        .routes
        .iter()
        .map(|r| {
            format!(
                "from .routes import router as {}_router\n",
                to_snake_case(&r.model_name)
            )
        })
        .collect();

    let router_includes: String = ctx
        .routes
        .iter()
        .map(|r| {
            format!(
                "app.include_router({name}_router, prefix=\"/{prefix}\", tags=[\"{name}\"])\n",
                name = to_snake_case(&r.model_name),
                prefix = r.path_prefix,
            )
        })
        .collect();

    format!(
        r#"""{}
FastAPI Application
Generated by sdd
"""
from fastapi import FastAPI
{}
app = FastAPI(
    title="{}",
    version="{}",
)

{}
@app.get("/")
async def health() -> dict:
    return {{"status": "ok", "version": "{}"}}
"#,
        ctx.project_name,
        router_imports,
        ctx.project_name,
        ctx.version,
        router_includes,
        ctx.version,
    )
}

/// `models.py` — Pydantic BaseModel definitions (one class per schema object).
fn generate_models_py(ctx: &FastAPIContext) -> String {
    let mut out = format!(
        r#""""
Pydantic Models
Generated by sdd
"""
from __future__ import annotations
from pydantic import BaseModel
from typing import Optional, List, Any

"#
    );

    for model in &ctx.models {
        out.push_str(&format!("\nclass {}(BaseModel):\n", model.name));
        if model.fields.is_empty() {
            out.push_str("    pass\n");
        } else {
            for field in &model.fields {
                let annotation = if field.required {
                    field.python_type.clone()
                } else {
                    format!("Optional[{}]", field.python_type)
                };
                let default_part = if field.required {
                    String::new()
                } else {
                    " = None".to_string()
                };
                if let Some(desc) = &field.description {
                    out.push_str(&format!("    # {}\n", desc));
                }
                out.push_str(&format!(
                    "    {}: {}{}\n",
                    field.name, annotation, default_part
                ));
            }
        }
    }

    out
}

/// `schemas.py` — Request/Response Pydantic schemas (cross-section composition).
///
/// For each model `Foo` this generates:
/// - `FooCreate`   — request body for POST (all required fields)
/// - `FooUpdate`   — request body for PUT/PATCH (all fields optional)
/// - `FooResponse` — response body (extends base model, adds `id`)
fn generate_schemas_py(ctx: &FastAPIContext) -> String {
    let model_names: Vec<&str> = ctx.models.iter().map(|m| m.name.as_str()).collect();
    let import_list = model_names.join(", ");

    let mut out = format!(
        r#""""
Request/Response Schemas
Generated by sdd

These schemas compose the REST API section with the data model section:
  route handler = rest-api × schema
"""
from __future__ import annotations
from typing import Optional, Any
from .models import {import_list}

"#,
        import_list = import_list,
    );

    for model in &ctx.models {
        // FooCreate — same required fields as the base model.
        out.push_str(&format!("\nclass {}Create({}):\n", model.name, model.name));
        out.push_str("    \"\"\"Request body for creating a new resource.\"\"\"\n");
        if model.fields.is_empty() {
            out.push_str("    pass\n");
        } else {
            // Override fields that should be required for creation.
            let required: Vec<&FieldContext> = model.fields.iter().filter(|f| f.required).collect();
            if required.is_empty() {
                out.push_str("    pass\n");
            } else {
                for f in &required {
                    out.push_str(&format!("    {}: {}\n", f.name, f.python_type));
                }
            }
        }

        // FooUpdate — all fields optional.
        out.push_str(&format!("\nclass {}Update({}):\n", model.name, model.name));
        out.push_str("    \"\"\"Request body for updating an existing resource (all fields optional).\"\"\"\n");
        if model.fields.is_empty() {
            out.push_str("    pass\n");
        } else {
            for f in &model.fields {
                out.push_str(&format!(
                    "    {}: Optional[{}] = None\n",
                    f.name, f.python_type
                ));
            }
        }

        // FooResponse — extends base with id field.
        out.push_str(&format!(
            "\nclass {}Response({}):\n",
            model.name, model.name
        ));
        out.push_str("    \"\"\"Response body including server-generated fields.\"\"\"\n");
        out.push_str("    id: int\n");
        out.push_str("\n    class Config:\n");
        out.push_str("        from_attributes = True\n");
    }

    out
}

/// `routes.py` — FastAPI `APIRouter` with typed handlers per resource.
///
/// This is the cross-section composition result:
///   route handler = rest-api section × schema section
fn generate_routes_py(ctx: &FastAPIContext) -> String {
    if ctx.routes.is_empty() {
        return format!(
            r#""""
Routes
Generated by sdd
"""
from fastapi import APIRouter

router = APIRouter()
"#
        );
    }

    let schema_imports: String = ctx
        .routes
        .iter()
        .flat_map(|r| {
            [
                r.create_schema.clone(),
                r.update_schema.clone(),
                r.response_schema.clone(),
            ]
        })
        .collect::<Vec<_>>()
        .join(", ");

    let mut out = format!(
        r#""""
API Routes
Generated by sdd

Cross-section composition: route handlers reference both models and schemas.
"""
from __future__ import annotations
from typing import List
from fastapi import APIRouter, HTTPException, status
from .schemas import {schema_imports}

router = APIRouter()

"#,
        schema_imports = schema_imports,
    );

    for route in &ctx.routes {
        let model_snake = to_snake_case(&route.model_name);

        out.push_str(&format!(
            r#"
# --- {model} resource ---

@router.get("", response_model=List[{resp}])
async def list_{slug}s() -> List[{resp}]:
    """List all {model} resources."""
    # TODO: implement
    return []


@router.get("/{{{slug}_id}}", response_model={resp})
async def get_{slug}({slug}_id: int) -> {resp}:
    """Get a {model} by ID."""
    # TODO: implement
    raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="{model} not found")


@router.post("", response_model={resp}, status_code=status.HTTP_201_CREATED)
async def create_{slug}(body: {create}) -> {resp}:
    """Create a new {model}."""
    # TODO: implement
    raise HTTPException(status_code=status.HTTP_501_NOT_IMPLEMENTED, detail="Not implemented")


@router.put("/{{{slug}_id}}", response_model={resp})
async def update_{slug}({slug}_id: int, body: {update}) -> {resp}:
    """Update a {model} by ID."""
    # TODO: implement
    raise HTTPException(status_code=status.HTTP_501_NOT_IMPLEMENTED, detail="Not implemented")


@router.delete("/{{{slug}_id}}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_{slug}({slug}_id: int) -> None:
    """Delete a {model} by ID."""
    # TODO: implement
    raise HTTPException(status_code=status.HTTP_501_NOT_IMPLEMENTED, detail="Not implemented")

"#,
            model = route.model_name,
            slug = model_snake,
            resp = route.response_schema,
            create = route.create_schema,
            update = route.update_schema,
        ));
    }

    out
}

/// `requirements.txt` — Python project dependencies.
fn generate_requirements_txt(_ctx: &FastAPIContext) -> String {
    r#"fastapi>=0.111.0
pydantic>=2.0.0
uvicorn[standard]>=0.29.0
"#
    .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::schema::parse_json;

    fn user_schema() -> JsonSchema {
        parse_json(
            r#"{
                "title": "User",
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Full name" },
                    "age":  { "type": "integer" }
                },
                "required": ["name"]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn test_schema_to_python_type() {
        assert_eq!(schema_to_python_type(&JsonSchema::string()), "str");
        assert_eq!(schema_to_python_type(&JsonSchema::integer()), "int");
        assert_eq!(
            schema_to_python_type(&JsonSchema::array(JsonSchema::string())),
            "list[str]"
        );
    }

    #[test]
    fn test_build_context_models() {
        let schema = user_schema();
        let settings = GeneratorSettings::default();
        let ctx = build_context(&schema, &settings).unwrap();

        assert_eq!(ctx.models.len(), 1);
        assert_eq!(ctx.models[0].name, "User");
        assert_eq!(ctx.models[0].fields.len(), 2);
        // Fields sorted deterministically.
        assert_eq!(ctx.models[0].fields[0].name, "age");
        assert_eq!(ctx.models[0].fields[1].name, "name");
    }

    #[test]
    fn test_build_context_routes() {
        let schema = user_schema();
        let settings = GeneratorSettings::default();
        let ctx = build_context(&schema, &settings).unwrap();

        assert_eq!(ctx.routes.len(), 1);
        assert_eq!(ctx.routes[0].model_name, "User");
        assert_eq!(ctx.routes[0].create_schema, "UserCreate");
        assert_eq!(ctx.routes[0].update_schema, "UserUpdate");
        assert_eq!(ctx.routes[0].response_schema, "UserResponse");
        assert_eq!(ctx.routes[0].path_prefix, "users");
    }

    #[test]
    fn test_generate_models_py_content() {
        let schema = user_schema();
        let settings = GeneratorSettings::default();
        let ctx = build_context(&schema, &settings).unwrap();
        let output = generate_models_py(&ctx);

        assert!(output.contains("class User(BaseModel)"));
        assert!(output.contains("name: str"));
        assert!(output.contains("age: Optional[int]"));
    }

    #[test]
    fn test_generate_schemas_py_content() {
        let schema = user_schema();
        let settings = GeneratorSettings::default();
        let ctx = build_context(&schema, &settings).unwrap();
        let output = generate_schemas_py(&ctx);

        assert!(output.contains("class UserCreate(User)"));
        assert!(output.contains("class UserUpdate(User)"));
        assert!(output.contains("class UserResponse(User)"));
        // Cross-section composition import
        assert!(output.contains("from .models import User"));
    }

    #[test]
    fn test_generate_routes_py_content() {
        let schema = user_schema();
        let settings = GeneratorSettings::default();
        let ctx = build_context(&schema, &settings).unwrap();
        let output = generate_routes_py(&ctx);

        assert!(output.contains("router = APIRouter()"));
        assert!(output.contains("response_model=List[UserResponse]"));
        assert!(output.contains("body: UserCreate"));
        assert!(output.contains("body: UserUpdate"));
        assert!(output.contains("from .schemas import"));
    }

    #[test]
    fn test_generate_requirements_txt() {
        let schema = user_schema();
        let settings = GeneratorSettings::default();
        let ctx = build_context(&schema, &settings).unwrap();
        let output = generate_requirements_txt(&ctx);

        assert!(output.contains("fastapi"));
        assert!(output.contains("pydantic"));
        assert!(output.contains("uvicorn"));
    }

    #[test]
    fn test_generator_produces_five_files() {
        let schema = user_schema();
        let settings = GeneratorSettings {
            output_dir: std::path::PathBuf::from("/tmp/test_fastapi_gen"),
            ..Default::default()
        };
        let engine = crate::generate::engine::TemplateEngine::empty();
        let gen = FastAPIGenerator::new();
        let manifest = gen.generate(&schema, &settings, &engine).unwrap();

        assert_eq!(manifest.files.len(), 5);
        let paths: Vec<String> = manifest
            .files
            .keys()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(paths.contains(&"app.py".to_string()));
        assert!(paths.contains(&"models.py".to_string()));
        assert!(paths.contains(&"schemas.py".to_string()));
        assert!(paths.contains(&"routes.py".to_string()));
        assert!(paths.contains(&"requirements.txt".to_string()));
    }

    #[test]
    fn test_deterministic_output() {
        let schema = parse_json(
            r#"{
                "type": "object",
                "definitions": {
                    "Zebra": { "type": "object", "properties": { "z": { "type": "string" } }, "required": ["z"] },
                    "Apple": { "type": "object", "properties": { "a": { "type": "integer" } } }
                }
            }"#,
        )
        .unwrap();

        let settings = GeneratorSettings::default();
        let ctx1 = build_context(&schema, &settings).unwrap();
        let ctx2 = build_context(&schema, &settings).unwrap();

        // Models sorted: Apple, Zebra
        assert_eq!(ctx1.models[0].name, ctx2.models[0].name);
        assert_eq!(ctx1.models[1].name, ctx2.models[1].name);
        assert_eq!(ctx1.models[0].name, "Apple");
        assert_eq!(ctx1.models[1].name, "Zebra");
    }
}
// CODEGEN-END
