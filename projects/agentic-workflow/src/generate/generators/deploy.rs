// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/deploy_preamble.md#source
// CODEGEN-BEGIN
//! Kubernetes Deployment + Service manifest generator
//!
//! Generates Kubernetes manifests from a [`DeploySpec`] (deploy section type):
//!
//! | Output file        | Description                                    |
//! |--------------------|------------------------------------------------|
//! | `deployment.yaml`  | `apps/v1 Deployment` resource                  |
//! | `service.yaml`     | `v1 Service` (ClusterIP) resource              |
//!
//! The generator implements [`SpecIRGenerator`] and only accepts
//! [`SpecIR::Deploy`] variants.

use super::common::{
    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::spec_ir::{DeploySpec, EnvVar, SpecIR};
use serde::Serialize;

// ---------------------------------------------------------------------------
// DeployGenerator
// ---------------------------------------------------------------------------
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/deploy.md#schema
// CODEGEN-BEGIN
/// Deploy generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/deploy.md#schema
pub struct DeployGenerator;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/deploy_runtime.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/deploy_runtime.md#source
impl DeployGenerator {
    pub fn new() -> Self {
        Self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/deploy_runtime.md#source
impl Default for DeployGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Template context
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct DeployContext {
    name: String,
    image: String,
    port: u16,
    replicas: u32,
    namespace: String,
    env: Vec<EnvVarContext>,
    cpu_limit: Option<String>,
    memory_limit: Option<String>,
}

#[derive(Debug, Serialize)]
struct EnvVarContext {
    name: String,
    /// `true` if a literal `value:` should be emitted
    is_literal: bool,
    value: Option<String>,
    /// Secret/ConfigMap reference string (rendered verbatim in the manifest)
    value_from: Option<String>,
}

// ---------------------------------------------------------------------------
// SpecIRGenerator impl
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/deploy_runtime.md#source
impl SpecIRGenerator for DeployGenerator {
    fn can_generate(&self, spec: &SpecIR) -> bool {
        matches!(spec, SpecIR::Deploy { .. })
    }

    fn template_dir(&self) -> &'static str {
        "deploy"
    }

    fn generate_from_ir(
        &self,
        spec: &SpecIR,
        settings: &GeneratorSettings,
        engine: &TemplateEngine,
    ) -> Result<Manifest, GeneratorError> {
        let deploy_spec = match spec {
            SpecIR::Deploy { spec, .. } => spec,
            _ => {
                return Err(GeneratorError::SchemaError(
                    "DeployGenerator: expected SpecIR::Deploy variant".into(),
                ))
            }
        };

        let mut manifest = Manifest::new();
        let ctx = build_context(deploy_spec, settings);

        let files: &[(&str, &str, fn(&DeployContext) -> String)] = &[
            (
                "deployment.yaml.j2",
                "deployment.yaml",
                generate_deployment_yaml,
            ),
            ("service.yaml.j2", "service.yaml", generate_service_yaml),
        ];

        for (template, output, inline_gen) in files {
            let output_path = settings.output_dir.join(output);

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
                engine.render(&template_name, &ctx).map_err(|e| {
                    GeneratorError::TemplateRenderError {
                        template: template_name.clone(),
                        message: e.to_string(),
                    }
                })?
            } else {
                inline_gen(&ctx)
            };

            manifest.add(GeneratedFile::written(output_path, &content));
        }

        Ok(manifest)
    }
}

// ---------------------------------------------------------------------------
// Context builder
// ---------------------------------------------------------------------------

fn build_context(spec: &DeploySpec, settings: &GeneratorSettings) -> DeployContext {
    let name = if spec.name.is_empty() {
        settings.name.clone()
    } else {
        spec.name.clone()
    };

    let env = spec.env.iter().map(|e| env_var_context(e)).collect();

    let (cpu_limit, memory_limit) = spec
        .resources
        .as_ref()
        .map(|r| (r.cpu.clone(), r.memory.clone()))
        .unwrap_or((None, None));

    DeployContext {
        name,
        image: spec.image.clone(),
        port: spec.port,
        replicas: spec.replicas,
        namespace: "default".to_string(),
        env,
        cpu_limit,
        memory_limit,
    }
}

fn env_var_context(e: &EnvVar) -> EnvVarContext {
    EnvVarContext {
        name: e.name.clone(),
        is_literal: e.value.is_some(),
        value: e.value.clone(),
        value_from: e.value_from.clone(),
    }
}

// ---------------------------------------------------------------------------
// Inline generators (used when Tera templates are absent)
// ---------------------------------------------------------------------------

fn generate_deployment_yaml(ctx: &DeployContext) -> String {
    let env_block = if ctx.env.is_empty() {
        String::new()
    } else {
        let mut block = "          env:\n".to_string();
        for e in &ctx.env {
            if e.is_literal {
                block.push_str(&format!(
                    "            - name: {}\n              value: \"{}\"\n",
                    e.name,
                    e.value.as_deref().unwrap_or("")
                ));
            } else if let Some(vf) = &e.value_from {
                block.push_str(&format!(
                    "            - name: {}\n              valueFrom: {}\n",
                    e.name, vf
                ));
            } else {
                block.push_str(&format!("            - name: {}\n", e.name));
            }
        }
        block
    };

    let resources_block = build_resources_block(ctx);

    format!(
        r#"# Generated by sdd
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {name}
  namespace: {namespace}
  labels:
    app: {name}
spec:
  replicas: {replicas}
  selector:
    matchLabels:
      app: {name}
  template:
    metadata:
      labels:
        app: {name}
    spec:
      containers:
        - name: {name}
          image: {image}
          ports:
            - containerPort: {port}
{env_block}{resources_block}
"#,
        name = ctx.name,
        namespace = ctx.namespace,
        replicas = ctx.replicas,
        image = ctx.image,
        port = ctx.port,
        env_block = env_block,
        resources_block = resources_block,
    )
}

fn build_resources_block(ctx: &DeployContext) -> String {
    if ctx.cpu_limit.is_none() && ctx.memory_limit.is_none() {
        return String::new();
    }
    let mut block = "          resources:\n            limits:\n".to_string();
    if let Some(cpu) = &ctx.cpu_limit {
        block.push_str(&format!("              cpu: \"{}\"\n", cpu));
    }
    if let Some(mem) = &ctx.memory_limit {
        block.push_str(&format!("              memory: \"{}\"\n", mem));
    }
    block
}

fn generate_service_yaml(ctx: &DeployContext) -> String {
    format!(
        r#"# Generated by sdd
apiVersion: v1
kind: Service
metadata:
  name: {name}
  namespace: {namespace}
  labels:
    app: {name}
spec:
  selector:
    app: {name}
  ports:
    - protocol: TCP
      port: {port}
      targetPort: {port}
  type: ClusterIP
"#,
        name = ctx.name,
        namespace = ctx.namespace,
        port = ctx.port,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::spec_ir::{DeploySpec, EnvVar, ResourceLimits, SpecIR, SpecMetadata};

    fn simple_spec() -> SpecIR {
        SpecIR::Deploy {
            spec: DeploySpec {
                name: "my-app".into(),
                image: "my-app:1.0".into(),
                port: 3000,
                replicas: 2,
                env: vec![EnvVar {
                    name: "ENV".into(),
                    value: Some("production".into()),
                    value_from: None,
                }],
                resources: None,
            },
            metadata: SpecMetadata::default(),
        }
    }

    #[test]
    fn test_can_generate_deploy() {
        let gen = DeployGenerator::new();
        assert!(gen.can_generate(&simple_spec()));
    }

    #[test]
    fn test_cannot_generate_non_deploy() {
        use crate::generate::schema::JsonSchema;
        let gen = DeployGenerator::new();
        let api_spec = SpecIR::Api {
            schema: JsonSchema::default(),
            metadata: SpecMetadata::default(),
        };
        assert!(!gen.can_generate(&api_spec));
    }

    #[test]
    fn test_generate_produces_two_files() {
        let spec = simple_spec();
        let settings = GeneratorSettings {
            output_dir: std::path::PathBuf::from("/tmp/test_deploy_gen"),
            ..Default::default()
        };
        let engine = crate::generate::engine::TemplateEngine::empty();
        let gen = DeployGenerator::new();
        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();

        assert_eq!(manifest.files.len(), 2);
        let names: Vec<String> = manifest
            .files
            .keys()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"deployment.yaml".to_string()));
        assert!(names.contains(&"service.yaml".to_string()));
    }

    #[test]
    fn test_deployment_yaml_content() {
        let spec = simple_spec();
        let settings = GeneratorSettings {
            output_dir: std::path::PathBuf::from("/tmp/test_deploy_gen_content"),
            ..Default::default()
        };
        let engine = crate::generate::engine::TemplateEngine::empty();
        let gen = DeployGenerator::new();
        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();

        let deployment = manifest
            .files
            .values()
            .find(|f| f.path.file_name().unwrap() == "deployment.yaml")
            .expect("deployment.yaml not found");

        // Content should be in manifest (content hash set)
        assert!(deployment.content_hash.is_some());
    }

    #[test]
    fn test_deployment_yaml_with_resources() {
        let spec = SpecIR::Deploy {
            spec: DeploySpec {
                name: "api".into(),
                image: "api:latest".into(),
                port: 8080,
                replicas: 3,
                env: vec![],
                resources: Some(ResourceLimits {
                    cpu: Some("500m".into()),
                    memory: Some("256Mi".into()),
                }),
            },
            metadata: SpecMetadata::default(),
        };
        let ctx = build_context(
            match &spec {
                SpecIR::Deploy { spec, .. } => spec,
                _ => unreachable!(),
            },
            &GeneratorSettings::default(),
        );

        let yaml = generate_deployment_yaml(&ctx);
        assert!(yaml.contains("resources:"));
        assert!(yaml.contains("cpu: \"500m\""));
        assert!(yaml.contains("memory: \"256Mi\""));
    }

    #[test]
    fn test_service_yaml_content() {
        let ctx = DeployContext {
            name: "my-svc".into(),
            image: "my-svc:1.0".into(),
            port: 9090,
            replicas: 1,
            namespace: "default".into(),
            env: vec![],
            cpu_limit: None,
            memory_limit: None,
        };
        let yaml = generate_service_yaml(&ctx);
        assert!(yaml.contains("kind: Service"));
        assert!(yaml.contains("name: my-svc"));
        assert!(yaml.contains("port: 9090"));
        assert!(yaml.contains("type: ClusterIP"));
    }
}
// CODEGEN-END
