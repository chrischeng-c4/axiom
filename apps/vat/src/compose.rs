// HANDWRITE-BEGIN gap="missing-generator:logic:compose-subset-parser" tracker="#1484" reason="R1-R3/R6: new parse()/expand()/materialize() -- a real YAML compose-subset parser plus a supported-vs-hard-reject key walk that produces the exact per-key error text, a build:-to-image() in-process resolution call, and a depends_on no-bridge-DNS warning. No existing generated module has this parse/validate/expand shape, so the whole file is hand-authored this WI (missing-generator:logic:compose-subset-parser, tracker #1484)."

//! Compose file parsing, expansion, and materialization to vat.toml.
//!
//! Reads a docker-compose.yml, validates the supported subset, expands build
//! entries, and writes a vat.toml with ServiceConfig entries plus a synthesized
//! runner that requires all services.

use crate::config::{PortSpec, RunnerConfig, ServiceConfig, ServiceRuntime, VatConfig, VolumeMount};
use crate::commands;
use anyhow::{bail, Result};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// Top-level compose file structure.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ComposeFile {
    #[serde(default)]
    pub services: BTreeMap<String, ComposeService>,
    #[serde(default)]
    pub volumes: BTreeMap<String, Value>,
    #[serde(default)]
    pub version: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Per-service compose structure.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct ComposeService {
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    build: Option<ComposeBuild>,
    #[serde(default)]
    ports: Vec<String>,
    #[serde(default)]
    environment: Option<ComposeEnv>,
    #[serde(default)]
    depends_on: Option<ComposeDependsOn>,
    #[serde(default)]
    volumes: Vec<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

/// Compose build field (short string or full object).
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum ComposeBuild {
    Short(String),
    Full {
        #[serde(default)]
        context: String,
        #[serde(default)]
        dockerfile: Option<String>,
        #[serde(default)]
        args: Option<ComposeEnv>,
    },
}

/// Compose environment field (list or map).
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum ComposeEnv {
    List(Vec<String>),
    Map(BTreeMap<String, Option<String>>),
}

/// Compose depends_on field (list or map).
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum ComposeDependsOn {
    List(Vec<String>),
    Map(BTreeMap<String, ComposeDependsOnEntry>),
}

/// Compose depends_on map entry.
#[derive(Debug, serde::Deserialize)]
struct ComposeDependsOnEntry {
    #[serde(default)]
    condition: Option<String>,
}

/// Parse a compose YAML file into a ComposeFile, validating the supported subset.
pub fn parse(path: &Path) -> Result<ComposeFile> {
    let content = std::fs::read_to_string(path)?;
    let file: ComposeFile = serde_yaml::from_str(&content)?;

    // Validate top-level keys.
    for key in file.extra.keys() {
        if !key.starts_with("x-") {
            bail!(
                "compose file `{}` uses unsupported key `{}` -- remove it or edit the generated vat.toml directly after `vat compose import`",
                path.display(),
                key
            );
        }
    }

    // Validate per-service keys.
    for (service_id, service) in &file.services {
        for key in service.extra.keys() {
            if !key.starts_with("x-") {
                let reason = match key.as_str() {
                    "deploy" => "deployment options not supported",
                    "secrets" => "secrets not supported",
                    "configs" => "configs not supported",
                    "extends" => "extends not supported",
                    "networks" => "custom networks not supported",
                    "profiles" => "profiles not supported",
                    "healthcheck" => "healthcheck not supported",
                    "command" => "command override not supported",
                    "entrypoint" => "entrypoint override not supported",
                    _ => "unsupported key",
                };
                bail!(
                    "compose file `{}` service `{}` uses unsupported key `{}` -- {} remove it or edit the generated vat.toml directly after `vat compose import`",
                    path.display(),
                    service_id,
                    key,
                    reason
                );
            }
        }

        // Validate environment: no bare keys.
        if let Some(ref env) = service.environment {
            match env {
                ComposeEnv::List(list) => {
                    for entry in list {
                        if !entry.contains('=') {
                            bail!(
                                "compose file `{}` service `{}` uses unsupported key `environment` -- bare key with no value not supported; remove it or edit the generated vat.toml directly after `vat compose import`",
                                path.display(),
                                service_id
                            );
                        }
                    }
                }
                ComposeEnv::Map(map) => {
                    for (_, v) in map {
                        if v.is_none() {
                            bail!(
                                "compose file `{}` service `{}` uses unsupported key `environment` -- map entry with null value not supported; remove it or edit the generated vat.toml directly after `vat compose import`",
                                path.display(),
                                service_id
                            );
                        }
                    }
                }
            }
        }

        // Validate depends_on: no service_healthy condition.
        if let Some(ref deps) = service.depends_on {
            match deps {
                ComposeDependsOn::Map(map) => {
                    for (_, entry) in map {
                        if entry.condition.as_deref() == Some("service_healthy") {
                            bail!(
                                "compose file `{}` service `{}` uses unsupported key `depends_on` -- service_healthy condition not supported; remove it or edit the generated vat.toml directly after `vat compose import`",
                                path.display(),
                                service_id
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        // Validate volumes: no bind mounts.
        for vol in &service.volumes {
            if let Some(colon_idx) = vol.find(':') {
                let host_part = &vol[..colon_idx];
                if host_part.contains('/') {
                    bail!(
                        "compose file `{}` service `{}` uses unsupported key `volumes` -- bind-mount form not supported; remove it or edit the generated vat.toml directly after `vat compose import`",
                        path.display(),
                        service_id
                    );
                }
            }
        }
    }

    Ok(file)
}

/// Expand a ComposeFile into ServiceConfig entries, resolving builds and materializing volumes/env/ports.
pub fn expand(file: &ComposeFile, project: &str, runtime: ServiceRuntime) -> Result<Vec<ServiceConfig>> {
    let mut services = Vec::new();

    for (service_id, service) in &file.services {
        let mut config = ServiceConfig {
            id: service_id.clone(),
            requires: Vec::new(),
            cmd: Vec::new(),
            preset: None,
            image: None,
            container_port: None,
            image_env: BTreeMap::new(),
            runtime,
            cluster: None,
            external: None,
            k8s_version: None,
            nodes: None,
            spec: None,
            version: None,
            port: crate::config::PortSpec::Auto(String::new()),
            seed: Vec::new(),
            export: BTreeMap::new(),
            ready_http: None,
            ready_cmd: Vec::new(),
            timeout_s: 300,
            volumes: Vec::new(),
        };

        // Resolve image or build.
        if let Some(ref image_str) = service.image {
            config.image = Some(image_str.clone());
        } else if let Some(ref build) = service.build {
            let (context_str, dockerfile_str) = match build {
                ComposeBuild::Short(s) => (s.clone(), "Dockerfile".to_string()),
                ComposeBuild::Full {
                    context,
                    dockerfile,
                    ..
                } => (context.clone(), dockerfile.clone().unwrap_or_else(|| "Dockerfile".to_string())),
            };
            let context_path = Path::new(&context_str);
            let dockerfile_path = context_path.join(&dockerfile_str);
            let tag = format!("{}:latest", service_id);
            let report = commands::build::build_image(
                context_path,
                &dockerfile_path,
                &tag,
                &[],
            )?;
            config.image = Some(report.tag);
        }

        // Parse ports.
        for port_str in &service.ports {
            if let Some(colon_idx) = port_str.find(':') {
                let host_port: u16 = port_str[..colon_idx].parse()?;
                let container_port: u16 = port_str[colon_idx + 1..].parse()?;
                config.container_port = Some(container_port);
                config.port = PortSpec::Fixed(host_port);
            } else {
                let container_port: u16 = port_str.parse()?;
                config.container_port = Some(container_port);
                config.port = PortSpec::Auto("auto".to_string());
            }
        }

        // Flatten environment.
        if let Some(ref env) = service.environment {
            match env {
                ComposeEnv::List(list) => {
                    for entry in list {
                        if let Some(eq_idx) = entry.find('=') {
                            let key = entry[..eq_idx].to_string();
                            let value = entry[eq_idx + 1..].to_string();
                            config.image_env.insert(key, value);
                        }
                    }
                }
                ComposeEnv::Map(map) => {
                    for (key, value) in map {
                        if let Some(v) = value {
                            config.image_env.insert(key.clone(), v.clone());
                        }
                    }
                }
            }
        }

        // Map depends_on to requires.
        if let Some(ref deps) = service.depends_on {
            match deps {
                ComposeDependsOn::List(list) => {
                    config.requires.extend(list.clone());
                }
                ComposeDependsOn::Map(map) => {
                    for key in map.keys() {
                        config.requires.push(key.clone());
                    }
                }
            }
        }

        // Warn about depends_on (no bridge DNS).
        if !config.requires.is_empty() {
            eprintln!(
                "vat compose: service `{}` has depends_on; note that vat does not simulate container-to-container bridge-network DNS — use VAT_SERVICE_ID_HOST/PORT instead",
                service_id
            );
        }

        // Map named volumes.
        for vol in &service.volumes {
            if let Some(colon_idx) = vol.find(':') {
                let name = vol[..colon_idx].to_string();
                let path = vol[colon_idx + 1..].to_string();
                config.volumes.push(VolumeMount { name, path });
            }
        }

        services.push(config);
    }

    Ok(services)
}

/// Materialize a list of ServiceConfigs into a vat.toml file.
pub fn materialize(services: &[ServiceConfig], out: &Path) -> Result<()> {
    let mut runner_requires = Vec::new();
    for svc in services {
        runner_requires.push(svc.id.clone());
    }

    let vat_config = VatConfig {
        version: 1,
        name: None,
        default_runner: None,
        workspace: crate::config::WorkspaceConfig::default(),
        env: BTreeMap::new(),
        setup: Vec::new(),
        services: services.to_vec(),
        runners: vec![RunnerConfig {
            id: "project.up".to_string(),
            // "infinity" is GNU-coreutils-only; BSD/macOS `sleep` rejects it
            // ("usage: sleep number[unit]"). A huge finite second count is
            // portable across both and still outlives any test run.
            cmd: vec!["sleep".to_string(), "2147483647".to_string()],
            requires: runner_requires,
            timeout_s: None,
            artifacts: Vec::new(),
        }],
        scenarios: Vec::new(),
        network: None,
        path: out.to_path_buf(),
        root: out.parent().unwrap_or_else(|| Path::new("/")).to_path_buf(),
        digest: String::new(),
    };

    let toml = toml::to_string_pretty(&vat_config)?;
    std::fs::write(out, toml)?;
    println!("Wrote {}", out.display());
    Ok(())
}
// HANDWRITE-END
