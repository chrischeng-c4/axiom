// HANDWRITE-BEGIN gap="missing-generator:logic:102c3b59" tracker="#1479" reason="R1-R4 plus #1529: `Args`/`BuildReport`, streamed-versus-captured builds, runtime-to-image-store selection, bounded Docker/Apple-Container preflight, and matching argv builders live together because compose must build into the exact store its generated service will run from. No generator owns this dual-runtime, dual-stdio subprocess protocol, so the file remains hand-authored (missing-generator:cli:streamed-subprocess-dual-mode; trackers #1479 and #1529)."

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

use crate::config::ServiceRuntime;
use crate::sandbox;

/// Inputs for `vat build`.
pub struct Args {
    pub file: Option<PathBuf>,
    pub context: Option<PathBuf>,
    pub tag: Option<String>,
    pub build_arg: Vec<String>,
    pub json: bool,
}

/// Successful build output, serialized as JSON in `--json` mode.
#[derive(Debug, serde::Serialize)]
pub struct BuildReport {
    pub tag: String,
    pub dockerfile: String,
    pub context: String,
    pub build_args: BTreeMap<String, String>,
    pub duration_ms: u64,
}

/// The local image store into which a build is written. Compose must select
/// this from the same runtime its generated image service will later use;
/// Docker and Apple Container do not share image stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageBuilder {
    Docker,
    MicroVm,
}

impl ImageBuilder {
    fn command_name(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::MicroVm => "container",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Docker => "Docker",
            Self::MicroVm => "Apple Container",
        }
    }
}

/// Resolve an image-backed service runtime into its owning build runtime and
/// prove that runtime is usable before a compose import materializes state.
/// `auto` and `native` intentionally retain the existing image-service
/// behavior: both use Docker; only `microvm` uses Apple's Container store.
pub(crate) fn resolve_image_builder(runtime: ServiceRuntime) -> Result<ImageBuilder> {
    let builder = image_builder_for_runtime(runtime);
    ensure_image_builder_available(builder)?;
    Ok(builder)
}

/// Pure runtime-to-store mapping. It intentionally mirrors `vat run` image
/// dispatch so a generated compose image cannot be built into one store and
/// started from another.
pub(crate) fn image_builder_for_runtime(runtime: ServiceRuntime) -> ImageBuilder {
    match runtime {
        ServiceRuntime::MicroVm => ImageBuilder::MicroVm,
        ServiceRuntime::Auto | ServiceRuntime::Docker | ServiceRuntime::Native => {
            ImageBuilder::Docker
        }
    }
}

