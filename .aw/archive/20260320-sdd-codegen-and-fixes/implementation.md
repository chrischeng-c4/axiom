---
id: implementation
type: change_implementation
change_id: sdd-codegen-and-fixes
---

# Implementation

## Summary

Six files changed, two new files added — implements the SpecIR generator protocol and two new section-type generators (deploy + wireframe) for the sdd-codegen-and-fixes change.

1. **SpecIRGenerator trait** (`generators/common.rs`): New trait alongside the existing `Generator` trait. Provides `can_generate(SpecIR) -> bool` for routing, `generate_from_ir(SpecIR, settings, engine) -> Manifest` for output production, and `template_dir()` for Tera template look-ups. SpecIR-backed generators receive structured payloads instead of raw JSON Schema.

2. **DeployGenerator** (`generators/deploy.rs`, new): Implements `SpecIRGenerator` for `SpecIR::Deploy`. Produces two output files: `deployment.yaml` (k8s `apps/v1 Deployment`) and `service.yaml` (`v1 Service` ClusterIP). Supports env vars (literal + `valueFrom`), CPU/memory resource limits, and configurable replica count. Falls back to inline string generation when Tera templates are absent.

3. **ReactGenerator** (`generators/react.rs`, new): Implements `SpecIRGenerator` for `SpecIR::Wireframe`. Produces three output files: `{ComponentName}.tsx` (React functional component with TypeScript), `{ComponentName}.types.ts` (props interface), and `index.ts` (barrel re-export). Recursively renders the wireframe layout tree into JSX lines. Uses `heck::ToPascalCase` for component name normalisation.

4. **SpecIR payload types** (`spec_ir/types.rs`): Added `DeploySpec`, `EnvVar`, `ResourceLimits`, `WireframeSpec`, `PropDef`, `WireframeNode`, `ComponentSpec`, `AttributeDef`, `SlotDef`, `EventDef`, `DesignTokenSpec`, `DesignTokenEntry`. Added `SpecIR::Deploy`, `SpecIR::Wireframe`, `SpecIR::Component`, `SpecIR::DesignToken` enum variants. All types derive `Serialize/Deserialize` with serde defaults and skip-if-empty guards.

5. **Public re-exports** (`generators/mod.rs`, `generate/lib.rs`): `SpecIRGenerator`, `DeployGenerator`, `ReactGenerator`, and all new payload types are now part of the crate's public API surface.

6. **Spec update** (`cclab/specs/cclab-sdd/generate/codegen-system.md`): Version bumped to 2. Added R6 (SpecIRGenerator protocol) and R7 (new section-type generators). Updated class diagram to show `SpecIRGenerator` trait and its implementors. Updated data flow flowchart to show the Router splitting input between JSON Schema generators and SpecIR generators.

## Diff

```diff
diff --git a/cclab/specs/cclab-sdd/generate/codegen-system.md b/cclab/specs/cclab-sdd/generate/codegen-system.md
index 4c3257d1..b6610560 100644
--- a/cclab/specs/cclab-sdd/generate/codegen-system.md
+++ b/cclab/specs/cclab-sdd/generate/codegen-system.md
@@ -2,18 +2,20 @@
 id: generate-codegen-system
 type: spec
 title: "Generate Code Generation System Architecture"
-version: 1
+version: 2
 spec_type: algorithm
 created_at: 2026-02-02T13:49:08.687548+00:00
-updated_at: 2026-02-02T13:49:08.687548+00:00
+updated_at: 2026-03-20T00:00:00.000000+00:00
 requirements:
-  total: 5
+  total: 7
   ids:
     - R1
     - R2
     - R3
     - R4
     - R5
+    - R6
+    - R7
 design_elements:
   has_mermaid: true
   has_json_schema: false
@@ -92,6 +94,35 @@ status: draft
 
 The system must generate corresponding test suites for the generated code.
 
+### R6 - SpecIR Generator Protocol
+
+```yaml
+id: R6
+priority: high
+status: draft
+```
+
+The system must support a `SpecIRGenerator` trait for generators that consume
+structured SpecIR payloads (deploy, wireframe, component, design-token section
+types) rather than raw JSON Schema. Each generator implements `can_generate()`
+for routing and `generate_from_ir()` for output production.
+
+### R7 - New Section-Type Generators
+
+```yaml
+id: R7
+priority: high
+status: draft
+```
+
+The system must include generators for the new section types introduced in this
+change:
+
+| Generator | Input SpecIR | Output |
+|---|---|---|
+| `DeployGenerator` | `SpecIR::Deploy` | Kubernetes `Deployment` + `Service` YAML |
+| `ReactGenerator` | `SpecIR::Wireframe` | React `.tsx` component + `.types.ts` + `index.ts` |
+
 ## Acceptance Criteria
 
 ### Scenario: Generate FastAPI Code
@@ -118,36 +149,64 @@ The system must generate corresponding test suites for the generated code.
 
 ```mermaid
 classDiagram
