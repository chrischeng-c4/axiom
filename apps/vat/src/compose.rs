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
use anyhow::{Context, Result, bail};
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
    _volumes: DeclaredComposeField<BTreeMap<String, Value>>,
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
        self.services.get(service_id).map_or(false, |service| {
            service.image.is_none() && service.build.is_some()
        })
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
    volumes: DeclaredComposeField<Vec<String>>,
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

/// Preserve whether an optional Compose key was explicitly declared. This is
/// needed for strict profiles where `volumes: []` is still an unsupported
/// Compose feature, while ordinary import may simply have no volume entries
/// to materialize. An explicit YAML `null` is rejected while deserializing
/// rather than being silently treated as absent.
#[derive(Debug, Default)]
enum DeclaredComposeField<T> {
    #[default]
    Absent,
    Present(T),
}

impl<T> DeclaredComposeField<T> {
    fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Absent => None,
            Self::Present(value) => Some(value),
        }
    }

    fn is_declared(&self) -> bool {
        matches!(self, Self::Present(_))
    }
}

impl<'de, T> serde::Deserialize<'de> for DeclaredComposeField<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Self::Present)
    }
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
        if let Some(volumes) = service.volumes.as_ref() {
            for vol in volumes {
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
    }

    Ok(file)
}

/// The intentionally narrow Compose contracts accepted by VAT's Docker-shaped
/// shim. These are not general Compose versions: each kind names the exact
/// runtime behavior that has been validated against Apple Container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DockerComposeProfile {
    StrictSingleImageV1,
    StrictSingleBuildV1,
    HostFacingIndependentV1,
}

impl DockerComposeProfile {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::StrictSingleImageV1 => "strict-single-image-v1",
            Self::StrictSingleBuildV1 => "strict-single-build-v1",
            Self::HostFacingIndependentV1 => "host-facing-independent-v1",
        }
    }

    /// Registry provenance is durable JSON, so Docker-shaped lifecycle
    /// commands must reject an absent, legacy, or invented profile string
    /// rather than treating any non-empty value as an authority grant.
    pub(crate) fn is_known(value: &str) -> bool {
        matches!(
            value,
            "strict-single-image-v1" | "strict-single-build-v1" | "host-facing-independent-v1"
        )
    }
}

/// A strict Compose document captured before VAT invokes a runtime. Carrying
/// the parsed document prevents a later path replacement from widening the
/// preflighted subset between validation and materialization.
pub(crate) struct ParsedDockerComposeProfile {
    pub(crate) file: ComposeFile,
    pub(crate) profile: DockerComposeProfile,
}

/// Parse and capture the literal-image Docker Compose profile. A file that
/// explicitly declares `x-vat-compose-profile: host-facing-independent-v1`
/// selects the bounded multi-service contract; files without that marker keep
/// the pre-existing strict single-image behavior.
pub(crate) fn parse_docker_compose_compat_profile(
    path: &Path,
) -> Result<ParsedDockerComposeProfile> {
    let file = parse(path)?;
    let profile = match requested_docker_compose_image_profile(&file)? {
        Some(DockerComposeProfile::HostFacingIndependentV1) => {
            validate_host_facing_independent_profile(&file)?;
            DockerComposeProfile::HostFacingIndependentV1
        }
        None => {
            validate_docker_compose_compat_profile_with_mode(
                &file,
                DockerComposeCompatMode::Image,
            )?;
            DockerComposeProfile::StrictSingleImageV1
        }
        Some(profile) => unreachable!("only host-facing image profile is selectable: {profile:?}"),
    };
    Ok(ParsedDockerComposeProfile { file, profile })
}

/// Parse and capture the strict source-build profile. It is intentionally a
/// separate mode rather than a relaxation of the literal-image profile:
/// `up -d --build` has a concrete typed build contract, while silently
/// accepting `build:` on normal `up -d` would hide a meaningful Docker Compose
/// semantic difference from an agent.
pub(crate) fn parse_docker_compose_build_compat_profile(
    path: &Path,
) -> Result<ParsedDockerComposeProfile> {
    let file = parse(path)?;
    validate_docker_compose_compat_profile_with_mode(&file, DockerComposeCompatMode::Build)?;
    Ok(ParsedDockerComposeProfile {
        file,
        profile: DockerComposeProfile::StrictSingleBuildV1,
    })
}

#[derive(Clone, Copy)]
enum DockerComposeCompatMode {
    Image,
    Build,
}

