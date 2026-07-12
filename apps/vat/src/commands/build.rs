// HANDWRITE-BEGIN gap="missing-generator:logic:102c3b59" tracker="pending-tracker" reason="R1-R4: new `Args`/`BuildReport` structs, `exec()`, `build_image()`, `container_build_command()`, and `ensure_microvm_available()`. `container_build_command()` is a mechanical argv builder mirroring `sandbox/microvm.rs`'s `resolve()` (itself codegen-owned), and `ensure_microvm_available()` structurally mirrors `run.rs`'s `ensure_docker_available`; but `exec()`'s R3 divergence — streaming build output live (inherited stdio) in human mode vs. capturing output and returning only the structured `BuildReport` in JSON mode — is a genuinely new pattern (no other vat command proxies a long-running streamed subprocess today), so the whole file is hand-authored this WI (missing-generator:cli:streamed-subprocess-dual-mode, tracker #1479)."

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

use anyhow::{bail, Context, Result};

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
            arg.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect();

    // Fail-closed gates shared by both modes (Logic: dockerfile_check then avail_check,
    // both ahead of mode_check). The default (no --file) Dockerfile path is never
    // existence-checked above, and the human-mode branch below never calls
    // build_image() — so without these, `vat build` (no --json) would silently skip
    // both AC3's missing-Dockerfile gate and ensure_microvm_available()'s fail-closed
    // container-system check. build_image() re-validates both internally for its own
    // direct callers (Phase 3 `vat compose`), which is intentionally redundant here.
    if !dockerfile.exists() {
        bail!("dockerfile not found: {}", dockerfile.display());
    }
    ensure_microvm_available()?;

    // Dispatch: JSON mode calls build_image (captured output), human mode streams direct.
    if args.json {
        match build_image(&context, &dockerfile, &tag, &build_args) {
            Ok(report) => {
                crate::commands::print_json(&report, false)?;
                Ok(ExitCode::SUCCESS)
            }
            Err(e) => {
                crate::commands::print_json(
                    &serde_json::json!({ "error": e.to_string() }),
                    false,
                )?;
                Ok(ExitCode::FAILURE)
            }
        }
    } else {
        // Human mode: stream output directly, print summary on success.
        let argv = container_build_command(&dockerfile, &tag, &build_args, &context);
        let started = Instant::now();
        let status = Command::new(&argv[0])
            .args(&argv[1..])
            .status()
            .context("spawn container build")?;

        if !status.success() {
            return Ok(ExitCode::FAILURE);
        }

        let duration_ms = started.elapsed().as_millis() as u64;
        println!("{} ({} ms)", tag, duration_ms);
        Ok(ExitCode::SUCCESS)
    }
}

/// In-process build entry point for Phase 3 `vat compose` to call directly.
/// Validates Dockerfile exists, ensures container system is available, spawns
/// `container build` with captured output, and returns either BuildReport or error.
pub fn build_image(
    context: &Path,
    dockerfile: &Path,
    tag: &str,
    build_args: &[(String, String)],
) -> Result<BuildReport> {
    // Fail-closed: ensure dockerfile exists before any subprocess invocation (AC3).
    if !dockerfile.exists() {
        bail!("dockerfile not found: {}", dockerfile.display());
    }

    ensure_microvm_available()?;

    let argv = container_build_command(dockerfile, tag, build_args, context);
    let started = Instant::now();

    // Spawn with captured stdout/stderr (no inheritance in Phase 3 compose mode).
    let output = Command::new(&argv[0])
        .args(&argv[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("spawn container build")?;

    let duration_ms = started.elapsed().as_millis() as u64;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("container build failed: {}", stderr);
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

/// Ensures the container CLI is available and the system is responsive.
/// Mirrors run.rs's ensure_docker_available: fail-closed if system is unavailable.
fn ensure_microvm_available() -> Result<()> {
    if !sandbox::microvm::available() {
        bail!("container CLI not found on PATH; install it with `brew install colima` or similar");
    }

    if !sandbox::microvm::system_up() {
        sandbox::microvm::ensure_system_started(std::time::Duration::from_secs(30))
            .map_err(|e| anyhow::anyhow!(e))?;
    }

    Ok(())
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
    fn sanitize_tag_lowercases_and_collapses() {
        assert_eq!(sanitize_tag("MyImage:Latest"), "myimage-latest");
        assert_eq!(sanitize_tag("test@image#tag"), "test-image-tag");
        assert_eq!(sanitize_tag("valid.tag_123-name"), "valid.tag_123-name");
    }
}
// HANDWRITE-END
