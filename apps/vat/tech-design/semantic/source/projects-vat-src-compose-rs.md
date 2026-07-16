---
id: vat-source-projects-vat-src-compose-rs
summary: >
  rust-source-unit mirror for apps/vat/src/compose.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: compose-runtime-local-build-artifacts
    coverage: full
    rationale: "#1529 source mirror for canonical compose-relative build resolution, runtime-local image stores, and atomic vat.toml materialization."
---

# Source mirror: apps/vat/src/compose.rs

## Overview
<!-- type: overview lang: markdown -->

Hand-written compose importer source, mirrored in full. Its #1529 contract binds a
build-bearing import to the compose file's canonical location and to one selected
local image store before it can replace a generated vat.toml, gives each raw
project/service pair an OCI-safe readable tag plus a BLAKE3 identity suffix, and
exposes rollback support for a failed registry publication. Its Docker-shaped
parser is separate and exposes exactly `strict-single-image-v1`,
`strict-single-build-v1`, and `host-facing-independent-v1`; the last requires
the exact `x-vat-compose-profile: host-facing-independent-v1` marker, two
through four literal-image services, unique loopback-published ports, and no
bridge/service-name DNS, topology, build, interpolation, or env-file escape.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ComposeFile` | apps/vat/src/compose.rs | struct | pub | 23 | |
| `source_path` | apps/vat/src/compose.rs | function | pub | 41 | source_path(&self) -> &Path |
| `service_uses_build` | apps/vat/src/compose.rs | function | pub(crate) | 50 | service_uses_build(&self, service_id: &str) -> bool |
| `DockerComposeProfile` | apps/vat/src/compose.rs | enum | pub(crate) | 268 | StrictSingleImageV1 \| StrictSingleBuildV1 \| HostFacingIndependentV1 |
| `ParsedDockerComposeProfile` | apps/vat/src/compose.rs | struct | pub(crate) | 297 | { file: ComposeFile, profile: DockerComposeProfile } |
| `parse` | apps/vat/src/compose.rs | function | pub | 115 | parse(path: &Path) -> Result<ComposeFile> |
| `parse_docker_compose_compat_profile` | apps/vat/src/compose.rs | function | pub(crate) | 306 | parse_docker_compose_compat_profile(path: &Path) -> Result<ParsedDockerComposeProfile> |
| `parse_docker_compose_build_compat_profile` | apps/vat/src/compose.rs | function | pub(crate) | 332 | parse_docker_compose_build_compat_profile(path: &Path) -> Result<ParsedDockerComposeProfile> |
| `expand` | apps/vat/src/compose.rs | function | pub | 405 | expand(file: &ComposeFile, project: &str, runtime: ServiceRuntime) -> Result<Vec<ServiceConfig>> |
| `materialize` | apps/vat/src/compose.rs | function | pub | 577 | materialize(services: &[ServiceConfig], out: &Path) -> Result<()> |
| `restore_materialized_config` | apps/vat/src/compose.rs | function | pub(crate) | 656 | restore_materialized_config(path: &Path, previous: Option<&[u8]>) -> Result<()> |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// HANDWRITE-BEGIN gap="missing-generator:logic:compose-subset-parser" tracker="#1484" reason="R1-R3/R6 plus #1529: parse()/expand()/materialize() own the YAML subset walk, canonical compose-source-relative build context/Dockerfile resolution, Docker-versus-Apple-Container image-store selection, project-scoped tags/build args, preflight-before-materialize, atomic vat.toml replacement, and depends_on no-bridge-DNS warning. No existing generated module has this parse/validate/expand shape, so the whole file remains hand-authored (missing-generator:logic:compose-subset-parser; trackers #1484 and #1529)."

//! Compose file parsing, expansion, and materialization to vat.toml.
//!
//! Reads a docker-compose.yml, validates the supported subset, expands build
//! entries, and writes a vat.toml with ServiceConfig entries plus a synthesized
//! runner that requires all services.

use crate::commands;
use crate::config::{
    PortSpec, RunnerConfig, ServiceConfig, ServiceRuntime, VatConfig, VolumeMount,
};
use anyhow::{bail, Context, Result};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Top-level compose file structure.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ComposeFile {
    /// Canonical source location retained outside the YAML document so every
    /// build path is resolved from the compose file, never the caller's cwd.
    #[serde(skip)]
    source_path: PathBuf,
    #[serde(default)]
    services: BTreeMap<String, ComposeService>,
    #[serde(default, rename = "volumes")]
    _volumes: BTreeMap<String, Value>,
    #[serde(default, rename = "version")]
    _version: String,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

impl ComposeFile {
    /// Canonical source path used for diagnostics and import-relative build
    /// resolution.
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// Whether a captured source service was build-backed. Import uses this
    /// only to label the concrete runtime-local image that `expand` produced;
    /// literal `image:` references must never be presented as VAT-owned build
    /// cleanup candidates.
    pub(crate) fn service_uses_build(&self, service_id: &str) -> bool {
        self.services
            .get(service_id)
            .map_or(false, |service| service.image.is_none() && service.build.is_some())
    }
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
    let source_path = fs::canonicalize(path)
        .with_context(|| format!("resolve compose file {}", path.display()))?;
    let content = fs::read_to_string(&source_path)
        .with_context(|| format!("read compose file {}", source_path.display()))?;
    let mut file: ComposeFile = serde_yaml::from_str(&content)
        .with_context(|| format!("parse compose file {}", source_path.display()))?;
    file.source_path = source_path;

    // Validate top-level keys.
    for key in file.extra.keys() {
        if !key.starts_with("x-") {
            bail!(
                "compose file `{}` uses unsupported key `{}` -- remove it or edit the generated vat.toml directly after `vat compose import`",
                file.source_path.display(),
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
                    file.source_path.display(),
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
                                file.source_path.display(),
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
                                file.source_path.display(),
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
                                file.source_path.display(),
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
                        file.source_path.display(),
                        service_id
                    );
                }
            }
        }
    }

    Ok(file)
}

/// Parse and capture the deliberately tiny literal-image profile exposed
/// through VAT's opt-in `docker` multicall shim. The returned document is the
/// one that must be materialized: reparsing the path after validation would
/// let a symlink or file replacement widen the strict profile in between.
pub(crate) fn parse_docker_compose_compat_profile(path: &Path) -> Result<ComposeFile> {
    parse_docker_compose_compat_profile_with_mode(path, DockerComposeCompatMode::Image)
}

/// Parse and capture the strict source-build profile. It is intentionally a
/// separate mode rather than a relaxation of the literal-image profile:
/// `up -d --build` has a concrete typed build contract, while silently
/// accepting `build:` on normal `up -d` would hide a meaningful Docker Compose
/// semantic difference from an agent.
pub(crate) fn parse_docker_compose_build_compat_profile(path: &Path) -> Result<ComposeFile> {
    parse_docker_compose_compat_profile_with_mode(path, DockerComposeCompatMode::Build)
}

#[derive(Clone, Copy)]
enum DockerComposeCompatMode {
    Image,
    Build,
}

fn parse_docker_compose_compat_profile_with_mode(
    path: &Path,
    mode: DockerComposeCompatMode,
) -> Result<ComposeFile> {
    let file = parse(path)?;
    validate_docker_compose_compat_profile_with_mode(&file, mode)?;
    Ok(file)
}

fn validate_docker_compose_compat_profile_with_mode(
    file: &ComposeFile,
    mode: DockerComposeCompatMode,
) -> Result<()> {
    if !file.extra.is_empty() {
        bail!(
            "VAT's docker compose compatibility profile rejects top-level extension keys; use only a literal single-service file"
        );
    }
    if !file._volumes.is_empty() {
        bail!("VAT's docker compose compatibility profile does not support top-level volumes");
    }
    if file.services.len() != 1 {
        bail!(
            "VAT's docker compose compatibility profile requires exactly one service; multi-service bridge-DNS semantics are not implemented"
        );
    }

    let (service_id, service) = file
        .services
        .iter()
        .next()
        .context("single-service compose profile has no service")?;
    if !service.extra.is_empty() {
        bail!(
            "VAT's docker compose compatibility profile service `{service_id}` uses unsupported keys; only the selected literal image/build source, one explicit port, and literal environment are allowed"
        );
    }
    match mode {
        DockerComposeCompatMode::Image => {
            if service.build.is_some() {
                bail!(
                    "VAT's docker compose compatibility profile service `{service_id}` may not use build; use `docker compose -f FILE -p PROJECT up -d --build` with a literal build-only service instead"
                );
            }
            let image = service.image.as_deref().context(format!(
                "VAT's docker compose compatibility profile service `{service_id}` requires a literal image"
            ))?;
            ensure_docker_compose_literal(image, service_id, "image")?;
        }
        DockerComposeCompatMode::Build => {
            if service.image.is_some() {
                bail!(
                    "VAT's docker compose build profile service `{service_id}` may not combine image and build; use exactly one literal short `build: <context>` field"
                );
            }
            let build = service.build.as_ref().context(format!(
                "VAT's docker compose build profile service `{service_id}` requires one literal short `build: <context>` field"
            ))?;
            let ComposeBuild::Short(context) = build else {
                bail!(
                    "VAT's docker compose build profile service `{service_id}` supports only literal short `build: <context>`; build mappings, build args, and custom Dockerfiles are not part of this profile"
                );
            };
            ensure_docker_compose_literal(context, service_id, "build")?;
        }
    }
    if service.depends_on.is_some() {
        bail!(
            "VAT's docker compose compatibility profile service `{service_id}` may not use depends_on; multi-service topology is unsupported"
        );
    }
    if !service.volumes.is_empty() {
        bail!(
            "VAT's docker compose compatibility profile service `{service_id}` may not use volumes"
        );
    }
    if service.ports.len() != 1 {
        bail!(
            "VAT's docker compose compatibility profile service `{service_id}` requires exactly one explicit host:container port"
        );
    }
    validate_docker_compose_port(service_id, &service.ports[0])?;
    if let Some(environment) = &service.environment {
        validate_docker_compose_environment(service_id, environment)?;
    }
    Ok(())
}

fn ensure_docker_compose_literal(value: &str, service_id: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() || value.contains('$') {
        bail!(
            "VAT's docker compose compatibility profile service `{service_id}` field `{field}` must be a non-empty literal; Compose interpolation and dollar escaping are unsupported"
        );
    }
    Ok(())
}

fn validate_docker_compose_port(service_id: &str, port: &str) -> Result<()> {
    ensure_docker_compose_literal(port, service_id, "ports")?;
    if port.matches(':').count() != 1 || port.contains('/') {
        bail!(
            "VAT's docker compose compatibility profile service `{service_id}` port `{port}` must use exactly host-port:container-port with no IP, protocol, or range"
        );
    }
    let (host, container) = port
        .split_once(':')
        .context("validated compose port separator")?;
    let host = host.parse::<u16>().with_context(|| {
        format!(
            "VAT's docker compose compatibility profile service `{service_id}` host port `{host}` is not a nonzero integer"
        )
    })?;
    let container = container.parse::<u16>().with_context(|| {
        format!(
            "VAT's docker compose compatibility profile service `{service_id}` container port `{container}` is not a nonzero integer"
        )
    })?;
    if host == 0 || container == 0 {
        bail!(
            "VAT's docker compose compatibility profile service `{service_id}` requires nonzero explicit host and container ports"
        );
    }
    Ok(())
}

fn validate_docker_compose_environment(service_id: &str, environment: &ComposeEnv) -> Result<()> {
    match environment {
        ComposeEnv::List(entries) => {
            for entry in entries {
                ensure_docker_compose_literal(entry, service_id, "environment")?;
            }
        }
        ComposeEnv::Map(entries) => {
            for (key, value) in entries {
                ensure_docker_compose_literal(key, service_id, "environment key")?;
                ensure_docker_compose_literal(
                    value
                        .as_deref()
                        .context("validated non-null compose environment value")?,
                    service_id,
                    "environment value",
                )?;
            }
        }
    }
    Ok(())
}

/// Expand a ComposeFile into ServiceConfig entries, resolving build paths from
/// the compose source location (never the caller cwd) and materializing
/// volumes/env/ports.
pub fn expand(
    file: &ComposeFile,
    project: &str,
    runtime: ServiceRuntime,
) -> Result<Vec<ServiceConfig>> {
    // Image-only imports must remain usable without either local builder. For
    // a build-bearing file, prove the selected image store before any build or
    // vat.toml materialization can happen.
    let builder = if file
        .services
        .values()
        .any(|service| service.image.is_none() && service.build.is_some())
    {
        Some(
            commands::build::resolve_image_builder(runtime).with_context(|| {
                format!(
                    "compose file `{}` cannot preflight its build runtime; make it available, then retry `{}`",
                    file.source_path.display(),
                    compose_import_retry_command(&file.source_path, project, runtime)
                )
            })?,
        )
    } else {
        None
    };
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
            port: PortSpec::Auto(String::new()),
            seed: Vec::new(),
            export: BTreeMap::new(),
            ready_http: None,
            ready_cmd: Vec::new(),
            timeout_s: 300,
            volumes: Vec::new(),
        };

        // An explicit image remains unchanged. A build-only service gets a
        // project-scoped tag and is built into exactly the selected runtime's
        // image store.
        if let Some(image) = &service.image {
            config.image = Some(image.clone());
        } else if let Some(build) = &service.build {
            let (context, dockerfile, build_args) = compose_build_paths(file, service_id, build)?;
            let tag = compose_build_tag(project, service_id);
            let builder =
                builder.context("build-bearing compose file skipped runtime preflight")?;
            let report = commands::build::build_image_with_builder(
                builder,
                &context,
                &dockerfile,
                &tag,
                &build_args,
            )
            .with_context(|| {
                format!(
                    "compose file `{}` service `{service_id}` failed to build; retry `{}`",
                    file.source_path.display(),
                    compose_import_retry_command(&file.source_path, project, runtime)
                )
            })?;
            config.image = Some(report.tag);
            // Persist the concrete store owner for build-backed services. In
            // particular, `--runtime auto` resolves to Docker once at import
            // time, so future run dispatch cannot drift into a different
            // image store. Image-only services keep the caller's original
            // runtime unchanged.
            config.runtime = match builder {
                commands::build::ImageBuilder::Docker => ServiceRuntime::Docker,
                commands::build::ImageBuilder::MicroVm => ServiceRuntime::MicroVm,
            };
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
        if let Some(env) = &service.environment {
            match env {
                ComposeEnv::List(list) => {
                    for entry in list {
                        if let Some((key, value)) = entry.split_once('=') {
                            config.image_env.insert(key.to_string(), value.to_string());
                        }
                    }
                }
                ComposeEnv::Map(map) => {
                    for (key, value) in map {
                        if let Some(value) = value {
                            config.image_env.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }

        // Map depends_on to requires.
        if let Some(deps) = &service.depends_on {
            match deps {
                ComposeDependsOn::List(list) => config.requires.extend(list.clone()),
                ComposeDependsOn::Map(map) => config.requires.extend(map.keys().cloned()),
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
        for volume in &service.volumes {
            if let Some(colon_idx) = volume.find(':') {
                config.volumes.push(VolumeMount {
                    name: volume[..colon_idx].to_string(),
                    path: volume[colon_idx + 1..].to_string(),
                });
            }
        }

        services.push(config);
    }

    Ok(services)
}

fn compose_build_paths(
    file: &ComposeFile,
    service_id: &str,
    build: &ComposeBuild,
) -> Result<(PathBuf, PathBuf, Vec<(String, String)>)> {
    let source_dir = file
        .source_path
        .parent()
        .context("canonical compose source has no parent directory")?;
    let (context_raw, dockerfile_raw, args) = match build {
        ComposeBuild::Short(context) => (context.as_str(), None, None),
        ComposeBuild::Full {
            context,
            dockerfile,
            args,
        } => {
            let context = if context.is_empty() { "." } else { context };
            (context, dockerfile.as_deref(), args.as_ref())
        }
    };
    let context = resolve_compose_build_path(
        source_dir,
        context_raw,
        &file.source_path,
        service_id,
        "build.context",
    )?;
    if !context.is_dir() {
        bail!(
            "compose file `{}` service `{service_id}` resolves build.context `{context_raw}` to `{}`, which is not a directory",
            file.source_path.display(),
            context.display()
        );
    }
    let dockerfile = match dockerfile_raw {
        // Explicit dockerfile values are source-file-relative under this
        // bounded import contract, matching build.context diagnostics.
        Some(path) => resolve_compose_build_path(
            source_dir,
            path,
            &file.source_path,
            service_id,
            "build.dockerfile",
        )?,
        // Docker Compose's default remains the Dockerfile in the resolved
        // build context.
        None => fs::canonicalize(context.join("Dockerfile")).with_context(|| {
            format!(
                "compose file `{}` service `{service_id}` cannot resolve default build.dockerfile `{}`",
                file.source_path.display(),
                context.join("Dockerfile").display()
            )
        })?,
    };
    if !dockerfile.is_file() {
        bail!(
            "compose file `{}` service `{service_id}` resolves build.dockerfile to `{}`, which is not a file",
            file.source_path.display(),
            dockerfile.display()
        );
    }
    Ok((
        context,
        dockerfile,
        compose_build_args(args, &file.source_path, service_id)?,
    ))
}

fn resolve_compose_build_path(
    source_dir: &Path,
    raw: &str,
    source_path: &Path,
    service_id: &str,
    field: &str,
) -> Result<PathBuf> {
    let raw_path = PathBuf::from(raw);
    let candidate = if raw_path.is_absolute() {
        raw_path
    } else {
        source_dir.join(raw_path)
    };
    fs::canonicalize(&candidate).with_context(|| {
        format!(
            "compose file `{}` service `{service_id}` cannot resolve {field} `{raw}` from `{}`",
            source_path.display(),
            candidate.display()
        )
    })
}

fn compose_build_args(
    args: Option<&ComposeEnv>,
    source_path: &Path,
    service_id: &str,
) -> Result<Vec<(String, String)>> {
    let Some(args) = args else {
        return Ok(Vec::new());
    };
    match args {
        ComposeEnv::List(entries) => entries
            .iter()
            .map(|entry| {
                entry.split_once('=').map_or_else(
                    || {
                        Err(anyhow::anyhow!(
                            "compose file `{}` service `{service_id}` uses build.args entry `{entry}` without `=`",
                            source_path.display()
                        ))
                    },
                    |(key, value)| Ok((key.to_string(), value.to_string())),
                )
            })
            .collect(),
        ComposeEnv::Map(entries) => entries
            .iter()
            .map(|(key, value)| {
                value.clone().map_or_else(
                    || {
                        Err(anyhow::anyhow!(
                            "compose file `{}` service `{service_id}` uses build.args map key `{key}` with a null value",
                            source_path.display()
                        ))
                    },
                    |value| Ok((key.clone(), value)),
                )
            })
            .collect(),
    }
}

/// Produce a runtime-local OCI image reference that remains visibly scoped to
/// its compose project and service, while the BLAKE3 suffix preserves the
/// exact raw pair. A delimiter-only tag such as `vat-{project}-{service}` is
/// ambiguous (`a`/`b-c` versus `a-b`/`c`) and lossy normalization can collapse
/// distinct Compose identifiers, so it is not safe as an ownership key.
fn compose_build_tag(project: &str, service_id: &str) -> String {
    let mut identity = blake3::Hasher::new();
    identity.update(project.as_bytes());
    identity.update(&[0]);
    identity.update(service_id.as_bytes());
    format!(
        "vat-{}-{}-b3-{}:latest",
        sanitize_tag_component(project, "project"),
        sanitize_tag_component(service_id, "service"),
        identity.finalize().to_hex(),
    )
}

/// Render a short, OCI-safe readable prefix. Its value is deliberately not
/// used as the identity proof: compose_build_tag adds the full raw-pair digest.
fn sanitize_tag_component(input: &str, fallback: &str) -> String {
    let mut out = String::new();
    let mut separator_pending = false;
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() {
            if separator_pending && !out.is_empty() {
                out.push('-');
            }
            out.push((byte as char).to_ascii_lowercase());
            separator_pending = false;
        } else if !out.is_empty() {
            // OCI path components cannot begin/end with a separator or place
            // multiple punctuation separators together. The digest in the
            // final tag retains the information this readable prefix drops.
            separator_pending = true;
        }
    }
    if out.is_empty() {
        fallback.to_string()
    } else {
        // Keep the human-readable part bounded; the fixed BLAKE3 suffix keeps
        // the complete reference comfortably within common OCI name limits.
        const READABLE_COMPONENT_LIMIT: usize = 40;
        out.truncate(READABLE_COMPONENT_LIMIT);
        out.trim_end_matches('-').to_string()
    }
}

fn compose_import_retry_command(
    source_path: &Path,
    project: &str,
    runtime: ServiceRuntime,
) -> String {
    format!(
        "vat compose import {} --project {} --runtime {}",
        shell_quote(&source_path.to_string_lossy()),
        shell_quote(project),
        runtime_name(runtime)
    )
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\\"'\\\"'"))
}

fn runtime_name(runtime: ServiceRuntime) -> &'static str {
    match runtime {
        ServiceRuntime::Auto => "auto",
        ServiceRuntime::Native => "native",
        ServiceRuntime::Docker => "docker",
        ServiceRuntime::MicroVm => "micro-vm",
    }
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
    atomic_write(out, toml.as_bytes())?;
    println!("Wrote {}", out.display());
    Ok(())
}

/// Replace generated vat.toml atomically. A failed runtime preflight or build
/// must leave the previous imported project intact rather than exposing a
/// truncated configuration to a concurrent compose command.
fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .context("materialized vat.toml path has no parent directory")?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .context("materialized vat.toml path has no UTF-8 file name")?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", crate::id::fresh()));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .with_context(|| format!("create temporary vat.toml {}", temporary.display()))?;
        file.write_all(contents)
            .with_context(|| format!("write temporary vat.toml {}", temporary.display()))?;
        file.sync_all()
            .with_context(|| format!("sync temporary vat.toml {}", temporary.display()))?;
        drop(file);
        fs::rename(&temporary, path).with_context(|| {
            format!(
                "atomically replace materialized vat.toml {} from {}",
                path.display(),
                temporary.display()
            )
        })?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

/// Restore the previous generated config if import failed after replacing
/// vat.toml but before publishing its matching compose registry record.
/// Without that record, `compose up` fails closed, but rollback keeps ordinary
/// write failures from unnecessarily leaving an unusable import behind.
pub(crate) fn restore_materialized_config(path: &Path, previous: Option<&[u8]>) -> Result<()> {
    match previous {
        Some(contents) => atomic_write(path, contents),
        None => match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error).with_context(|| {
                format!("remove uncommitted materialized config {}", path.display())
            }),
        },
    }
}
// HANDWRITE-END
````

## Changes
<!-- type: changes lang: yaml -->

````yaml
changes:
  - path: apps/vat/src/compose.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1529 synchronizes the full hand-written source mirror: parse canonicalizes
      the compose source; strict Docker-shaped profiles capture that document
      before materialization so a later path replacement cannot widen it;
      build context/dockerfile/args resolve deterministically; build-only
      services receive OCI-safe readable project-scoped tags with BLAKE3
      raw-pair identity suffixes in their runtime-local store; materialize
      atomically replaces vat.toml only after preflight/build; and
      restore_materialized_config() supports rollback if the matching registry
      record cannot be published.
````