/// Main entry point for `vat build`. Resolves defaults, validates the Dockerfile path,
/// and dispatches to either human-mode (inherited stdio) or JSON mode (captured output).
pub fn exec(args: Args) -> Result<ExitCode> {
    // Resolve context to absolute path (defaults to current directory).
    let context = match args.context {
        Some(p) => std::fs::canonicalize(&p)
            .with_context(|| format!("resolve context dir {}", p.display()))?,
        None => std::env::current_dir().context("get current directory")?,
    };

    // Resolve dockerfile path (defaults to Dockerfile inside context).
    let dockerfile = match args.file {
        Some(p) => std::fs::canonicalize(&p)
            .with_context(|| format!("resolve dockerfile path {}", p.display()))?,
        None => context.join("Dockerfile"),
    };

    // Resolve tag (defaults to `<context-dir-basename>:latest`, sanitized).
    let tag = args.tag.map(|t| t.to_string()).unwrap_or_else(|| {
        let basename = context
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("build");
        sanitize_tag(&format!("{}:latest", basename))
    });

    // Parse build_args from --build-arg K=V flags, preserving CLI order.
    let build_args: Vec<(String, String)> = args
        .build_arg
        .iter()
        .filter_map(|arg| {
            arg.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect();

    // Fail-closed gates shared by both modes (Logic: dockerfile_check then avail_check,
    // both ahead of mode_check). The default (no --file) Dockerfile path is never
    // existence-checked above, and the human-mode branch below never calls
    // build_image() — so without these, `vat build` (no --json) would silently skip
    // both AC3's missing-Dockerfile gate and resolve_image_builder()'s fail-closed
    // container-system check. build_image() re-validates both internally for its own
    // direct callers (Phase 3 `vat compose`), which is intentionally redundant here.
    if !dockerfile.exists() {
        bail!("dockerfile not found: {}", dockerfile.display());
    }
    let builder = resolve_image_builder(ServiceRuntime::MicroVm)?;

    // Dispatch: JSON mode calls build_image (captured output), human mode streams direct.
    if args.json {
        match build_image_with_builder(builder, &context, &dockerfile, &tag, &build_args) {
            Ok(report) => {
                crate::commands::print_json(&report, false)?;
                Ok(ExitCode::SUCCESS)
            }
            Err(e) => {
                crate::commands::print_json(&serde_json::json!({ "error": e.to_string() }), false)?;
                Ok(ExitCode::FAILURE)
            }
        }
    } else {
        // Human mode: stream output directly, print summary on success.
        let argv = build_command(builder, &dockerfile, &tag, &build_args, &context);
        let started = Instant::now();
        let status = Command::new(&argv[0])
            .args(&argv[1..])
            .status()
            .with_context(|| format!("spawn {} build", builder.command_name()))?;

        if !status.success() {
            return Ok(ExitCode::FAILURE);
        }

        let duration_ms = started.elapsed().as_millis() as u64;
        println!("{} ({} ms)", tag, duration_ms);
        Ok(ExitCode::SUCCESS)
    }
}

/// In-process MicroVM build entry point for the existing `vat build` surface.
/// Compose callers must use [`resolve_image_builder`] plus
/// [`build_image_with_builder`] so image placement matches their runtime.
pub fn build_image(
    context: &Path,
    dockerfile: &Path,
    tag: &str,
    build_args: &[(String, String)],
) -> Result<BuildReport> {
    let builder = resolve_image_builder(ServiceRuntime::MicroVm)?;
    build_image_with_builder(builder, context, dockerfile, tag, build_args)
}

/// Build into a builder that has already passed [`resolve_image_builder`].
/// This captures output for compose import while keeping the standalone
/// `vat build` human path streamed.
pub(crate) fn build_image_with_builder(
    builder: ImageBuilder,
    context: &Path,
    dockerfile: &Path,
    tag: &str,
    build_args: &[(String, String)],
) -> Result<BuildReport> {
    // Fail-closed: ensure dockerfile exists before any subprocess invocation (AC3).
    if !dockerfile.exists() {
        bail!("dockerfile not found: {}", dockerfile.display());
    }

    let argv = build_command(builder, dockerfile, tag, build_args, context);
    let started = Instant::now();

    // Spawn with captured stdout/stderr (no inheritance in Phase 3 compose mode).
    let output = Command::new(&argv[0])
        .args(&argv[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("spawn {} build", builder.command_name()))?;

    let duration_ms = started.elapsed().as_millis() as u64;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{} build failed: {}", builder.display_name(), stderr);
    }

    // Sort build_args for deterministic JSON field ordering in the report.
    let mut sorted_args = BTreeMap::new();
    for (k, v) in build_args {
        sorted_args.insert(k.clone(), v.clone());
    }

    Ok(BuildReport {
        tag: tag.to_string(),
        dockerfile: dockerfile.to_string_lossy().into_owned(),
        context: context.to_string_lossy().into_owned(),
        build_args: sorted_args,
        duration_ms,
    })
}

fn build_command(
    builder: ImageBuilder,
    dockerfile: &Path,
    tag: &str,
    build_args: &[(String, String)],
    context: &Path,
) -> Vec<String> {
    match builder {
        ImageBuilder::Docker => docker_build_command(dockerfile, tag, build_args, context),
        ImageBuilder::MicroVm => container_build_command(dockerfile, tag, build_args, context),
    }
}

/// Pure argv builder: returns exactly ["container", "build", "-f", dockerfile, "-t", tag,
/// "--build-arg", "K=V", ... context] per R2.
fn container_build_command(
    dockerfile: &Path,
    tag: &str,
    build_args: &[(String, String)],
    context: &Path,
) -> Vec<String> {
    let mut argv = vec![
        "container".to_string(),
        "build".to_string(),
        "-f".to_string(),
        dockerfile.to_string_lossy().into_owned(),
        "-t".to_string(),
        tag.to_string(),
    ];

    for (k, v) in build_args {
        argv.push("--build-arg".to_string());
        argv.push(format!("{}={}", k, v));
    }

    argv.push(context.to_string_lossy().into_owned());
    argv
}

/// Pure Docker argv builder mirroring [`container_build_command`]. Docker
/// keeps an independent image store, so compose cannot substitute one command
/// for the other even though the flag shapes match.
fn docker_build_command(
    dockerfile: &Path,
    tag: &str,
    build_args: &[(String, String)],
    context: &Path,
) -> Vec<String> {
    let mut argv = vec![
        "docker".to_string(),
        "build".to_string(),
        "-f".to_string(),
        dockerfile.to_string_lossy().into_owned(),
        "-t".to_string(),
        tag.to_string(),
    ];

    for (key, value) in build_args {
        argv.push("--build-arg".to_string());
        argv.push(format!("{key}={value}"));
    }

    argv.push(context.to_string_lossy().into_owned());
    argv
}

/// Ensure a selected runtime is available before compose writes a generated
/// vat.toml. Docker probing is bounded here rather than reusing run.rs's
/// unbounded service-time probe: import must not hang before materialization.
fn ensure_image_builder_available(builder: ImageBuilder) -> Result<()> {
    match builder {
        ImageBuilder::Docker => ensure_docker_builder_available(),
        ImageBuilder::MicroVm => ensure_microvm_available(),
    }
}

/// Ensures the container CLI is available and the system is responsive.
fn ensure_microvm_available() -> Result<()> {
    if !sandbox::microvm::available() {
        bail!(
            "Apple Container CLI not found on PATH; install Apple's `container` CLI (for example `brew install container`) and retry"
        );
    }

    if !sandbox::microvm::system_up() {
        sandbox::microvm::ensure_system_started(std::time::Duration::from_secs(30))
            .map_err(|e| anyhow::anyhow!(e))?;
    }

    Ok(())
}

fn ensure_docker_builder_available() -> Result<()> {
    let mut child = match Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!(
                "Docker builder unavailable: `docker` was not found on PATH; install/start Docker, then retry `docker info`"
            );
        }
        Err(error) => return Err(error).context("spawn bounded `docker info` builder probe"),
    };
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = child
            .try_wait()
            .context("poll bounded `docker info` builder probe")?
        {
            if status.success() {
                return Ok(());
            }
            bail!(
                "Docker builder unavailable: `docker info` failed; start Docker, then retry `docker info`"
            );
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!(
                "Docker builder unavailable: `docker info` did not respond within 2s; start Docker, then retry `docker info`"
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Sanitize a tag to be a valid OCI reference: lowercase, collapse non [a-z0-9._-] to `-`.
fn sanitize_tag(tag: &str) -> String {
    tag.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_build_command_shape() {
        let dockerfile = PathBuf::from("/tmp/Dockerfile");
        let context = PathBuf::from("/project");
        let tag = "myimage:latest";
        let build_args = vec![
            ("VERSION".to_string(), "1.0".to_string()),
            ("DEBUG".to_string(), "true".to_string()),
        ];

        let argv = container_build_command(&dockerfile, tag, &build_args, &context);

        assert_eq!(argv[0], "container");
        assert_eq!(argv[1], "build");
        assert_eq!(argv[2], "-f");
        assert_eq!(argv[3], "/tmp/Dockerfile");
        assert_eq!(argv[4], "-t");
        assert_eq!(argv[5], "myimage:latest");
        assert_eq!(argv[6], "--build-arg");
        assert_eq!(argv[7], "VERSION=1.0");
        assert_eq!(argv[8], "--build-arg");
        assert_eq!(argv[9], "DEBUG=true");
        assert_eq!(argv[10], "/project");
    }

    #[test]
    fn docker_build_command_shape() {
        let dockerfile = PathBuf::from("/tmp/Dockerfile");
        let context = PathBuf::from("/project");
        let build_args = vec![("VERSION".to_string(), "1.0".to_string())];

        assert_eq!(
            docker_build_command(&dockerfile, "vat-demo-web:latest", &build_args, &context),
            vec![
                "docker",
                "build",
                "-f",
                "/tmp/Dockerfile",
                "-t",
                "vat-demo-web:latest",
                "--build-arg",
                "VERSION=1.0",
                "/project",
            ]
        );
    }

    #[test]
    fn image_builder_resolution_matches_image_service_runtime_dispatch() {
        assert_eq!(
            image_builder_for_runtime(ServiceRuntime::Auto),
            ImageBuilder::Docker
        );
        assert_eq!(
            image_builder_for_runtime(ServiceRuntime::Docker),
            ImageBuilder::Docker
        );
        assert_eq!(
            image_builder_for_runtime(ServiceRuntime::Native),
            ImageBuilder::Docker
        );
        assert_eq!(
            image_builder_for_runtime(ServiceRuntime::MicroVm),
            ImageBuilder::MicroVm
        );
    }

    #[test]
    fn sanitize_tag_lowercases_and_collapses() {
        assert_eq!(sanitize_tag("MyImage:Latest"), "myimage-latest");
        assert_eq!(sanitize_tag("test@image#tag"), "test-image-tag");
        assert_eq!(sanitize_tag("valid.tag_123-name"), "valid.tag_123-name");
    }
}
// HANDWRITE-END