/// Select a literal-image profile from its sole permitted top-level extension.
/// Any extension other than the exact opt-in marker remains a fail-closed
/// rejection rather than a generic Compose escape hatch.
fn requested_docker_compose_image_profile(
    file: &ComposeFile,
) -> Result<Option<DockerComposeProfile>> {
    if file.extra.is_empty() {
        return Ok(None);
    }
    if file.extra.len() != 1 {
        bail!(
            "VAT's docker compose compatibility profile accepts no extension keys except the sole exact `x-vat-compose-profile: host-facing-independent-v1` marker"
        );
    }
    let Some(marker) = file.extra.get("x-vat-compose-profile") else {
        bail!(
            "VAT's docker compose compatibility profile accepts no extension keys except `x-vat-compose-profile: host-facing-independent-v1`"
        );
    };
    let marker = marker.as_str().context(
        "VAT's `x-vat-compose-profile` must be the literal string `host-facing-independent-v1`",
    )?;
    if marker != DockerComposeProfile::HostFacingIndependentV1.as_str() {
        bail!(
            "unsupported VAT docker compose profile `{marker}`; the only multi-service profile is `host-facing-independent-v1`"
        );
    }
    Ok(Some(DockerComposeProfile::HostFacingIndependentV1))
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
    if file
        ._volumes
        .as_ref()
        .is_some_and(|volumes| !volumes.is_empty())
    {
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
    if service
        .volumes
        .as_ref()
        .is_some_and(|volumes| !volumes.is_empty())
    {
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

/// Validate the explicit multi-service profile. It deliberately supports only
/// independent host-facing processes: VAT starts each image through the
/// existing Compose lifecycle, but never claims a Docker bridge network or
/// service-name DNS between them.
fn validate_host_facing_independent_profile(file: &ComposeFile) -> Result<()> {
    if file._volumes.is_declared() {
        bail!("VAT's host-facing-independent-v1 profile does not support top-level volumes");
    }
    if !(2..=4).contains(&file.services.len()) {
        bail!(
            "VAT's host-facing-independent-v1 profile requires 2 through 4 independently host-facing services; it does not provide general Compose topology or bridge DNS"
        );
    }

    let mut host_ports = BTreeMap::new();
    for (service_id, service) in &file.services {
        validate_host_facing_service_id(service_id)?;
        if !service.extra.is_empty() {
            bail!(
                "VAT's host-facing-independent-v1 profile service `{service_id}` uses unsupported keys; only literal image, one host:container port, and literal environment are allowed"
            );
        }
        if service.build.is_some() {
            bail!(
                "VAT's host-facing-independent-v1 profile service `{service_id}` may not use build; every service must name one literal image"
            );
        }
        let image = service.image.as_deref().context(format!(
            "VAT's host-facing-independent-v1 profile service `{service_id}` requires one literal image"
        ))?;
        ensure_host_facing_literal(image, service_id, "image")?;
        if service.depends_on.is_some() {
            bail!(
                "VAT's host-facing-independent-v1 profile service `{service_id}` may not use depends_on; service-name DNS and startup topology are unsupported"
            );
        }
        if service.volumes.is_declared() {
            bail!(
                "VAT's host-facing-independent-v1 profile service `{service_id}` may not use volumes"
            );
        }
        if service.ports.len() != 1 {
            bail!(
                "VAT's host-facing-independent-v1 profile service `{service_id}` requires exactly one explicit nonzero host:container port"
            );
        }
        let (host_port, _) = validate_docker_compose_port(service_id, &service.ports[0])?;
        if let Some(other_service) = host_ports.insert(host_port, service_id) {
            bail!(
                "VAT's host-facing-independent-v1 profile services `{other_service}` and `{service_id}` both publish host port `{host_port}`; every host-facing service needs a unique host port"
            );
        }
        if let Some(environment) = &service.environment {
            validate_host_facing_environment(service_id, environment)?;
        }
    }
    Ok(())
}

fn validate_host_facing_service_id(service_id: &str) -> Result<()> {
    let mut bytes = service_id.bytes();
    let Some(first) = bytes.next() else {
        bail!(
            "VAT's host-facing-independent-v1 profile requires service ids matching [a-z0-9][a-z0-9_-]*"
        );
    };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit())
        || !bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
    {
        bail!(
            "VAT's host-facing-independent-v1 profile service `{service_id}` must match [a-z0-9][a-z0-9_-]*"
        );
    }
    Ok(())
}

fn ensure_host_facing_literal(value: &str, service_id: &str, field: &str) -> Result<()> {
    ensure_docker_compose_literal(value, service_id, field)?;
    if value.trim() != value
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        bail!(
            "VAT's host-facing-independent-v1 profile service `{service_id}` field `{field}` must be one whitespace-free literal"
        );
    }
    Ok(())
}

fn validate_host_facing_environment(service_id: &str, environment: &ComposeEnv) -> Result<()> {
    match environment {
        ComposeEnv::List(entries) => {
            for entry in entries {
                ensure_host_facing_literal(entry, service_id, "environment")?;
            }
        }
        ComposeEnv::Map(entries) => {
            for (key, value) in entries {
                ensure_host_facing_literal(key, service_id, "environment key")?;
                ensure_host_facing_literal(
                    value
                        .as_deref()
                        .context("validated non-null host-facing environment value")?,
                    service_id,
                    "environment value",
                )?;
            }
        }
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

fn validate_docker_compose_port(service_id: &str, port: &str) -> Result<(u16, u16)> {
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
    Ok((host, container))
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
        if let Some(volumes) = service.volumes.as_ref() {
            for volume in volumes {
                if let Some(colon_idx) = volume.find(':') {
                    config.volumes.push(VolumeMount {
                        name: volume[..colon_idx].to_string(),
                        path: volume[colon_idx + 1..].to_string(),
                    });
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn validate_profile(source: &str) -> Result<()> {
        let directory = tempfile::tempdir().expect("compose profile tempdir");
        let path = directory.path().join("compose.yml");
        fs::write(&path, source).expect("write compose profile fixture");
        parse_docker_compose_compat_profile(&path).map(|_| ())
    }

    fn validate_build_profile(source: &str) -> Result<()> {
        let directory = tempfile::tempdir().expect("compose build profile tempdir");
        let path = directory.path().join("compose.yml");
        fs::write(&path, source).expect("write compose build profile fixture");
        parse_docker_compose_build_compat_profile(&path).map(|_| ())
    }

    #[test]
    fn docker_compat_profile_accepts_one_literal_image_service() {
        validate_profile(
            r#"
services:
  web:
    image: nginx:1.27-alpine
    ports:
      - "18080:80"
    environment:
      MODE: test
"#,
        )
        .expect("strict compatible compose profile");
    }

    #[test]
    fn docker_compat_profile_captures_document_before_path_mutation() {
        let directory = tempfile::tempdir().expect("compose profile tempdir");
        let path = directory.path().join("compose.yml");
        fs::write(
            &path,
            r#"services:
  web:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
"#,
        )
        .expect("write strict source compose file");
        let captured = parse_docker_compose_compat_profile(&path)
            .expect("capture strict source compose profile");

        // A later replacement would be accepted by general `vat compose
        // import`, but must not change the already captured strict shim input.
        fs::write(
            &path,
            r#"services:
  web:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
  sidecar:
    image: busybox:1.36
    ports: ["18081:80"]
"#,
        )
        .expect("replace compose path after capture");

        let services = expand(&captured.file, "captured-profile", ServiceRuntime::MicroVm)
            .expect("expand the captured strict compose profile");
        assert_eq!(
            services.len(),
            1,
            "replacement must not widen captured profile"
        );
        assert_eq!(services[0].id, "web");
        assert_eq!(services[0].image.as_deref(), Some("nginx:1.27-alpine"));
    }

    #[test]
    fn docker_host_facing_independent_profile_accepts_two_literal_image_services() {
        let directory = tempfile::tempdir().expect("host-facing compose profile tempdir");
        let path = directory.path().join("compose.yml");
        fs::write(
            &path,
            r#"
x-vat-compose-profile: host-facing-independent-v1
services:
  docs:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
    environment:
      MODE: docs
  inspector:
    image: nginx:1.27-alpine
    ports: ["18081:80"]
    environment:
      MODE: inspect
"#,
        )
        .expect("write host-facing compose profile");

        let captured = parse_docker_compose_compat_profile(&path)
            .expect("capture host-facing compose profile");
        assert_eq!(
            captured.profile,
            DockerComposeProfile::HostFacingIndependentV1
        );
        let services = expand(&captured.file, "local-tools", ServiceRuntime::MicroVm)
            .expect("expand independent host-facing services");
        assert_eq!(services.len(), 2);
        assert_eq!(services[0].id, "docs");
        assert_eq!(services[0].port, PortSpec::Fixed(18080));
        assert_eq!(services[1].id, "inspector");
        assert_eq!(services[1].port, PortSpec::Fixed(18081));
    }

    #[test]
    fn docker_host_facing_independent_profile_rejects_topology_or_lossy_shapes() {
        for source in [
            r#"
x-vat-compose-profile: host-facing-independent-v1
services:
  docs:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
"#,
            r#"
x-vat-compose-profile: host-facing-independent-v1
services:
  docs:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
  inspector:
    image: nginx:1.27-alpine
    ports: ["18080:81"]
"#,
            r#"
x-vat-compose-profile: host-facing-independent-v1
services:
  Docs:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
  inspector:
    image: nginx:1.27-alpine
    ports: ["18081:80"]
"#,
            r#"
x-vat-compose-profile: host-facing-independent-v1
services:
  docs:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
    depends_on: [inspector]
  inspector:
    image: nginx:1.27-alpine
    ports: ["18081:80"]
"#,
            r#"
x-vat-compose-profile: host-facing-independent-v1
services:
  docs:
    build: .
    ports: ["18080:80"]
  inspector:
    image: nginx:1.27-alpine
    ports: ["18081:80"]
"#,
            r#"
x-vat-compose-profile: host-facing-independent-v1
services:
  docs:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
    volumes: []
  inspector:
    image: nginx:1.27-alpine
    ports: ["18081:80"]
"#,
            r#"
x-vat-compose-profile: host-facing-independent-v1
volumes: {}
services:
  docs:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
  inspector:
    image: nginx:1.27-alpine
    ports: ["18081:80"]
"#,
            r#"
x-vat-compose-profile: host-facing-independent-v1
services:
  docs:
    image: ${DOCS_IMAGE}
    ports: ["18080:80"]
  inspector:
    image: nginx:1.27-alpine
    ports: ["18081:80"]
"#,
            r#"
x-vat-compose-profile: host-facing-independent-v1
services:
  docs:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
  inspector:
    image: nginx:1.27-alpine
    ports: ["18081:80"]
    networks: []
"#,
            r#"
x-vat-compose-profile: bridge-like-v1
services:
  docs:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
  inspector:
    image: nginx:1.27-alpine
    ports: ["18081:80"]
"#,
        ] {
            assert!(
                validate_profile(source).is_err(),
                "host-facing profile must fail closed for unsupported Compose shape: {source}"
            );
        }
    }

    #[test]
    fn docker_compat_profile_rejects_lossy_or_unmodeled_compose_shapes() {
        for source in [
            r#"services:
  web:
    image: nginx:alpine
    build: .
    ports: ["18080:80"]
"#,
            r#"services:
  web:
    image: nginx:alpine
    ports: ["18080:80", "18081:81"]
"#,
            r#"services:
  web:
    image: nginx:alpine
    ports: ["18080:80"]
    depends_on: [db]
  db:
    image: postgres:16
    ports: ["15432:5432"]
"#,
            r#"services:
  web:
    image: ${IMAGE}
    ports: ["18080:80"]
"#,
            r#"services:
  web:
    image: nginx:alpine
    ports: ["127.0.0.1:18080:80"]
"#,
        ] {
            assert!(
                validate_profile(source).is_err(),
                "lossy compose shape must fail closed: {source}"
            );
        }
    }

    #[test]
    fn docker_compat_build_profile_accepts_one_literal_short_build_service() {
        validate_build_profile(
            r#"
services:
  web:
    build: .
    ports:
      - "18080:80"
    environment:
      MODE: dev
"#,
        )
        .expect("strict build-compatible compose profile");
    }

    #[test]
    fn docker_compat_build_profile_rejects_image_or_lossy_build_shapes() {
        for source in [
            r#"services:
  web:
    image: nginx:alpine
    build: .
    ports: ["18080:80"]
"#,
            r#"services:
  web:
    build:
      context: .
    ports: ["18080:80"]
"#,
            r#"services:
  web:
    build: ${CONTEXT}
    ports: ["18080:80"]
"#,
            r#"services:
  web:
    build: .
    ports: ["18080:80"]
  worker:
    build: .
    ports: ["18081:81"]
"#,
            r#"services:
  web:
    build: .
    ports: ["18080:80"]
    volumes: [cache:/var/cache/app]
"#,
            r#"services:
  web:
    build: .
"#,
        ] {
            assert!(
                validate_build_profile(source).is_err(),
                "lossy strict build compose shape must fail closed: {source}"
            );
        }
    }
}
// HANDWRITE-END
