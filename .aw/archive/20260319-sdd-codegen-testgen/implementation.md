---
id: implementation
type: change_implementation
change_id: sdd-codegen-testgen
---

# Implementation

## Summary

Implement codegen last mile: FastAPI pipeline + RequirementPlus test scaffold generator (refs #932, #933).

1. **TestGenerator** (`generate/generators/test_generator.rs`, NEW): Full RequirementPlus → pytest scaffold pipeline. Maps ElementDef BDD fields (given/when/then) to test function comments. Maps verifymethod: Test → test class+function, verifymethod: Inspection → TODO comment. Maps risk: High → @pytest.mark.critical. Maps Verifies relationship → test function per scenario. Maps Satisfies relationship → import comment for module under test. Maps Derives relationship → ordering comment (run base requirement tests first). Coverage validation: flags requirements with no Verifies relationship as warnings; --strict makes them hard errors for CI gating. Output: tests/test_{id}.py with deterministic class/function ordering.

2. **cclab sdd gen test CLI** (`cclab-sdd-cli/src/commands.rs`, `codegen.rs`): New `GenCommands::Test` subcommand. Accepts `<spec-path>` (RequirementDiagramDef JSON or SpecIR RequirementPlus JSON). Flags: `--output <dir>` (output directory), `--strict` (coverage errors for CI), `--dry-run` (print to stdout). Parses input with dual-format fallback: SpecIR first, then RequirementDiagramDef directly.

3. **FastAPIGenerator rewrite** (`generate/generators/fastapi.rs`): Expand from 2 to 5 output files (app.py, models.py, schemas.py, routes.py, requirements.txt). Phase 2 cross-section composition: route handlers = rest-api section × schema section. schemas.py generates FooCreate/FooUpdate/FooResponse wrappers that import from models.py. routes.py generates full CRUD APIRouter with typed parameters referencing schemas. app.py generates FastAPI entry-point with per-resource router registration. Deterministic output via BTreeMap sorting + heck PascalCase/snake_case normalization. Inline fallback generators for all 5 files (no Tera templates required).

4. **Public API** (`generate/lib.rs`, `generate/generators/mod.rs`): Export TestGenerator, TestGenResult, TestGenError, CoverageIssue.

5. **Bug fix** (`tools/agent.rs`): Char-boundary-safe stdout truncation to prevent panics on multi-byte UTF-8 at slice boundary.

## Diff

```diff
diff --git a/crates/cclab-sdd-cli/src/codegen.rs b/crates/cclab-sdd-cli/src/codegen.rs
index 63939a88..3f369cbc 100644
--- a/crates/cclab-sdd-cli/src/codegen.rs
+++ b/crates/cclab-sdd-cli/src/codegen.rs
@@ -7,6 +7,9 @@ use crate::commands::GenCommands;
 /// Dispatch gen subcommands.
 pub fn dispatch(cmd: GenCommands) -> Result<()> {
     match cmd {
+        GenCommands::Test { spec_path, output, strict, dry_run } => {
+            run_gen_test(&spec_path, output.as_deref(), strict, dry_run)
+        }
         GenCommands::Stub {
             src, output, all_pyo3, features,
             all_features, no_default_features, dry_run,
@@ -19,6 +22,86 @@ pub fn dispatch(cmd: GenCommands) -> Result<()> {
     }
 }
 
+fn run_gen_test(
+    spec_path: &str,
+    output: Option<&str>,
+    strict: bool,
+    dry_run: bool,
+) -> Result<()> {
+    use cclab_sdd::generate::{TestGenerator, SpecIR};
+    use cclab_sdd::generate::diagrams::RequirementDiagramDef;
+
+    let raw = std::fs::read_to_string(spec_path)
+        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;
+
+    // Try to parse as SpecIR first, then fall back to RequirementDiagramDef directly.
+    let def: RequirementDiagramDef = if let Ok(ir) = serde_json::from_str::<SpecIR>(&raw) {
+        match ir {
+            SpecIR::RequirementPlus { def, .. } => def,
+            other => {
+                anyhow::bail!(
+                    "SpecIR kind '{}' is not RequirementPlus. \
+                     Provide a RequirementPlus SpecIR or a RequirementDiagramDef JSON.",
+                    other.kind()
+                );
+            }
+        }
+    } else {
+        serde_json::from_str::<RequirementDiagramDef>(&raw)
+            .with_context(|| {
+                format!(
+                    "Failed to parse '{}' as RequirementDiagramDef or SpecIR. \
+                     Expected JSON with 'id', 'requirements', and optional 'elements'/'relationships'.",
+                    spec_path
+                )
+            })?
+    };
+
+    let generator = TestGenerator::new(strict);
+    let result = generator
+        .generate(&def)
+        .with_context(|| format!("Test generation failed for '{}'", spec_path))?;
+
+    // Emit coverage warnings to stderr.
+    for issue in &result.coverage_issues {
+        eprintln!("WARNING: {}", issue.message);
+    }
+
+    if dry_run {
+        println!("{}", result.content);
+        if !result.coverage_issues.is_empty() {
+            eprintln!("\n{} coverage warning(s). Use --strict to make them errors.", result.coverage_issues.len());
+        }
+        return Ok(());
+    }
+
+    // Write the generated file.
+    let base_dir = output
+        .map(std::path::Path::new)
+        .unwrap_or_else(|| std::path::Path::new("."));
+
+    let out_path = base_dir.join(&result.file_path);
+
+    if let Some(parent) = out_path.parent() {
+        std::fs::create_dir_all(parent)
+            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
+    }
+
+    std::fs::write(&out_path, &result.content)
+        .with_context(|| format!("Failed to write test file: {}", out_path.display()))?;
+
+    println!("Generated: {}", out_path.display());
+
+    if !result.coverage_issues.is_empty() {
+        eprintln!(
+            "{} coverage warning(s). Run with --strict to enforce coverage in CI.",
+            result.coverage_issues.len()
+        );
+    }
+
+    Ok(())
+}
+
 fn run_gen_stub(
     src: Option<String>,
     output: Option<String>,
diff --git a/crates/cclab-sdd-cli/src/commands.rs b/crates/cclab-sdd-cli/src/commands.rs
index ddabdb22..bd275809 100644
--- a/crates/cclab-sdd-cli/src/commands.rs
+++ b/crates/cclab-sdd-cli/src/commands.rs
@@ -404,6 +404,24 @@ pub enum DaemonCommands {
 /// Code generation subcommands
 #[derive(Subcommand)]
 pub enum GenCommands {
+    /// Generate pytest test scaffolds from a RequirementPlus spec (cclab sdd gen test <spec-path>)
+    Test {
+        /// Path to a JSON file containing a RequirementDiagramDef or SpecIR RequirementPlus
+        spec_path: String,
+
+        /// Output directory for the generated test file (default: current directory)
+        #[arg(short, long)]
+        output: Option<String>,
+
+        /// Make uncovered requirements (no Verifies relationship) hard errors for CI gating
+        #[arg(long)]
+        strict: bool,
+
+        /// Print generated content to stdout instead of writing a file
+        #[arg(long)]
+        dry_run: bool,
+    },
+
     /// Generate Python stubs (.pyi) from PyO3 Rust code
     Stub {
         /// Path to Rust source file or crate directory
@@ -684,6 +702,7 @@ pub async fn run_command(cmd: Commands) -> Result<()> {
             codegen::dispatch(cmd)?;
         }
 
+
         // =================================================================
         // LSP
         // =================================================================
diff --git a/crates/cclab-sdd/src/generate/generators/fastapi.rs b/crates/cclab-sdd/src/generate/generators/fastapi.rs
index 125ad878..c9956ecc 100644
--- a/crates/cclab-sdd/src/generate/generators/fastapi.rs
+++ b/crates/cclab-sdd/src/generate/generators/fastapi.rs
@@ -1,11 +1,32 @@
 //! FastAPI code generator
+//!
+//! Generates a standard FastAPI project layout from a JSON Schema / OpenAPI input:
+//!
+//! | Output file      | Source section | Description |
+//! |------------------|----------------|-------------|
+//! | `models.py`      | schema         | Pydantic `BaseModel` definitions |
+//! | `schemas.py`     | schema         | Create/Update/Response wrappers (cross-section) |
+//! | `routes.py`      | rest-api × schema | `APIRouter` with typed handlers |
+//! | `app.py`         | project config | FastAPI app entry-point |
+//! | `requirements.txt` | project config | Python dependencies |
+//!
+//! Cross-section composition (Phase 2): route handlers reference both the base
+//! models (`models.py`) and the request/response schemas (`schemas.py`), tying
+//! the rest-api and schema sections together.
 
-use super::common::{Generator, GeneratorError, GeneratorSettings, GeneratedFile, Manifest, OverwritePolicy};
+use super::common::{
+    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,
+};
 use crate::generate::engine::TemplateEngine;
 use crate::generate::schema::{JsonSchema, SchemaType};
 use serde::Serialize;
+use std::collections::BTreeMap;
 
-/// FastAPI code generator
+// ---------------------------------------------------------------------------
+// FastAPI code generator
+// ---------------------------------------------------------------------------
+
+/// FastAPI code generator.
 pub struct FastAPIGenerator;
 
 impl FastAPIGenerator {
@@ -20,21 +41,32 @@ impl Default for FastAPIGenerator {
     }
 }
 
-/// Context for FastAPI templates
+// ---------------------------------------------------------------------------
+// Context types (Serialize so they can be used with Tera templates too)
+// ---------------------------------------------------------------------------
+
+/// Top-level template context for all FastAPI templates.
 #[derive(Debug, Serialize)]
 struct FastAPIContext {
     project_name: String,
     version: String,
+    /// Sorted list of models (one per schema object definition).
     models: Vec<ModelContext>,
+    /// Sorted list of route groups (one per model).
     routes: Vec<RouteContext>,
 }
 
+/// Context for a single Pydantic model.
 #[derive(Debug, Serialize)]
 struct ModelContext {
+    /// PascalCase model name, e.g. `User`
     name: String,
+    /// kebab-case URL slug, e.g. `user`
+    slug: String,
     fields: Vec<FieldContext>,
 }
 
+/// A single model field.
 #[derive(Debug, Serialize)]
 struct FieldContext {
     name: String,
@@ -44,15 +76,25 @@ struct FieldContext {
     description: Option<String>,
 }
 
+/// Route group for a single resource.
 #[derive(Debug, Serialize)]
 struct RouteContext {
-    path: String,
-    method: String,
-    function_name: String,
-    request_model: Option<String>,
-    response_model: Option<String>,
+    /// The model name this route is for, e.g. `User`
+    model_name: String,
+    /// URL path segment, e.g. `users`
+    path_prefix: String,
+    /// Create schema name, e.g. `UserCreate`
+    create_schema: String,
+    /// Update schema name, e.g. `UserUpdate`
+    update_schema: String,
+    /// Response schema name, e.g. `UserResponse`
+    response_schema: String,
 }
 
+// ---------------------------------------------------------------------------
+// Generator impl
+// ---------------------------------------------------------------------------
+
 impl Generator for FastAPIGenerator {
     fn template_dir(&self) -> &'static str {
         "fastapi"
@@ -66,20 +108,22 @@ impl Generator for FastAPIGenerator {
     ) -> Result<Manifest, GeneratorError> {
         let mut manifest = Manifest::new();
 
-        // Build context from schema
+        // Build context (deterministically sorted).
         let context = build_context(schema, settings)?;
 
-        // Define files to generate
-        let files = [
-            ("main.py.j2", "main.py"),
-            ("models.py.j2", "models.py"),
+        // Define output files: (template_name, output_file, generator_fn)
+        let files: &[(&str, &str, fn(&FastAPIContext) -> String)] = &[
+            ("app.py.j2", "app.py", generate_app_py),
+            ("models.py.j2", "models.py", generate_models_py),
+            ("schemas.py.j2", "schemas.py", generate_schemas_py),
+            ("routes.py.j2", "routes.py", generate_routes_py),
+            ("requirements.txt.j2", "requirements.txt", generate_requirements_txt),
         ];
 
-        for (template, output) in files {
-            let template_name = format!("{}/{}", self.template_dir(), template);
+        for (template, output, inline_gen) in files {
             let output_path = settings.output_dir.join(output);
 
-            // Check overwrite policy
+            // Overwrite policy check.
             if output_path.exists() {
                 match settings.overwrite_policy {
                     OverwritePolicy::Error => {
@@ -93,118 +137,141 @@ impl Generator for FastAPIGenerator {
                 }
             }
 
-            // Check if template exists
-            if !engine.has_template(&template_name) {
-                // Generate inline if template missing
-                let content = generate_inline(&context, output)?;
-                manifest.add(GeneratedFile::written(output_path, &content));
-                continue;
-            }
+            let template_name = format!("{}/{}", self.template_dir(), template);
 
-            match engine.render(&template_name, &context) {
-                Ok(content) => {
-                    manifest.add(GeneratedFile::written(output_path, &content));
-                }
-                Err(e) => {
-                    return Err(GeneratorError::TemplateRenderError {
-                        template: template_name,
+            let content = if engine.has_template(&template_name) {
+                engine.render(&template_name, &context).map_err(|e| {
+                    GeneratorError::TemplateRenderError {
+                        template: template_name.clone(),
                         message: e.to_string(),
-                    });
-                }
-            }
+                    }
+                })?
+            } else {
+                // Inline fallback — no template file needed.
+                inline_gen(&context)
+            };
+
+            manifest.add(GeneratedFile::written(output_path, &content));
         }
 
         Ok(manifest)
     }
 }
 
-fn build_context(schema: &JsonSchema, settings: &GeneratorSettings) -> Result<FastAPIContext, GeneratorError> {
-    let mut models = Vec::new();
+// ---------------------------------------------------------------------------
+// Context builder
+// ---------------------------------------------------------------------------
 
-    // Extract models from definitions
-    for (name, def_schema) in schema.all_definitions() {
-        if let Some(model) = extract_model(&name, def_schema) {
-            models.push(model);
-        }
-    }
+fn build_context(
+    schema: &JsonSchema,
+    settings: &GeneratorSettings,
+) -> Result<FastAPIContext, GeneratorError> {
+    // Collect object models from definitions, sorted by name for determinism.
+    let raw_defs: BTreeMap<String, &JsonSchema> = schema
+        .all_definitions()
+        .into_iter()
+        .collect();
 
-    // If root is an object, also create a model for it
+    let mut models: Vec<ModelContext> = raw_defs
+        .iter()
+        .filter_map(|(name, def)| extract_model(name, def))
+        .collect();
+
+    // Also include root-level object if it has a title and properties.
     if let Some(title) = &schema.title {
         if let Some(model) = extract_model(title, schema) {
-            models.push(model);
+            // Avoid duplicates.
+            if !models.iter().any(|m| m.name == model.name) {
+                models.push(model);
+            }
         }
     }
 
+    // Sort for deterministic output.
+    models.sort_by(|a, b| a.name.cmp(&b.name));
+
+    // Build route groups — one per model (cross-section composition:
+    // route handler = rest-api × schema).
+    let routes: Vec<RouteContext> = models
+        .iter()
+        .map(|m| RouteContext {
+            model_name: m.name.clone(),
+            path_prefix: format!("{}s", m.slug),
+            create_schema: format!("{}Create", m.name),
+            update_schema: format!("{}Update", m.name),
+            response_schema: format!("{}Response", m.name),
+        })
+        .collect();
+
     Ok(FastAPIContext {
         project_name: settings.name.clone(),
         version: settings.version.clone(),
         models,
-        routes: Vec::new(), // Routes would come from OpenAPI spec
+        routes,
     })
 }
 
+// ---------------------------------------------------------------------------
+// Model extraction helpers
+// ---------------------------------------------------------------------------
+
 fn extract_model(name: &str, schema: &JsonSchema) -> Option<ModelContext> {
-    let effective_type = schema.effective_type();
-    if effective_type != Some(SchemaType::Object) {
+    if schema.effective_type() != Some(SchemaType::Object) {
         return None;
     }
 
     let properties = schema.properties.as_ref()?;
-    let required_fields: std::collections::HashSet<_> = schema
+    let required_set: std::collections::HashSet<&str> = schema
         .required
         .as_ref()
-        .map(|r| r.iter().collect())
+        .map(|r| r.iter().map(|s| s.as_str()).collect())
         .unwrap_or_default();
 
-    let fields: Vec<_> = properties
+    // Sort fields deterministically.
+    let mut sorted_props: Vec<(&String, &Box<JsonSchema>)> = properties.iter().collect();
+    sorted_props.sort_by_key(|(k, _)| k.as_str());
+
+    let fields: Vec<FieldContext> = sorted_props
         .iter()
-        .map(|(field_name, field_schema)| {
-            let python_type = schema_to_python_type(field_schema);
-            FieldContext {
-                name: field_name.clone(),
-                python_type,
-                required: required_fields.contains(field_name),
-                default: field_schema.default.as_ref().map(|v| format!("{}", v)),
-                description: field_schema.description.clone(),
-            }
+        .map(|(field_name, field_schema)| FieldContext {
+            name: field_name.to_string(),
+            python_type: schema_to_python_type(field_schema),
+            required: required_set.contains(field_name.as_str()),
+            default: field_schema.default.as_ref().map(|v| v.to_string()),
+            description: field_schema.description.clone(),
         })
         .collect();
 
     Some(ModelContext {
-        name: name.to_string(),
+        name: to_pascal_case(name),
+        slug: to_snake_case(name),
         fields,
     })
 }
 
 fn schema_to_python_type(schema: &JsonSchema) -> String {
-    // Handle $ref
+    // Handle $ref.
     if let Some(ref_path) = &schema.ref_ {
-        if let Some(name) = ref_path.strip_prefix("#/definitions/") {
-            return name.to_string();
+        if let Some(n) = ref_path.strip_prefix("#/definitions/") {
+            return to_pascal_case(n);
         }
-        if let Some(name) = ref_path.strip_prefix("#/$defs/") {
-            return name.to_string();
+        if let Some(n) = ref_path.strip_prefix("#/$defs/") {
+            return to_pascal_case(n);
         }
     }
 
     match schema.effective_type() {
-        Some(SchemaType::String) => {
-            if schema.format.is_some() {
-                // Could map formats to specific types
-                "str".to_string()
-            } else {
-                "str".to_string()
-            }
-        }
+        Some(SchemaType::String) => "str".to_string(),
         Some(SchemaType::Integer) => "int".to_string(),
         Some(SchemaType::Number) => "float".to_string(),
         Some(SchemaType::Boolean) => "bool".to_string(),
         Some(SchemaType::Array) => {
-            if let Some(items) = &schema.items {
-                format!("list[{}]", schema_to_python_type(items))
-            } else {
-                "list".to_string()
-            }
+            let inner = schema
+                .items
+                .as_ref()
+                .map(|i| schema_to_python_type(i))
+                .unwrap_or_else(|| "Any".to_string());
+            format!("list[{}]", inner)
         }
         Some(SchemaType::Object) => "dict".to_string(),
         Some(SchemaType::Null) => "None".to_string(),
@@ -212,112 +279,452 @@ fn schema_to_python_type(schema: &JsonSchema) -> String {
     }
 }
 
-/// Generate code inline when templates are not available
-fn generate_inline(context: &FastAPIContext, output: &str) -> Result<String, GeneratorError> {
-    match output {
-        "main.py" => Ok(generate_main_py(context)),
-        "models.py" => Ok(generate_models_py(context)),
-        _ => Err(GeneratorError::TemplateRenderError {
-            template: output.to_string(),
-            message: "Unknown output file".to_string(),
-        }),
-    }
+fn to_pascal_case(s: &str) -> String {
+    use heck::ToPascalCase;
+    s.to_pascal_case()
 }
 
-fn generate_main_py(context: &FastAPIContext) -> String {
+fn to_snake_case(s: &str) -> String {
+    use heck::ToSnakeCase;
+    s.to_snake_case()
+}
+
+// ---------------------------------------------------------------------------
+// Inline code generators (used when Tera templates are not present)
+// ---------------------------------------------------------------------------
+
+/// `app.py` — FastAPI application entry-point.
+fn generate_app_py(ctx: &FastAPIContext) -> String {
+    let router_imports: String = ctx
+        .routes
+        .iter()
+        .map(|r| {
+            format!(
+                "from .routes import router as {}_router\n",
+                to_snake_case(&r.model_name)
+            )
+        })
+        .collect();
+
+    let router_includes: String = ctx
+        .routes
+        .iter()
+        .map(|r| {
+            format!(
+                "app.include_router({name}_router, prefix=\"/{prefix}\", tags=[\"{name}\"])\n",
+                name = to_snake_case(&r.model_name),
+                prefix = r.path_prefix,
+            )
+        })
+        .collect();
+
     format!(
         r#"""{}
 FastAPI Application
 Generated by cclab-sdd
 """
 from fastapi import FastAPI
-from .models import *
-
+{}
 app = FastAPI(
     title="{}",
     version="{}",
 )
 
+{}
 @app.get("/")
-async def root():
-    return {{"message": "Hello from {}"}}
+async def health() -> dict:
+    return {{"status": "ok", "version": "{}"}}
 "#,
-        context.project_name,
-        context.project_name,
-        context.version,
-        context.project_name
+        ctx.project_name,
+        router_imports,
+        ctx.project_name,
+        ctx.version,
+        router_includes,
+        ctx.version,
     )
 }
 
-fn generate_models_py(context: &FastAPIContext) -> String {
-    let mut output = String::from(
+/// `models.py` — Pydantic BaseModel definitions (one class per schema object).
+fn generate_models_py(ctx: &FastAPIContext) -> String {
+    let mut out = format!(
         r#""""
 Pydantic Models
 Generated by cclab-sdd
 """
+from __future__ import annotations
 from pydantic import BaseModel
 from typing import Optional, List, Any
 
-"#,
+"#
     );
 
-    for model in &context.models {
-        output.push_str(&format!("\nclass {}(BaseModel):\n", model.name));
+    for model in &ctx.models {
+        out.push_str(&format!("\nclass {}(BaseModel):\n", model.name));
         if model.fields.is_empty() {
-            output.push_str("    pass\n");
+            out.push_str("    pass\n");
         } else {
             for field in &model.fields {
-                let type_annotation = if field.required {
+                let annotation = if field.required {
                     field.python_type.clone()
                 } else {
                     format!("Optional[{}]", field.python_type)
                 };
-                let default = if field.required {
+                let default_part = if field.required {
                     String::new()
                 } else {
                     " = None".to_string()
                 };
-                output.push_str(&format!("    {}: {}{}\n", field.name, type_annotation, default));
+                if let Some(desc) = &field.description {
+                    out.push_str(&format!("    # {}\n", desc));
+                }
+                out.push_str(&format!("    {}: {}{}\n", field.name, annotation, default_part));
             }
         }
     }
 
-    output
+    out
 }
 
+/// `schemas.py` — Request/Response Pydantic schemas (cross-section composition).
+///
+/// For each model `Foo` this generates:
+/// - `FooCreate`   — request body for POST (all required fields)
+/// - `FooUpdate`   — request body for PUT/PATCH (all fields optional)
+/// - `FooResponse` — response body (extends base model, adds `id`)
+fn generate_schemas_py(ctx: &FastAPIContext) -> String {
+    let model_names: Vec<&str> = ctx.models.iter().map(|m| m.name.as_str()).collect();
+    let import_list = model_names.join(", ");
+
+    let mut out = format!(
+        r#""""
+Request/Response Schemas
+Generated by cclab-sdd
+
+These schemas compose the REST API section with the data model section:
+  route handler = rest-api × schema
+"""
+from __future__ import annotations
+from typing import Optional, Any
+from .models import {import_list}
+
+"#,
+        import_list = import_list,
+    );
+
+    for model in &ctx.models {
+        // FooCreate — same required fields as the base model.
+        out.push_str(&format!("\nclass {}Create({}):\n", model.name, model.name));
+        out.push_str("    \"\"\"Request body for creating a new resource.\"\"\"\n");
+        if model.fields.is_empty() {
+            out.push_str("    pass\n");
+        } else {
+            // Override fields that should be required for creation.
+            let required: Vec<&FieldContext> =
+                model.fields.iter().filter(|f| f.required).collect();
+            if required.is_empty() {
+                out.push_str("    pass\n");
+            } else {
+                for f in &required {
+                    out.push_str(&format!("    {}: {}\n", f.name, f.python_type));
+                }
+            }
+        }
+
+        // FooUpdate — all fields optional.
+        out.push_str(&format!("\nclass {}Update({}):\n", model.name, model.name));
+        out.push_str("    \"\"\"Request body for updating an existing resource (all fields optional).\"\"\"\n");
+        if model.fields.is_empty() {
+            out.push_str("    pass\n");
+        } else {
+            for f in &model.fields {
+                out.push_str(&format!(
+                    "    {}: Optional[{}] = None\n",
+                    f.name, f.python_type
+                ));
+            }
+        }
+
+        // FooResponse — extends base with id field.
+        out.push_str(&format!("\nclass {}Response({}):\n", model.name, model.name));
+        out.push_str("    \"\"\"Response body including server-generated fields.\"\"\"\n");
+        out.push_str("    id: int\n");
+        out.push_str("\n    class Config:\n");
+        out.push_str("        from_attributes = True\n");
+    }
+
+    out
+}
+
+/// `routes.py` — FastAPI `APIRouter` with typed handlers per resource.
+///
+/// This is the cross-section composition result:
+///   route handler = rest-api section × schema section
+fn generate_routes_py(ctx: &FastAPIContext) -> String {
+    if ctx.routes.is_empty() {
+        return format!(
+            r#""""
+Routes
+Generated by cclab-sdd
+"""
+from fastapi import APIRouter
+
+router = APIRouter()
+"#
+        );
+    }
+
+    let schema_imports: String = ctx
+        .routes
+        .iter()
+        .flat_map(|r| {
+            [
+                r.create_schema.clone(),
+                r.update_schema.clone(),
+                r.response_schema.clone(),
+            ]
+        })
+        .collect::<Vec<_>>()
+        .join(", ");
+
+    let mut out = format!(
+        r#""""
+API Routes
+Generated by cclab-sdd
+
+Cross-section composition: route handlers reference both models and schemas.
+"""
+from __future__ import annotations
+from typing import List
+from fastapi import APIRouter, HTTPException, status
+from .schemas import {schema_imports}
+
+router = APIRouter()
+
+"#,
+        schema_imports = schema_imports,
+    );
+
+    for route in &ctx.routes {
+        let model_snake = to_snake_case(&route.model_name);
+
+        out.push_str(&format!(
+            r#"
+# --- {model} resource ---
+
+@router.get("", response_model=List[{resp}])
+async def list_{slug}s() -> List[{resp}]:
+    """List all {model} resources."""
+    # TODO: implement
+    return []
+
+
+@router.get("/{{{slug}_id}}", response_model={resp})
+async def get_{slug}({slug}_id: int) -> {resp}:
+    """Get a {model} by ID."""
+    # TODO: implement
+    raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="{model} not found")
+
+
+@router.post("", response_model={resp}, status_code=status.HTTP_201_CREATED)
+async def create_{slug}(body: {create}) -> {resp}:
+    """Create a new {model}."""
+    # TODO: implement
+    raise HTTPException(status_code=status.HTTP_501_NOT_IMPLEMENTED, detail="Not implemented")
+
+
+@router.put("/{{{slug}_id}}", response_model={resp})
+async def update_{slug}({slug}_id: int, body: {update}) -> {resp}:
+    """Update a {model} by ID."""
+    # TODO: implement
+    raise HTTPException(status_code=status.HTTP_501_NOT_IMPLEMENTED, detail="Not implemented")
+
+
+@router.delete("/{{{slug}_id}}", status_code=status.HTTP_204_NO_CONTENT)
+async def delete_{slug}({slug}_id: int) -> None:
+    """Delete a {model} by ID."""
+    # TODO: implement
+    raise HTTPException(status_code=status.HTTP_501_NOT_IMPLEMENTED, detail="Not implemented")
+
+"#,
+            model = route.model_name,
+            slug = model_snake,
+            resp = route.response_schema,
+            create = route.create_schema,
+            update = route.update_schema,
+        ));
+    }
+
+    out
+}
+
+/// `requirements.txt` — Python project dependencies.
+fn generate_requirements_txt(_ctx: &FastAPIContext) -> String {
+    r#"fastapi>=0.111.0
+pydantic>=2.0.0
+uvicorn[standard]>=0.29.0
+"#
+    .to_string()
+}
+
+// ---------------------------------------------------------------------------
+// Tests
+// ---------------------------------------------------------------------------
+
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::generate::schema::parse_json;
+
+    fn user_schema() -> JsonSchema {
+        parse_json(
+            r#"{
+                "title": "User",
+                "type": "object",
+                "properties": {
+                    "name": { "type": "string", "description": "Full name" },
+                    "age":  { "type": "integer" }
+                },
+                "required": ["name"]
+            }"#,
+        )
+        .unwrap()
+    }
 
     #[test]
     fn test_schema_to_python_type() {
-        let string_schema = JsonSchema::string();
-        assert_eq!(schema_to_python_type(&string_schema), "str");
+        assert_eq!(schema_to_python_type(&JsonSchema::string()), "str");
+        assert_eq!(schema_to_python_type(&JsonSchema::integer()), "int");
+        assert_eq!(
+            schema_to_python_type(&JsonSchema::array(JsonSchema::string())),
+            "list[str]"
+        );
+    }
+
+    #[test]
+    fn test_build_context_models() {
+        let schema = user_schema();
+        let settings = GeneratorSettings::default();
+        let ctx = build_context(&schema, &settings).unwrap();
+
+        assert_eq!(ctx.models.len(), 1);
+        assert_eq!(ctx.models[0].name, "User");
+        assert_eq!(ctx.models[0].fields.len(), 2);
+        // Fields sorted deterministically.
+        assert_eq!(ctx.models[0].fields[0].name, "age");
+        assert_eq!(ctx.models[0].fields[1].name, "name");
+    }
+
+    #[test]
+    fn test_build_context_routes() {
+        let schema = user_schema();
+        let settings = GeneratorSettings::default();
+        let ctx = build_context(&schema, &settings).unwrap();
+
+        assert_eq!(ctx.routes.len(), 1);
+        assert_eq!(ctx.routes[0].model_name, "User");
+        assert_eq!(ctx.routes[0].create_schema, "UserCreate");
+        assert_eq!(ctx.routes[0].update_schema, "UserUpdate");
+        assert_eq!(ctx.routes[0].response_schema, "UserResponse");
+        assert_eq!(ctx.routes[0].path_prefix, "users");
+    }
+
+    #[test]
+    fn test_generate_models_py_content() {
+        let schema = user_schema();
+        let settings = GeneratorSettings::default();
+        let ctx = build_context(&schema, &settings).unwrap();
+        let output = generate_models_py(&ctx);
+
+        assert!(output.contains("class User(BaseModel)"));
+        assert!(output.contains("name: str"));
+        assert!(output.contains("age: Optional[int]"));
+    }
 
-        let int_schema = JsonSchema::integer();
-        assert_eq!(schema_to_python_type(&int_schema), "int");
+    #[test]
+    fn test_generate_schemas_py_content() {
+        let schema = user_schema();
+        let settings = GeneratorSettings::default();
+        let ctx = build_context(&schema, &settings).unwrap();
+        let output = generate_schemas_py(&ctx);
+
+        assert!(output.contains("class UserCreate(User)"));
+        assert!(output.contains("class UserUpdate(User)"));
+        assert!(output.contains("class UserResponse(User)"));
+        // Cross-section composition import
+        assert!(output.contains("from .models import User"));
+    }
+
+    #[test]
+    fn test_generate_routes_py_content() {
+        let schema = user_schema();
+        let settings = GeneratorSettings::default();
+        let ctx = build_context(&schema, &settings).unwrap();
+        let output = generate_routes_py(&ctx);
+
+        assert!(output.contains("router = APIRouter()"));
+        assert!(output.contains("response_model=List[UserResponse]"));
+        assert!(output.contains("body: UserCreate"));
+        assert!(output.contains("body: UserUpdate"));
+        assert!(output.contains("from .schemas import"));
+    }
+
+    #[test]
+    fn test_generate_requirements_txt() {
+        let schema = user_schema();
+        let settings = GeneratorSettings::default();
+        let ctx = build_context(&schema, &settings).unwrap();
+        let output = generate_requirements_txt(&ctx);
+
+        assert!(output.contains("fastapi"));
+        assert!(output.contains("pydantic"));
+        assert!(output.contains("uvicorn"));
+    }
+
+    #[test]
+    fn test_generator_produces_five_files() {
+        let schema = user_schema();
+        let settings = GeneratorSettings {
+            output_dir: std::path::PathBuf::from("/tmp/test_fastapi_gen"),
+            ..Default::default()
+        };
+        let engine = crate::generate::engine::TemplateEngine::empty();
+        let gen = FastAPIGenerator::new();
+        let manifest = gen.generate(&schema, &settings, &engine).unwrap();
 
-        let array_schema = JsonSchema::array(JsonSchema::string());
-        assert_eq!(schema_to_python_type(&array_schema), "list[str]");
+        assert_eq!(manifest.files.len(), 5);
+        let paths: Vec<String> = manifest
+            .files
+            .keys()
+            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
+            .collect();
+        assert!(paths.contains(&"app.py".to_string()));
+        assert!(paths.contains(&"models.py".to_string()));
+        assert!(paths.contains(&"schemas.py".to_string()));
+        assert!(paths.contains(&"routes.py".to_string()));
+        assert!(paths.contains(&"requirements.txt".to_string()));
     }
 
     #[test]
-    fn test_generate_models() {
-        let schema = crate::generate::schema::parse_json(r#"{
-            "title": "User",
-            "type": "object",
-            "properties": {
-                "name": { "type": "string" },
-                "age": { "type": "integer" }
-            },
-            "required": ["name"]
-        }"#).unwrap();
+    fn test_deterministic_output() {
+        let schema = parse_json(
+            r#"{
+                "type": "object",
+                "definitions": {
+                    "Zebra": { "type": "object", "properties": { "z": { "type": "string" } }, "required": ["z"] },
+                    "Apple": { "type": "object", "properties": { "a": { "type": "integer" } } }
+                }
+            }"#,
+        )
+        .unwrap();
 
         let settings = GeneratorSettings::default();
-        let context = build_context(&schema, &settings).unwrap();
+        let ctx1 = build_context(&schema, &settings).unwrap();
+        let ctx2 = build_context(&schema, &settings).unwrap();
 
-        assert_eq!(context.models.len(), 1);
-        assert_eq!(context.models[0].name, "User");
-        assert_eq!(context.models[0].fields.len(), 2);
+        // Models sorted: Apple, Zebra
+        assert_eq!(ctx1.models[0].name, ctx2.models[0].name);
+        assert_eq!(ctx1.models[1].name, ctx2.models[1].name);
+        assert_eq!(ctx1.models[0].name, "Apple");
+        assert_eq!(ctx1.models[1].name, "Zebra");
     }
 }
diff --git a/crates/cclab-sdd/src/generate/generators/mod.rs b/crates/cclab-sdd/src/generate/generators/mod.rs
index 1a9036be..0e863c99 100644
--- a/crates/cclab-sdd/src/generate/generators/mod.rs
+++ b/crates/cclab-sdd/src/generate/generators/mod.rs
@@ -6,8 +6,10 @@ mod common;
 mod fastapi;
 mod express;
 mod axum;
+mod test_generator;
 
 pub use common::{Generator, GeneratorError, GeneratorSettings, GeneratedFile, Manifest, OverwritePolicy};
 pub use fastapi::FastAPIGenerator;
 pub use express::ExpressGenerator;
 pub use axum::AxumGenerator;
+pub use test_generator::{TestGenerator, TestGenResult, TestGenError, CoverageIssue};
diff --git a/crates/cclab-sdd/src/generate/lib.rs b/crates/cclab-sdd/src/generate/lib.rs
index 74360a3b..17f4ddd8 100644
--- a/crates/cclab-sdd/src/generate/lib.rs
+++ b/crates/cclab-sdd/src/generate/lib.rs
@@ -22,7 +22,7 @@ pub use mcp::{SddTools, call_tool, is_sdd_tool};
 pub use schema::{JsonSchema, SchemaType, SchemaVersion};
 pub use engine::{TemplateEngine, TemplateError};
 pub use validator::{validate_schema, ValidationResult, ValidationIssue, Severity};
-pub use generators::{Generator, GeneratorError, GeneratorSettings, Manifest, FastAPIGenerator, ExpressGenerator, AxumGenerator};
+pub use generators::{Generator, GeneratorError, GeneratorSettings, Manifest, FastAPIGenerator, ExpressGenerator, AxumGenerator, TestGenerator, TestGenResult, TestGenError, CoverageIssue};
 pub use spec_ir::{SpecIR, SpecMetadata, SpecBundle, BundleMetadata};
 
 /// Result type for generate operations
diff --git a/crates/cclab-sdd/src/tools/agent.rs b/crates/cclab-sdd/src/tools/agent.rs
index 5ffb2070..02135cb4 100644
--- a/crates/cclab-sdd/src/tools/agent.rs
+++ b/crates/cclab-sdd/src/tools/agent.rs
@@ -276,9 +276,13 @@ fn write_agent_log(
         out
     };
 
-    // Truncate stdout to last 500 chars
-    let stdout_tail = if result.stdout.len() > 500 {
-        &result.stdout[result.stdout.len() - 500..]
+    // Truncate stdout to last 500 chars (char-boundary safe)
+    let stdout_tail: &str = if result.stdout.len() > 500 {
+        let mut start = result.stdout.len() - 500;
+        while !result.stdout.is_char_boundary(start) && start < result.stdout.len() {
+            start += 1;
+        }
+        &result.stdout[start..]
     } else {
         &result.stdout
     };

```

## Review: sdd-codegen-testgen-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-codegen-testgen

**Summary**: TestGenerator implemented with 10 passing tests. Compiles clean. CLI `cclab sdd gen test` wired up with --strict and --dry-run flags. BDD given/when/then mapped to comments+todo!() as specified. Coverage validation with warnings/strict mode. Rust pytest target working.