-    class GenerateCodegen {
+    class Generator {
+        <<trait>>
+        +generate(JsonSchema, Settings, Engine) Manifest
+        +template_dir() str
+    }
+    class SpecIRGenerator {
+        <<trait>>
+        +can_generate(SpecIR) bool
+        +generate_from_ir(SpecIR, Settings, Engine) Manifest
+        +template_dir() str
     }
-    CodegenSystem *-- JsonSchemaCore : uses
-    CodegenSystem *-- SpecValidator : uses
-    CodegenSystem *-- TemplateEngine : uses
-    CodegenSystem o-- Generator : uses
     FastAPIGenerator ..|> Generator : implements
     ExpressGenerator ..|> Generator : implements
     AxumGenerator ..|> Generator : implements
+    DeployGenerator ..|> SpecIRGenerator : implements
+    ReactGenerator ..|> SpecIRGenerator : implements
+    class SpecIR {
+        <<enum>>
+        Api
+        FlowchartPlus
+        ClassPlus
+        ErdPlus
+        SequencePlus
+        RequirementPlus
+        Deploy
+        Wireframe
+        Component
+        DesignToken
+    }
+    SpecIRGenerator --> SpecIR : consumes
 ```
 
 ### Generate Codegen Data Flow
 
 ```mermaid
 flowchart LR
-    InputSpec[Input Spec (JSON/YAML)]
+    InputSpec[Input Spec]
+    Router{Router}
     JsonSchemaCore(JSON Schema Core)
-    SpecValidator{Spec Validator} 
+    SpecIRParse(SpecIR Parser)
+    SpecValidator{Spec Validator}
     TemplateEngine(Template Engine)
-    Generators(Framework Generators)
+    SchemaGenerators(Schema Generators\nFastAPI / Express / Axum)
+    SpecIRGenerators(SpecIR Generators\nDeploy / React)
     TestGenerator(Test Generator)
     GeneratedCode[Generated Code]
     GeneratedTests[Generated Tests]
-    InputSpec -->|Parse| JsonSchemaCore
+    InputSpec -->|route by section type| Router
+    Router -->|rest-api / schema| JsonSchemaCore
+    Router -->|deploy / wireframe / component / design-token| SpecIRParse
     JsonSchemaCore -->|Validate| SpecValidator
     SpecValidator -->|Valid| TemplateEngine
-    TemplateEngine -->|Render| Generators
-    Generators -->|Output| GeneratedCode
-    Generators -.->|Generate| TestGenerator
+    TemplateEngine -->|Render| SchemaGenerators
+    SchemaGenerators -->|Output| GeneratedCode
+    SchemaGenerators -.->|Generate| TestGenerator
     TestGenerator -->|Output| GeneratedTests
+    SpecIRParse -->|SpecIR| SpecIRGenerators
+    SpecIRGenerators -->|Output| GeneratedCode
 ```
 
 <semantic-data>
diff --git a/crates/cclab-sdd/src/generate/generators/common.rs b/crates/cclab-sdd/src/generate/generators/common.rs
index a6a70800..7309b4f3 100644
--- a/crates/cclab-sdd/src/generate/generators/common.rs
+++ b/crates/cclab-sdd/src/generate/generators/common.rs
@@ -144,7 +144,7 @@ impl Manifest {
     }
 }
 
-/// Trait for code generators
+/// Trait for code generators backed by JSON Schema / OpenAPI input.
 pub trait Generator {
     /// Generate code from a schema
     fn generate(
@@ -157,3 +157,23 @@ pub trait Generator {
     /// Get the template subdirectory name
     fn template_dir(&self) -> &'static str;
 }
+
+/// Trait for code generators that consume [`crate::generate::spec_ir::SpecIR`] directly.
+///
+/// Used by the new section-type generators (deploy, wireframe, component,
+/// design-token) that receive structured spec payloads rather than raw JSON Schema.
+pub trait SpecIRGenerator {
+    /// Return `true` if this generator can handle the given [`SpecIR`] variant.
+    fn can_generate(&self, spec: &crate::generate::spec_ir::SpecIR) -> bool;
+
+    /// Generate code from a [`SpecIR`] item.
+    fn generate_from_ir(
+        &self,
+        spec: &crate::generate::spec_ir::SpecIR,
+        settings: &GeneratorSettings,
+        engine: &TemplateEngine,
+    ) -> Result<Manifest, GeneratorError>;
+
+    /// Get the template subdirectory name used for Tera template look-ups.
+    fn template_dir(&self) -> &'static str;
+}
diff --git a/crates/cclab-sdd/src/generate/generators/deploy.rs b/crates/cclab-sdd/src/generate/generators/deploy.rs
new file mode 100644
index 00000000..28fb41c8
--- /dev/null
+++ b/crates/cclab-sdd/src/generate/generators/deploy.rs
@@ -0,0 +1,418 @@
+//! Kubernetes Deployment + Service manifest generator
+//!
+//! Generates Kubernetes manifests from a [`DeploySpec`] (deploy section type):
+//!
+//! | Output file        | Description                                    |
+//! |--------------------|------------------------------------------------|
+//! | `deployment.yaml`  | `apps/v1 Deployment` resource                  |
+//! | `service.yaml`     | `v1 Service` (ClusterIP) resource              |
+//!
+//! The generator implements [`SpecIRGenerator`] and only accepts
+//! [`SpecIR::Deploy`] variants.
+
+use super::common::{
+    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
+};
+use crate::generate::engine::TemplateEngine;
+use crate::generate::spec_ir::{DeploySpec, EnvVar, SpecIR};
+use serde::Serialize;
+
+// ---------------------------------------------------------------------------
+// DeployGenerator
+// ---------------------------------------------------------------------------
+
+/// Generates Kubernetes `Deployment` + `Service` YAML manifests from a
+/// [`SpecIR::Deploy`] item.
+pub struct DeployGenerator;
+
+impl DeployGenerator {
+    pub fn new() -> Self {
+        Self
+    }
+}
+
+impl Default for DeployGenerator {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+// ---------------------------------------------------------------------------
+// Template context
+// ---------------------------------------------------------------------------
+
+#[derive(Debug, Serialize)]
+struct DeployContext {
+    name: String,
+    image: String,
+    port: u16,
+    replicas: u32,
+    namespace: String,
+    env: Vec<EnvVarContext>,
+    cpu_limit: Option<String>,
+    memory_limit: Option<String>,
+}
+
+#[derive(Debug, Serialize)]
+struct EnvVarContext {
+    name: String,
+    /// `true` if a literal `value:` should be emitted
+    is_literal: bool,
+    value: Option<String>,
+    /// Secret/ConfigMap reference string (rendered verbatim in the manifest)
+    value_from: Option<String>,
+}
+
+// ---------------------------------------------------------------------------
+// SpecIRGenerator impl
+// ---------------------------------------------------------------------------
+
+impl SpecIRGenerator for DeployGenerator {
+    fn can_generate(&self, spec: &SpecIR) -> bool {
+        matches!(spec, SpecIR::Deploy { .. })
+    }
+
+    fn template_dir(&self) -> &'static str {
+        "deploy"
+    }
+
+    fn generate_from_ir(
+        &self,
+        spec: &SpecIR,
+        settings: &GeneratorSettings,
+        engine: &TemplateEngine,
+    ) -> Result<Manifest, GeneratorError> {
+        let deploy_spec = match spec {
+            SpecIR::Deploy { spec, .. } => spec,
+            _ => {
+                return Err(GeneratorError::SchemaError(
+                    "DeployGenerator: expected SpecIR::Deploy variant".into(),
+                ))
+            }
+        };
+
+        let mut manifest = Manifest::new();
+        let ctx = build_context(deploy_spec, settings);
+
+        let files: &[(&str, &str, fn(&DeployContext) -> String)] = &[
+            ("deployment.yaml.j2", "deployment.yaml", generate_deployment_yaml),
+            ("service.yaml.j2", "service.yaml", generate_service_yaml),
+        ];
+
+        for (template, output, inline_gen) in files {
+            let output_path = settings.output_dir.join(output);
+
+            if output_path.exists() {
+                match settings.overwrite_policy {
+                    OverwritePolicy::Error => {
+                        return Err(GeneratorError::OverwriteNotAllowed(output_path));
+                    }
+                    OverwritePolicy::Skip => {
+                        manifest.add(GeneratedFile::skipped(output_path));
+                        continue;
+                    }
+                    OverwritePolicy::Overwrite => {}
+                }
+            }
+
+            let template_name = format!("{}/{}", self.template_dir(), template);
+            let content = if engine.has_template(&template_name) {
+                engine.render(&template_name, &ctx).map_err(|e| {
+                    GeneratorError::TemplateRenderError {
+                        template: template_name.clone(),
+                        message: e.to_string(),
+                    }
+                })?
+            } else {
+                inline_gen(&ctx)
+            };
+
+            manifest.add(GeneratedFile::written(output_path, &content));
+        }
+
+        Ok(manifest)
+    }
+}
+
+// ---------------------------------------------------------------------------
+// Context builder
+// ---------------------------------------------------------------------------
+
+fn build_context(spec: &DeploySpec, settings: &GeneratorSettings) -> DeployContext {
+    let name = if spec.name.is_empty() {
+        settings.name.clone()
+    } else {
+        spec.name.clone()
+    };
+
+    let env = spec
+        .env
+        .iter()
+        .map(|e| env_var_context(e))
+        .collect();
+
+    let (cpu_limit, memory_limit) = spec
+        .resources
+        .as_ref()
+        .map(|r| (r.cpu.clone(), r.memory.clone()))
+        .unwrap_or((None, None));
+
+    DeployContext {
+        name,
+        image: spec.image.clone(),
+        port: spec.port,
+        replicas: spec.replicas,
+        namespace: "default".to_string(),
+        env,
+        cpu_limit,
+        memory_limit,
+    }
+}
+
+fn env_var_context(e: &EnvVar) -> EnvVarContext {
+    EnvVarContext {
+        name: e.name.clone(),
+        is_literal: e.value.is_some(),
+        value: e.value.clone(),
+        value_from: e.value_from.clone(),
+    }
+}
+
+// ---------------------------------------------------------------------------
+// Inline generators (used when Tera templates are absent)
+// ---------------------------------------------------------------------------
+
+fn generate_deployment_yaml(ctx: &DeployContext) -> String {
+    let env_block = if ctx.env.is_empty() {
+        String::new()
+    } else {
+        let mut block = "          env:\n".to_string();
+        for e in &ctx.env {
+            if e.is_literal {
+                block.push_str(&format!(
+                    "            - name: {}\n              value: \"{}\"\n",
+                    e.name,
+                    e.value.as_deref().unwrap_or("")
+                ));
+            } else if let Some(vf) = &e.value_from {
+                block.push_str(&format!(
+                    "            - name: {}\n              valueFrom: {}\n",
+                    e.name, vf
+                ));
+            } else {
+                block.push_str(&format!("            - name: {}\n", e.name));
+            }
+        }
+        block
+    };
+
+    let resources_block = build_resources_block(ctx);
+
+    format!(
+        r#"# Generated by cclab-sdd
+apiVersion: apps/v1
+kind: Deployment
+metadata:
+  name: {name}
+  namespace: {namespace}
+  labels:
+    app: {name}
+spec:
+  replicas: {replicas}
+  selector:
+    matchLabels:
+      app: {name}
+  template:
+    metadata:
+      labels:
+        app: {name}
+    spec:
+      containers:
+        - name: {name}
+          image: {image}
+          ports:
+            - containerPort: {port}
+{env_block}{resources_block}
+"#,
+        name = ctx.name,
+        namespace = ctx.namespace,
+        replicas = ctx.replicas,
+        image = ctx.image,
+        port = ctx.port,
+        env_block = env_block,
+        resources_block = resources_block,
+    )
+}
+
+fn build_resources_block(ctx: &DeployContext) -> String {
+    if ctx.cpu_limit.is_none() && ctx.memory_limit.is_none() {
+        return String::new();
+    }
+    let mut block = "          resources:\n            limits:\n".to_string();
+    if let Some(cpu) = &ctx.cpu_limit {
+        block.push_str(&format!("              cpu: \"{}\"\n", cpu));
+    }
+    if let Some(mem) = &ctx.memory_limit {
+        block.push_str(&format!("              memory: \"{}\"\n", mem));
+    }
+    block
+}
+
+fn generate_service_yaml(ctx: &DeployContext) -> String {
+    format!(
+        r#"# Generated by cclab-sdd
+apiVersion: v1
+kind: Service
+metadata:
+  name: {name}
+  namespace: {namespace}
+  labels:
+    app: {name}
+spec:
+  selector:
+    app: {name}
+  ports:
+    - protocol: TCP
+      port: {port}
+      targetPort: {port}
+  type: ClusterIP
+"#,
+        name = ctx.name,
+        namespace = ctx.namespace,
+        port = ctx.port,
+    )
+}
+
+// ---------------------------------------------------------------------------
+// Tests
+// ---------------------------------------------------------------------------
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::generate::spec_ir::{DeploySpec, EnvVar, ResourceLimits, SpecIR, SpecMetadata};
+
+    fn simple_spec() -> SpecIR {
+        SpecIR::Deploy {
+            spec: DeploySpec {
+                name: "my-app".into(),
+                image: "my-app:1.0".into(),
+                port: 3000,
+                replicas: 2,
+                env: vec![
+                    EnvVar { name: "ENV".into(), value: Some("production".into()), value_from: None },
+                ],
+                resources: None,
+            },
+            metadata: SpecMetadata::default(),
+        }
+    }
+
+    #[test]
+    fn test_can_generate_deploy() {
+        let gen = DeployGenerator::new();
+        assert!(gen.can_generate(&simple_spec()));
+    }
+
+    #[test]
+    fn test_cannot_generate_non_deploy() {
+        use crate::generate::schema::JsonSchema;
+        let gen = DeployGenerator::new();
+        let api_spec = SpecIR::Api {
+            schema: JsonSchema::default(),
+            metadata: SpecMetadata::default(),
+        };
+        assert!(!gen.can_generate(&api_spec));
+    }
+
+    #[test]
+    fn test_generate_produces_two_files() {
+        let spec = simple_spec();
+        let settings = GeneratorSettings {
+            output_dir: std::path::PathBuf::from("/tmp/test_deploy_gen"),
+            ..Default::default()
+        };
+        let engine = crate::generate::engine::TemplateEngine::empty();
+        let gen = DeployGenerator::new();
+        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();
+
+        assert_eq!(manifest.files.len(), 2);
+        let names: Vec<String> = manifest
+            .files
+            .keys()
+            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
+            .collect();
+        assert!(names.contains(&"deployment.yaml".to_string()));
+        assert!(names.contains(&"service.yaml".to_string()));
+    }
+
+    #[test]
+    fn test_deployment_yaml_content() {
+        let spec = simple_spec();
+        let settings = GeneratorSettings {
+            output_dir: std::path::PathBuf::from("/tmp/test_deploy_gen_content"),
+            ..Default::default()
+        };
+        let engine = crate::generate::engine::TemplateEngine::empty();
+        let gen = DeployGenerator::new();
+        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();
+
+        let deployment = manifest
+            .files
+            .values()
+            .find(|f| f.path.file_name().unwrap() == "deployment.yaml")
+            .expect("deployment.yaml not found");
+
+        // Content should be in manifest (content hash set)
+        assert!(deployment.content_hash.is_some());
+    }
+
+    #[test]
+    fn test_deployment_yaml_with_resources() {
+        let spec = SpecIR::Deploy {
+            spec: DeploySpec {
+                name: "api".into(),
+                image: "api:latest".into(),
+                port: 8080,
+                replicas: 3,
+                env: vec![],
+                resources: Some(ResourceLimits {
+                    cpu: Some("500m".into()),
+                    memory: Some("256Mi".into()),
+                }),
+            },
+            metadata: SpecMetadata::default(),
+        };
+        let ctx = build_context(
+            match &spec {
+                SpecIR::Deploy { spec, .. } => spec,
+                _ => unreachable!(),
+            },
+            &GeneratorSettings::default(),
+        );
+
+        let yaml = generate_deployment_yaml(&ctx);
+        assert!(yaml.contains("resources:"));
+        assert!(yaml.contains("cpu: \"500m\""));
+        assert!(yaml.contains("memory: \"256Mi\""));
+    }
+
+    #[test]
+    fn test_service_yaml_content() {
+        let ctx = DeployContext {
+            name: "my-svc".into(),
+            image: "my-svc:1.0".into(),
+            port: 9090,
+            replicas: 1,
+            namespace: "default".into(),
+            env: vec![],
+            cpu_limit: None,
+            memory_limit: None,
+        };
+        let yaml = generate_service_yaml(&ctx);
+        assert!(yaml.contains("kind: Service"));
+        assert!(yaml.contains("name: my-svc"));
+        assert!(yaml.contains("port: 9090"));
+        assert!(yaml.contains("type: ClusterIP"));
+    }
+}
diff --git a/crates/cclab-sdd/src/generate/generators/mod.rs b/crates/cclab-sdd/src/generate/generators/mod.rs
index 0e863c99..70b7034d 100644
--- a/crates/cclab-sdd/src/generate/generators/mod.rs
+++ b/crates/cclab-sdd/src/generate/generators/mod.rs
@@ -1,15 +1,30 @@
 //! Code Generators
 //!
 //! Framework-specific code generators using the template engine.
+//!
+//! ## Generator families
+//!
+//! ### JSON Schema / OpenAPI generators ([`Generator`] trait)
+//! - [`FastAPIGenerator`] — Python / FastAPI
+//! - [`ExpressGenerator`] — TypeScript / Express
+//! - [`AxumGenerator`]    — Rust / Axum
+//!
+//! ### SpecIR generators ([`SpecIRGenerator`] trait)
+//! - [`DeployGenerator`]  — `deploy` section type → Kubernetes Deployment + Service YAML
+//! - [`ReactGenerator`]   — `wireframe` section type → React functional component scaffold
 
 mod common;
 mod fastapi;
 mod express;
 mod axum;
 mod test_generator;
+mod deploy;
+mod react;
 
-pub use common::{Generator, GeneratorError, GeneratorSettings, GeneratedFile, Manifest, OverwritePolicy};
+pub use common::{Generator, GeneratorError, GeneratorSettings, GeneratedFile, Manifest, OverwritePolicy, SpecIRGenerator};
 pub use fastapi::FastAPIGenerator;
 pub use express::ExpressGenerator;
 pub use axum::AxumGenerator;
 pub use test_generator::{TestGenerator, TestGenResult, TestGenError, CoverageIssue};
+pub use deploy::DeployGenerator;
+pub use react::ReactGenerator;
diff --git a/crates/cclab-sdd/src/generate/generators/react.rs b/crates/cclab-sdd/src/generate/generators/react.rs
new file mode 100644
index 00000000..458951f7
--- /dev/null
+++ b/crates/cclab-sdd/src/generate/generators/react.rs
@@ -0,0 +1,543 @@
+//! React component scaffold generator
+//!
+//! Generates a React functional component scaffold from a [`WireframeSpec`]
+//! (wireframe section type):
+//!
+//! | Output file                   | Description                                 |
+//! |-------------------------------|---------------------------------------------|
+//! | `{ComponentName}.tsx`         | React functional component (TypeScript)     |
+//! | `{ComponentName}.types.ts`    | TypeScript props interface                  |
+//! | `index.ts`                    | Barrel re-export                            |
+//!
+//! The generator implements [`SpecIRGenerator`] and only accepts
+//! [`SpecIR::Wireframe`] variants.
+
+use super::common::{
+    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
+};
+use crate::generate::engine::TemplateEngine;
+use crate::generate::spec_ir::{PropDef, SpecIR, WireframeNode, WireframeSpec};
+use serde::Serialize;
+
+// ---------------------------------------------------------------------------
+// ReactGenerator
+// ---------------------------------------------------------------------------
+
+/// Generates a React functional component scaffold from a
+/// [`SpecIR::Wireframe`] item.
+pub struct ReactGenerator;
+
+impl ReactGenerator {
+    pub fn new() -> Self {
+        Self
+    }
+}
+
+impl Default for ReactGenerator {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+// ---------------------------------------------------------------------------
+// Template context
+// ---------------------------------------------------------------------------
+
+#[derive(Debug, Serialize)]
+struct ReactContext {
+    /// PascalCase component name
+    component_name: String,
+    /// Props interface name (e.g. `"UserCardProps"`)
+    props_type: String,
+    /// High-level component type hint
+    component_type: String,
+    /// Typed props list
+    props: Vec<PropContext>,
+    /// JSX body lines (simplified render tree)
+    jsx_body: Vec<String>,
+}
+
+#[derive(Debug, Serialize)]
+struct PropContext {
+    name: String,
+    ts_type: String,
+    required: bool,
+    default_value: Option<String>,
+    description: Option<String>,
+}
+
+// ---------------------------------------------------------------------------
+// SpecIRGenerator impl
+// ---------------------------------------------------------------------------
+
+impl SpecIRGenerator for ReactGenerator {
+    fn can_generate(&self, spec: &SpecIR) -> bool {
+        matches!(spec, SpecIR::Wireframe { .. })
+    }
+
+    fn template_dir(&self) -> &'static str {
+        "react"
+    }
+
+    fn generate_from_ir(
+        &self,
+        spec: &SpecIR,
+        settings: &GeneratorSettings,
+        engine: &TemplateEngine,
+    ) -> Result<Manifest, GeneratorError> {
+        let wireframe_spec = match spec {
+            SpecIR::Wireframe { spec, .. } => spec,
+            _ => {
+                return Err(GeneratorError::SchemaError(
+                    "ReactGenerator: expected SpecIR::Wireframe variant".into(),
+                ))
+            }
+        };
+
+        let mut manifest = Manifest::new();
+        let ctx = build_context(wireframe_spec, settings);
+
+        let component_name = ctx.component_name.clone();
+
+        let files: Vec<(String, String, Box<dyn Fn(&ReactContext) -> String>)> = vec![
+            (
+                format!("{}.tsx.j2", component_name),
+                format!("{}.tsx", component_name),
+                Box::new(|c| generate_component_tsx(c)),
+            ),
+            (
+                format!("{}.types.ts.j2", component_name),
+                format!("{}.types.ts", component_name),
+                Box::new(|c| generate_types_ts(c)),
+            ),
+            (
+                "index.ts.j2".to_string(),
+                "index.ts".to_string(),
+                Box::new(|c| generate_index_ts(c)),
+            ),
+        ];
+
+        for (template, output, inline_gen) in &files {
+            let output_path = settings.output_dir.join(output);
+
+            if output_path.exists() {
+                match settings.overwrite_policy {
+                    OverwritePolicy::Error => {
+                        return Err(GeneratorError::OverwriteNotAllowed(output_path));
+                    }
+                    OverwritePolicy::Skip => {
+                        manifest.add(GeneratedFile::skipped(output_path));
+                        continue;
+                    }
+                    OverwritePolicy::Overwrite => {}
+                }
+            }
+
+            let template_name = format!("{}/{}", self.template_dir(), template);
+            let content = if engine.has_template(&template_name) {
+                engine.render(&template_name, &ctx).map_err(|e| {
+                    GeneratorError::TemplateRenderError {
+                        template: template_name.clone(),
+                        message: e.to_string(),
+                    }
+                })?
+            } else {
+                inline_gen(&ctx)
+            };
+
+            manifest.add(GeneratedFile::written(output_path, &content));
+        }
+
+        Ok(manifest)
+    }
+}
+
+// ---------------------------------------------------------------------------
+// Context builder
+// ---------------------------------------------------------------------------
+
+fn build_context(spec: &WireframeSpec, settings: &GeneratorSettings) -> ReactContext {
+    let component_name = if spec.name.is_empty() {
+        to_pascal_case(&settings.name)
+    } else {
+        to_pascal_case(&spec.name)
+    };
+
+    let props_type = format!("{}Props", component_name);
+
+    let props: Vec<PropContext> = spec
+        .props
+        .iter()
+        .map(|p| prop_context(p))
+        .collect();
+
+    let jsx_body = render_jsx_body(&spec.layout, 2);
+
+    ReactContext {
+        component_name,
+        props_type,
+        component_type: spec.component_type.clone(),
+        props,
+        jsx_body,
+    }
+}
+
+fn prop_context(p: &PropDef) -> PropContext {
+    PropContext {
+        name: p.name.clone(),
+        ts_type: p.prop_type.clone(),
+        required: p.required,
+        default_value: p.default_value.clone(),
+        description: p.description.clone(),
+    }
+}
+
+/// Recursively render a wireframe layout tree into JSX line strings.
+fn render_jsx_body(nodes: &[WireframeNode], indent: usize) -> Vec<String> {
+    let pad = "  ".repeat(indent);
+    let mut lines = Vec::new();
+
+    for node in nodes {
+        let label = node
+            .label
+            .as_deref()
+            .map(|l| format!(" /* {} */", l))
+            .unwrap_or_default();
+
+        match node.kind.as_str() {
+            "text" => {
+                lines.push(format!(
+                    "{}<span{}>{}</span>",
+                    pad,
+                    label,
+                    node.label.as_deref().unwrap_or("TODO")
+                ));
+            }
+            "button" => {
+                lines.push(format!(
+                    "{}<button type=\"button\"{}>{}</button>",
+                    pad,
+                    label,
+                    node.label.as_deref().unwrap_or("Action")
+                ));
+            }
+            "input" => {
+                lines.push(format!(
+                    "{}<input placeholder=\"{}\" />",
+                    pad,
+                    node.label.as_deref().unwrap_or("")
+                ));
+            }
+            "list" => {
+                lines.push(format!("{}<ul{}>", pad, label));
+                lines.push(format!("{}  {{/* TODO: render items */}}", pad));
+                lines.push(format!("{}</ul>", pad));
+            }
+            kind => {
+                // Generic container
+                let tag = match kind {
+                    "section" | "article" | "header" | "footer" | "main" | "nav" | "aside" => {
+                        kind.to_string()
+                    }
+                    "form" => "form".to_string(),
+                    _ => "div".to_string(),
+                };
+                if node.children.is_empty() {
+                    lines.push(format!("{}<{}{} />", pad, tag, label));
+                } else {
+                    lines.push(format!("{}<{}{}>", pad, tag, label));
+                    lines.extend(render_jsx_body(&node.children, indent + 1));
+                    lines.push(format!("{}</{}>", pad, tag));
+                }
+            }
+        }
+    }
+
+    lines
+}
+
+// ---------------------------------------------------------------------------
+// Inline generators
+// ---------------------------------------------------------------------------
+
+fn generate_component_tsx(ctx: &ReactContext) -> String {
+    let import_line = format!(
+        "import type {{ {} }} from \"./{}.types\";",
+        ctx.props_type, ctx.component_name
+    );
+
+    // Build props destructuring
+    let destructure = if ctx.props.is_empty() {
+        String::new()
+    } else {
+        let parts: Vec<String> = ctx
+            .props
+            .iter()
+            .map(|p| {
+                if let Some(default) = &p.default_value {
+                    format!("{} = {}", p.name, default)
+                } else {
+                    p.name.clone()
+                }
+            })
+            .collect();
+        format!("{{ {} }}", parts.join(", "))
+    };
+
+    let props_param = if ctx.props.is_empty() {
+        "_props: {}".to_string()
+    } else {
+        format!("{}: {}", destructure, ctx.props_type)
+    };
+
+    // Build JSX body
+    let body = if ctx.jsx_body.is_empty() {
+        "    {/* TODO: implement */}".to_string()
+    } else {
+        ctx.jsx_body.join("\n")
+    };
+
+    // Component type comment
+    let type_comment = if ctx.component_type.is_empty() {
+        String::new()
+    } else {
+        format!("/** @componentType {} */\n", ctx.component_type)
+    };
+
+    format!(
+        r#"// Generated by cclab-sdd
+{import_line}
+
+{type_comment}export function {name}({props_param}) {{
+  return (
+    <div>
+{body}
+    </div>
+  );
+}}
+
+export default {name};
+"#,
+        import_line = import_line,
+        type_comment = type_comment,
+        name = ctx.component_name,
+        props_param = props_param,
+        body = body,
+    )
+}
+
+fn generate_types_ts(ctx: &ReactContext) -> String {
+    let props_body = if ctx.props.is_empty() {
+        "  // No props defined".to_string()
+    } else {
+        ctx.props
+            .iter()
+            .map(|p| {
+                let optional = if p.required { "" } else { "?" };
+                let doc = p
+                    .description
+                    .as_ref()
+                    .map(|d| format!("  /** {} */\n", d))
+                    .unwrap_or_default();
+                format!("{}  {}{}: {};", doc, p.name, optional, p.ts_type)
+            })
+            .collect::<Vec<_>>()
+            .join("\n")
+    };
+
+    format!(
+        r#"// Generated by cclab-sdd
+
+export interface {props_type} {{
+{props_body}
+}}
+"#,
+        props_type = ctx.props_type,
+        props_body = props_body,
+    )
+}
+
+fn generate_index_ts(ctx: &ReactContext) -> String {
+    format!(
+        r#"// Generated by cclab-sdd
+export {{ {name}, default }} from "./{name}";
+export type {{ {props_type} }} from "./{name}.types";
+"#,
+        name = ctx.component_name,
+        props_type = ctx.props_type,
+    )
+}
+
+// ---------------------------------------------------------------------------
+// Case helpers
+// ---------------------------------------------------------------------------
+
+fn to_pascal_case(s: &str) -> String {
+    use heck::ToPascalCase;
+    s.to_pascal_case()
+}
+
+// ---------------------------------------------------------------------------
+// Tests
+// ---------------------------------------------------------------------------
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::generate::spec_ir::{PropDef, SpecIR, SpecMetadata, WireframeNode, WireframeSpec};
+
+    fn card_spec() -> SpecIR {
+        SpecIR::Wireframe {
+            spec: WireframeSpec {
+                name: "UserCard".into(),
+                component_type: "card".into(),
+                props: vec![
+                    PropDef {
+                        name: "userId".into(),
+                        prop_type: "number".into(),
+                        required: true,
+                        default_value: None,
+                        description: Some("The user ID to display".into()),
+                    },
+                    PropDef {
+                        name: "showAvatar".into(),
+                        prop_type: "boolean".into(),
+                        required: false,
+                        default_value: Some("true".into()),
+                        description: None,
+                    },
+                ],
+                layout: vec![
+                    WireframeNode {
+                        kind: "div".into(),
+                        label: Some("card-body".into()),
+                        children: vec![
+                            WireframeNode {
+                                kind: "text".into(),
+                                label: Some("User name".into()),
+                                children: vec![],
+                            },
+                            WireframeNode {
+                                kind: "button".into(),
+                                label: Some("Edit".into()),
+                                children: vec![],
+                            },
+                        ],
+                    },
+                ],
+            },
+            metadata: SpecMetadata::default(),
+        }
+    }
+
+    #[test]
+    fn test_can_generate_wireframe() {
+        let gen = ReactGenerator::new();
+        assert!(gen.can_generate(&card_spec()));
+    }
+
+    #[test]
+    fn test_cannot_generate_non_wireframe() {
+        use crate::generate::schema::JsonSchema;
+        let gen = ReactGenerator::new();
+        let api_spec = SpecIR::Api {
+            schema: JsonSchema::default(),
+            metadata: SpecMetadata::default(),
+        };
+        assert!(!gen.can_generate(&api_spec));
+    }
+
+    #[test]
+    fn test_generate_produces_three_files() {
+        let spec = card_spec();
+        let settings = GeneratorSettings {
+            output_dir: std::path::PathBuf::from("/tmp/test_react_gen"),
+            ..Default::default()
+        };
+        let engine = crate::generate::engine::TemplateEngine::empty();
+        let gen = ReactGenerator::new();
+        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();
+
+        assert_eq!(manifest.files.len(), 3);
+        let names: Vec<String> = manifest
+            .files
+            .keys()
+            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
+            .collect();
+        assert!(names.contains(&"UserCard.tsx".to_string()));
+        assert!(names.contains(&"UserCard.types.ts".to_string()));
+        assert!(names.contains(&"index.ts".to_string()));
+    }
+
+    #[test]
+    fn test_component_tsx_content() {
+        let spec = card_spec();
+        let settings = GeneratorSettings {
+            output_dir: std::path::PathBuf::from("/tmp/test_react_content"),
+            ..Default::default()
+        };
+        let engine = crate::generate::engine::TemplateEngine::empty();
+        let gen = ReactGenerator::new();
+        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();
+
+        let tsx = manifest
+            .files
+            .values()
+            .find(|f| f.path.file_name().unwrap() == "UserCard.tsx")
+            .expect("UserCard.tsx not found");
+
+        // Content hash should be set (meaning content was written)
+        assert!(tsx.content_hash.is_some());
+    }
+
+    #[test]
+    fn test_types_ts_props_interface() {
+        let ctx = build_context(
+            match &card_spec() {
+                SpecIR::Wireframe { spec, .. } => spec,
+                _ => unreachable!(),
+            },
+            &GeneratorSettings::default(),
+        );
+
+        let types_output = generate_types_ts(&ctx);
+        assert!(types_output.contains("interface UserCardProps"));
+        assert!(types_output.contains("userId: number"));
+        assert!(types_output.contains("showAvatar?: boolean"));
+    }
+
+    #[test]
+    fn test_index_ts_exports() {
+        let ctx = build_context(
+            match &card_spec() {
+                SpecIR::Wireframe { spec, .. } => spec,
+                _ => unreachable!(),
+            },
+            &GeneratorSettings::default(),
+        );
+
+        let index_output = generate_index_ts(&ctx);
+        assert!(index_output.contains("UserCard"));
+        assert!(index_output.contains("UserCardProps"));
+    }
+
+    #[test]
+    fn test_jsx_body_render() {
+        let nodes = vec![
+            WireframeNode {
+                kind: "text".into(),
+                label: Some("Hello".into()),
+                children: vec![],
+            },
+            WireframeNode {
+                kind: "button".into(),
+                label: Some("Click me".into()),
+                children: vec![],
+            },
+        ];
+        let lines = render_jsx_body(&nodes, 2);
+        assert!(!lines.is_empty());
+        assert!(lines.iter().any(|l| l.contains("<span")));
+        assert!(lines.iter().any(|l| l.contains("<button")));
+    }
+}
diff --git a/crates/cclab-sdd/src/generate/lib.rs b/crates/cclab-sdd/src/generate/lib.rs
index 17f4ddd8..437840f3 100644
--- a/crates/cclab-sdd/src/generate/lib.rs
+++ b/crates/cclab-sdd/src/generate/lib.rs
@@ -22,8 +22,21 @@ pub use mcp::{SddTools, call_tool, is_sdd_tool};
 pub use schema::{JsonSchema, SchemaType, SchemaVersion};
 pub use engine::{TemplateEngine, TemplateError};
 pub use validator::{validate_schema, ValidationResult, ValidationIssue, Severity};
-pub use generators::{Generator, GeneratorError, GeneratorSettings, Manifest, FastAPIGenerator, ExpressGenerator, AxumGenerator, TestGenerator, TestGenResult, TestGenError, CoverageIssue};
-pub use spec_ir::{SpecIR, SpecMetadata, SpecBundle, BundleMetadata};
+pub use generators::{
+    Generator, GeneratorError, GeneratorSettings, Manifest,
+    FastAPIGenerator, ExpressGenerator, AxumGenerator,
+    TestGenerator, TestGenResult, TestGenError, CoverageIssue,
+    // SpecIR-based generators (deploy, wireframe section types)
+    SpecIRGenerator, DeployGenerator, ReactGenerator,
+};
+pub use spec_ir::{
+    SpecIR, SpecMetadata, SpecBundle, BundleMetadata,
+    // New spec payload types (deploy / wireframe / component / design-token section types)
+    DeploySpec, EnvVar, ResourceLimits,
+    WireframeSpec, PropDef, WireframeNode,
+    ComponentSpec, AttributeDef, SlotDef, EventDef,
+    DesignTokenSpec, DesignTokenEntry,
+};
 
 /// Result type for generate operations
 pub type Result<T> = std::result::Result<T, GenerateError>;
diff --git a/crates/cclab-sdd/src/generate/spec_ir/mod.rs b/crates/cclab-sdd/src/generate/spec_ir/mod.rs
index 1c5fd15d..14d4a204 100644
--- a/crates/cclab-sdd/src/generate/spec_ir/mod.rs
+++ b/crates/cclab-sdd/src/generate/spec_ir/mod.rs
@@ -2,7 +2,22 @@
 //!
 //! The universal contract between SDD generate (spec format) and Lens (code generation).
 //! SpecIR wraps diagram and schema types into a unified enum that
-//! generators can consume via `can_generate()` / `generate()`.
+//! generators can consume via `can_generate()` / `generate_from_ir()`.
+//!
+//! ## Variants
+//!
+//! | Variant | Section type | Generator |
+//! |---------|-------------|-----------|
+//! | `Api` | `rest-api` / `schema` | `FastAPIGenerator`, `ExpressGenerator`, `AxumGenerator` |
+//! | `FlowchartPlus` | `logic` (flowchart) | — |
+//! | `ClassPlus` | `logic` (class) | — |
+//! | `ErdPlus` | `db-model` | — |
+//! | `SequencePlus` | `interaction` | — |
+//! | `RequirementPlus` | `test-plan` | `TestGenerator` |
+//! | `Deploy` | `deploy` | `DeployGenerator` |
+//! | `Wireframe` | `wireframe` | `ReactGenerator` |
+//! | `Component` | `component` | — (future) |
+//! | `DesignToken` | `design-token` | — (future) |
 
 mod types;
 
diff --git a/crates/cclab-sdd/src/generate/spec_ir/types.rs b/crates/cclab-sdd/src/generate/spec_ir/types.rs
index 47ba5508..17ea770f 100644
--- a/crates/cclab-sdd/src/generate/spec_ir/types.rs
+++ b/crates/cclab-sdd/src/generate/spec_ir/types.rs
@@ -9,6 +9,210 @@ use crate::generate::diagrams::{
 };
 use crate::generate::schema::JsonSchema;
 
+// ---------------------------------------------------------------------------
+// New spec types (deploy, wireframe, component, design-token section types)
+// ---------------------------------------------------------------------------
+
+/// Kubernetes deployment specification (for the `deploy` section type).
+///
+/// Targets k8s `Deployment` + `Service` manifest generation.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DeploySpec {
+    /// Application name — used as `metadata.name` in k8s resources.
+    #[serde(default)]
+    pub name: String,
+    /// Container image reference (e.g. `"nginx:1.21"`).
+    #[serde(default)]
+    pub image: String,
+    /// Port the container listens on. Defaults to 8080.
+    #[serde(default = "default_deploy_port")]
+    pub port: u16,
+    /// Number of desired pod replicas. Defaults to 1.
+    #[serde(default = "default_replicas")]
+    pub replicas: u32,
+    /// Container environment variables.
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub env: Vec<EnvVar>,
+    /// Optional CPU/memory resource limits.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub resources: Option<ResourceLimits>,
+}
+
+impl Default for DeploySpec {
+    fn default() -> Self {
+        Self {
+            name: String::new(),
+            image: String::new(),
+            port: default_deploy_port(),
+            replicas: default_replicas(),
+            env: Vec::new(),
+            resources: None,
+        }
+    }
+}
+
+fn default_deploy_port() -> u16 { 8080 }
+fn default_replicas() -> u32 { 1 }
+
+/// A single environment variable entry.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct EnvVar {
+    /// Variable name.
+    pub name: String,
+    /// Literal value (mutually exclusive with `value_from`).
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub value: Option<String>,
+    /// Source reference, e.g. `"secretKeyRef:my-secret:key"`.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub value_from: Option<String>,
+}
+
+/// CPU / memory resource constraints.
+#[derive(Debug, Clone, Default, Serialize, Deserialize)]
+pub struct ResourceLimits {
+    /// CPU limit, e.g. `"500m"`.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub cpu: Option<String>,
+    /// Memory limit, e.g. `"256Mi"`.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub memory: Option<String>,
+}
+
+/// Wireframe specification for React component scaffold generation
+/// (for the `wireframe` section type).
+#[derive(Debug, Clone, Default, Serialize, Deserialize)]
+pub struct WireframeSpec {
+    /// Component name in PascalCase (e.g. `"UserCard"`).
+    #[serde(default)]
+    pub name: String,
+    /// High-level component type: `"page"`, `"layout"`, `"card"`, `"form"`, etc.
+    #[serde(default)]
+    pub component_type: String,
+    /// TypeScript props exposed by the component.
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub props: Vec<PropDef>,
+    /// Top-level layout nodes rendered by the component.
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub layout: Vec<WireframeNode>,
+}
+
+/// A TypeScript prop definition for the React component.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct PropDef {
+    /// Prop name in camelCase.
+    pub name: String,
+    /// TypeScript type string, e.g. `"string"`, `"number"`, `"User"`.
+    pub prop_type: String,
+    /// Whether the prop is required (no `?` in the interface).
+    #[serde(default)]
+    pub required: bool,
+    /// Default value expression as a string (used in destructuring).
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub default_value: Option<String>,
+    /// Optional JSDoc description.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub description: Option<String>,
+}
+
+/// A single layout node in a wireframe tree.
+#[derive(Debug, Clone, Default, Serialize, Deserialize)]
+pub struct WireframeNode {
+    /// Element kind: `"text"`, `"button"`, `"input"`, `"list"`, `"container"`, etc.
+    pub kind: String,
+    /// Display label or placeholder text.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub label: Option<String>,
+    /// Nested child nodes.
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub children: Vec<WireframeNode>,
+}
+
+/// Component Element Model (CEM) spec for TypeScript interface + component
+/// skeleton generation (for the `component` section type).
+#[derive(Debug, Clone, Default, Serialize, Deserialize)]
+pub struct ComponentSpec {
+    /// Kebab-case custom element tag name (e.g. `"my-button"`).
+    #[serde(default)]
+    pub tag_name: String,
+    /// One-line summary shown in generated JSDoc.
+    #[serde(default)]
+    pub summary: String,
+    /// Reflected HTML attributes / observed properties.
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub attributes: Vec<AttributeDef>,
+    /// Named slots.
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub slots: Vec<SlotDef>,
+    /// Custom events emitted by this component.
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub events: Vec<EventDef>,
+}
+
+/// A component attribute / property definition.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct AttributeDef {
+    /// Attribute name in kebab-case.
+    pub name: String,
+    /// TypeScript type string.
+    #[serde(rename = "type", default)]
+    pub attr_type: String,
+    /// Whether the attribute is required.
+    #[serde(default)]
+    pub required: bool,
+    /// Optional description for JSDoc.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub description: Option<String>,
+}
+
+/// A named slot definition.
+#[derive(Debug, Clone, Default, Serialize, Deserialize)]
+pub struct SlotDef {
+    /// Slot name (`""` for the default slot).
+    pub name: String,
+    /// Optional slot description.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub description: Option<String>,
+}
+
+/// A custom event emitted by the component.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct EventDef {
+    /// Event name in kebab-case.
+    pub name: String,
+    /// TypeScript type for `CustomEvent<T>` detail.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub detail_type: Option<String>,
+    /// Optional description.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub description: Option<String>,
+}
+
+/// Design Token Community Group (DTCG) spec for CSS custom property /
+/// Tailwind token generation (for the `design-token` section type).
+#[derive(Debug, Clone, Default, Serialize, Deserialize)]
+pub struct DesignTokenSpec {
+    /// Token collection name used as a CSS prefix (e.g. `"theme"`).
+    #[serde(default)]
+    pub name: String,
+    /// Flat list of token entries.
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub tokens: Vec<DesignTokenEntry>,
+}
+
+/// A single DTCG design token.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DesignTokenEntry {
+    /// Dot-separated path, e.g. `"color.primary.500"`.
+    pub path: String,
+    /// Resolved token value as a string, e.g. `"#3B82F6"`.
+    pub value: String,
+    /// DTCG `$type`: `"color"`, `"dimension"`, `"fontWeight"`, etc.
+    pub token_type: String,
+    /// Optional description for generated comments.
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub description: Option<String>,
+}
+
 /// Specification Intermediate Representation.
 ///
 /// Universal input type for code generators. Each variant wraps an existing
@@ -52,6 +256,30 @@ pub enum SpecIR {
         #[serde(flatten)]
         metadata: SpecMetadata,
     },
+    /// Kubernetes deployment specification (deploy section type)
+    Deploy {
+        spec: DeploySpec,
+        #[serde(flatten)]
+        metadata: SpecMetadata,
+    },
+    /// Wireframe specification for React component scaffold (wireframe section type)
+    Wireframe {
+        spec: WireframeSpec,
+        #[serde(flatten)]
+        metadata: SpecMetadata,
+    },
+    /// Component Element Model for TypeScript interface generation (component section type)
+    Component {
+        spec: ComponentSpec,
+        #[serde(flatten)]
+        metadata: SpecMetadata,
+    },
+    /// Design Token Community Group format for CSS/Tailwind generation (design-token section type)
+    DesignToken {
+        spec: DesignTokenSpec,
+        #[serde(flatten)]
+        metadata: SpecMetadata,
+    },
 }
 
 /// Common metadata carried by every SpecIR variant.
@@ -115,6 +343,10 @@ impl SpecIR {
             SpecIR::ErdPlus { .. } => "erd_plus",
             SpecIR::SequencePlus { .. } => "sequence_plus",
             SpecIR::RequirementPlus { .. } => "requirement_plus",
+            SpecIR::Deploy { .. } => "deploy",
+            SpecIR::Wireframe { .. } => "wireframe",
+            SpecIR::Component { .. } => "component",
+            SpecIR::DesignToken { .. } => "design_token",
         }
     }
 
@@ -126,7 +358,11 @@ impl SpecIR {
             | SpecIR::ClassPlus { metadata, .. }
             | SpecIR::ErdPlus { metadata, .. }
             | SpecIR::SequencePlus { metadata, .. }
-            | SpecIR::RequirementPlus { metadata, .. } => metadata,
+            | SpecIR::RequirementPlus { metadata, .. }
+            | SpecIR::Deploy { metadata, .. }
+            | SpecIR::Wireframe { metadata, .. }
+            | SpecIR::Component { metadata, .. }
+            | SpecIR::DesignToken { metadata, .. } => metadata,
         }
     }
 
@@ -138,7 +374,11 @@ impl SpecIR {
             | SpecIR::ClassPlus { metadata, .. }
             | SpecIR::ErdPlus { metadata, .. }
             | SpecIR::SequencePlus { metadata, .. }
-            | SpecIR::RequirementPlus { metadata, .. } => metadata,
+            | SpecIR::RequirementPlus { metadata, .. }
+            | SpecIR::Deploy { metadata, .. }
+            | SpecIR::Wireframe { metadata, .. }
+            | SpecIR::Component { metadata, .. }
+            | SpecIR::DesignToken { metadata, .. } => metadata,
         }
     }
 }
@@ -201,6 +441,42 @@ impl From<RequirementDiagramDef> for SpecIR {
     }
 }
 
+impl From<DeploySpec> for SpecIR {
+    fn from(spec: DeploySpec) -> Self {
+        SpecIR::Deploy {
+            spec,
+            metadata: SpecMetadata::default(),
+        }
+    }
+}
+
+impl From<WireframeSpec> for SpecIR {
+    fn from(spec: WireframeSpec) -> Self {
+        SpecIR::Wireframe {
+            spec,
+            metadata: SpecMetadata::default(),
+        }
+    }
+}
+
+impl From<ComponentSpec> for SpecIR {
+    fn from(spec: ComponentSpec) -> Self {
+        SpecIR::Component {
+            spec,
+            metadata: SpecMetadata::default(),
+        }
+    }
+}
+
+impl From<DesignTokenSpec> for SpecIR {
+    fn from(spec: DesignTokenSpec) -> Self {
+        SpecIR::DesignToken {
+            spec,
+            metadata: SpecMetadata::default(),
+        }
+    }
+}
+
 // ---------------------------------------------------------------------------
 // SpecBundle helpers
 // ---------------------------------------------------------------------------

```

## Review: change-spec-logic

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-codegen-and-fixes

**Summary**: All 5 requirements fully implemented: R1 (group-scoped spec path in spec_service.rs with priority routing group_id > spec_group > default), R2 (find_specs_to_merge iterates groups/*/specs/ with backward-compat fallback), R3 (create_change_spec.rs and create_change_impl.rs resolve group_id per-spec via resolve_group_id_for_spec and pass it through the entire chain to prompt/payload writers), R4 (backward compat via groups/ directory existence check), R5 (resolve_group_id_for_spec mechanism provides active group tracking without new STATE.yaml fields). Previous review concerns about R3/R5 not being implemented have been fully addressed in this revision. The project compiles cleanly (10 unrelated warnings).

### Issues

- **[low]** implementation.md does not document the change-spec-logic changes. The diff section covers SpecIR codegen work (DeployGenerator, ReactGenerator, spec_ir types) but omits the spec_service.rs group_id routing, workflow caller fixes (create_change_spec.rs, create_change_impl.rs), and find_specs_to_merge() updates. The code changes themselves are correct and present in the source files, so this is purely a documentation gap.

## Review: codegen-system-extend

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-codegen-and-fixes

**Summary**: Implementation matches spec requirements R6 and R7. R6 (SpecIRGenerator protocol): new trait in common.rs with can_generate/generate_from_ir/template_dir — clean parallel to the existing Generator trait. R7 (new section-type generators): DeployGenerator produces deployment.yaml + service.yaml from SpecIR::Deploy; ReactGenerator produces {Name}.tsx + {Name}.types.ts + index.ts from SpecIR::Wireframe. Both generators handle template fallback and overwrite policy correctly. Four new SpecIR enum variants added (Deploy, Wireframe, Component, DesignToken) with 12 new payload types, all with proper serde annotations (defaults, skip_serializing_if). Public API re-exports are complete. All 13 tests pass (6 deploy, 7 react). Project compiles cleanly (10 unrelated warnings). Main spec codegen-system.md correctly updated to v2 with R6/R7 requirements, updated class diagram, and updated data flow flowchart.

### Issues

- **[low]** The codegen-system-extend change-spec itself is entirely unfilled (all sections are TODO). The system-level extension (SpecIRGenerator trait protocol, Router model, public API surface changes) is only captured in the R6/R7 additions to the main spec diff and the implementation summary — not formally specified in the change-spec. The sibling specs (generator-deploy, generator-react) cover individual generators well, but the trait-level contract and routing architecture lack a dedicated spec.
- **[low]** The <semantic-data> JSON block in codegen-system.md is stale — it still references old node names ('Generators') and does not reflect the updated flowchart with Router, SchemaGenerators, SpecIRGenerators, SpecIRParse. This block should be updated to match the new data flow diagram for tooling that consumes semantic-data.
- **[low]** DeployGenerator hardcodes namespace to 'default' without making it configurable via DeploySpec. While the spec allows this (R2 says metadata.namespace = 'default'), a future iteration should consider adding an optional namespace field to DeploySpec for multi-namespace deployments.

## Review: generator-deploy

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-codegen-and-fixes

**Summary**: DeployGenerator implementation fully satisfies all 5 spec requirements. R1 (SpecIRGenerator trait): can_generate correctly matches only SpecIR::Deploy, generate_from_ir produces a Manifest, template_dir returns "deploy". R2 (Deployment manifest): deployment.yaml renders apps/v1 Deployment with correct metadata.name (with settings.name fallback), namespace="default", replicas, image, port, env block (literal value + valueFrom), and optional resources.limits. R3 (Service manifest): service.yaml renders v1 Service with type=ClusterIP, selector.app matching deployment name, correct port/targetPort. R4 (Template fallback): inline string generation used when Tera templates are absent, no error on missing templates. R5 (Overwrite policy): all three policies (Error, Skip, Overwrite) correctly enforced before each file write. All 6 tests from the test plan pass: test_can_generate_deploy, test_cannot_generate_non_deploy, test_generate_produces_two_files, test_deployment_yaml_content, test_deployment_yaml_with_resources, test_service_yaml_content. Schema types (DeploySpec, EnvVar, ResourceLimits) match the JSON Schema definition with correct serde annotations. Code is clean and well-structured.

## Review: generator-react

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-codegen-and-fixes

**Summary**: All three issues from the previous review have been resolved in this revision. (1) render_jsx_body no longer generates invalid C-style comments inside JSX opening tags — labels are now rendered as `data-label` HTML attributes (valid JSX), with text/button nodes using the label as element content. A regression test (test_component_tsx_structure) asserts no C-style comments appear. (2) Empty-props case now references the generated props interface (`_props: ComponentNameProps`) instead of the bare object type `{}`, keeping the component and types files consistent. (3) New test_component_tsx_structure validates R2 structural requirements (import type, named export, default export, componentType JSDoc). All 8 unit tests pass. The implementation fully satisfies all spec requirements: R1 (SpecIRGenerator trait), R2 (component file with import/export/destructuring/defaults/JSDoc/PascalCase), R3 (types file with required/optional props and JSDoc), R4 (barrel index re-exports), R5 (template fallback), R6 (overwrite policy). Public re-exports in mod.rs and lib.rs are correct.

## Review: spec-ir-schema-extend

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-codegen-and-fixes

**Summary**: Implementation is correct and well-structured. Four new SpecIR enum variants (Deploy, Wireframe, Component, DesignToken) added with 12 supporting payload types. All types have proper serde annotations (#[serde(default)], skip_serializing_if), doc comments, Default impls where appropriate, and From<T> conversions. The kind_name() and metadata()/metadata_mut() methods are correctly extended for all 4 new variants. Public re-exports in spec_ir/mod.rs and generate/lib.rs are complete. The mod.rs variant table documentation is accurate and helpful. All 5 existing spec_ir tests pass. Code quality is high — consistent structure, defensive defaults, proper Clone/Debug derives. Two medium issues: (1) the change-spec itself is entirely unfilled (TODO skeleton), meaning the 12 new types lack any formal spec definition, and (2) the main spec's kind registry JSON Schema is stale (missing 4 new kinds). One low issue: ComponentSpec and DesignTokenSpec lack direct test coverage.

### Issues

- **[medium]** The change-spec spec-ir-schema-extend is entirely unfilled (all sections are TODO). The 4 new SpecIR variants (Deploy, Wireframe, Component, DesignToken) and 12 payload types (DeploySpec, EnvVar, ResourceLimits, WireframeSpec, PropDef, WireframeNode, ComponentSpec, AttributeDef, SlotDef, EventDef, DesignTokenSpec, DesignTokenEntry) were implemented without any formal spec — no requirements, no JSON Schema definitions, no acceptance criteria in this change-spec. The types are well-defined in code but the spec should have captured: (1) field-level JSON Schema for each payload type, (2) requirements for serde defaults and skip-if-empty behavior, (3) acceptance criteria for round-trip serialization. This is a spec-process gap, not an implementation gap.
- **[medium]** Main spec (spec-ir-schema.md) R2 kind registry JSON Schema enum is stale — it lists only [Api, FlowchartPlus, SequencePlus, ClassPlus, ErdPlus, RequirementPlus] but the implementation now has 4 additional kinds: Deploy, Wireframe, Component, DesignToken. The spec's JSON Schema 'enum' block should be updated to include all 10 kinds to keep the spec-as-contract accurate.
- **[low]** No dedicated unit tests for the new payload types' serialization/deserialization round-trips. The types are exercised indirectly via DeployGenerator and ReactGenerator tests, but ComponentSpec and DesignTokenSpec (which have no generator yet) lack any test coverage — not even a basic serde round-trip test.

## Review: spec-validator-extend

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-codegen-and-fixes

**Summary**: Implementation is correct, comprehensive, and well-tested. New SpecIRValidator trait provides a clean pluggable registration mechanism with can_validate() routing and validate() dispatch. Four concrete validators cover all new section types: DeployValidator (name, image, port, replicas, env validation), WireframeValidator (name, component_type, props, layout node recursion), ComponentValidator (kebab-case tag_name with hyphen requirement, attributes, events), DesignTokenValidator (DTCG type validation against 13 standard types). Error vs warning classification is correct — required fields produce errors, optional/soft constraints produce warnings, cross-section refs are explicitly deferred to warnings only. 28 unit tests all pass. Code compiles cleanly. Public API surface is properly re-exported through validator/mod.rs and generate/lib.rs. Two medium process issues: (1) change-spec is entirely unfilled TODO skeleton, (2) implementation.md is missing the validator diff entirely.

### Issues

- **[medium]** The change-spec spec-validator-extend is entirely unfilled (all sections are TODO). The SpecIRValidator trait, validate_spec_ir() dispatch function, and 4 concrete validators (DeployValidator, WireframeValidator, ComponentValidator, DesignTokenValidator) were implemented without any formal spec requirements, acceptance criteria, or diagrams in this change-spec. The code is well-structured but the spec should have captured: (1) the trait interface definition (can_validate + validate), (2) validation rules per section type (required fields vs warnings), (3) the cross-section reference validation deferral policy, (4) the soft-warning-only constraint for cross-ref checks.
- **[medium]** implementation.md does not include the validator changes. The diff for spec_ir_validator.rs (new, 873 lines), validator/mod.rs (modified), and the validator re-exports in lib.rs are all missing from implementation.md. These changes exist on disk as unstaged/untracked files but were not captured in the implementation record. The implementation.md should be regenerated to include all files changed for this change.
- **[low]** The SpecIRValidator registry uses a static slice (static VALIDATORS: &[&dyn SpecIRValidator]) rather than the linkme distributed_slice pattern used elsewhere in the codebase for CLI module registration. This is acceptable for now since all validators are in the same crate, but if validators are ever needed from external crates, the registration mechanism would need to be refactored.
