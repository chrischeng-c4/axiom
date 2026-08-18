// CODEGEN-BEGIN
//! MicroVM sandbox backend for Apple Silicon via the `container` CLI.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::spec::EgressPolicy;

/// Each Apple Container observation has a hard bound. The host CLI has been
/// observed to leave `stats` and `system df` pending indefinitely, so this is
/// intentionally shorter than a runtime readiness wait and advisory-only.
const BUILDER_ADVISORY_TIMEOUT: Duration = Duration::from_millis(500);

/// Read-only observation of Apple's singleton BuildKit builder.
///
/// Apple Container exposes no per-project builder identity: VAT therefore
/// treats every builder/cache/disk value as shared host state and never owns
/// its lifecycle. This model deliberately keeps configured builder resources
/// separate from process-observed usage.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AppleContainerBuilderAdvisory {
    /// Apple-reported builder state, or one of `unavailable`, `not_running`,
    /// or `unknown` when a read-only probe cannot establish it.
    pub state: String,
    /// The Apple CLI exposes a singleton builder without VAT/project ownership.
    pub ownership: String,
    /// VAT never starts, stops, deletes, or prunes this shared builder.
    pub automatic_cleanup: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<AppleContainerBuilderConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_stats: Option<AppleContainerBuilderObservedStats>,
    /// `container system df` is host-global rather than attributable to VAT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_disk: Option<AppleContainerGlobalDiskAdvisory>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub probe_errors: Vec<AppleContainerProbeError>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppleContainerBuilderConfiguration {
    pub id: String,
    pub resources: AppleContainerBuilderResources,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppleContainerBuilderResources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppleContainerBuilderObservedStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_usage_usec: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_count: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppleContainerGlobalDiskAdvisory {
    /// The Apple CLI reports host-global totals, not VAT-owned resources.
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containers: Option<AppleContainerDiskUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<AppleContainerDiskUsage>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppleContainerDiskUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reclaimable_bytes: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppleContainerProbeError {
    pub probe: String,
    pub message: String,
}

/// MicroVmBackend runs a workload inside an ephemeral Apple `container` microVM.
#[derive(Debug, Clone)]
pub struct MicroVmBackend {
    /// Network egress policy (Open or Deny; LocalhostOnly is rejected at pick() time).
    pub egress: EgressPolicy,
    /// Environment variables to inject (deterministic BTreeMap ordering for stable argv).
    pub env: BTreeMap<String, String>,
    /// Working directory inside the container (nested under /workspace).
    pub workdir: std::path::PathBuf,
    /// OCI image reference for the container.
    pub image: String,
}

impl super::Sandbox for MicroVmBackend {
    fn name(&self) -> &'static str {
        "microvm"
    }

    fn resolve(&self, rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>) {
        let mut argv = vec![
            "run".to_string(),
            "--rm".to_string(),
            "-v".to_string(),
            format!("{}:/workspace", rootfs.display()),
            "-w".to_string(),
            format!("/workspace/{}", self.workdir.display()),
        ];

        // Add environment variables in deterministic (BTreeMap) order.
        for (key, val) in &self.env {
            argv.push("-e".to_string());
            argv.push(format!("{}={}", key, val));
        }

        // Add network policy if Deny egress.
        if self.egress == EgressPolicy::Deny {
            argv.push("--network".to_string());
            argv.push("none".to_string());
        }
        // Open egress: omit --network flag (default behavior).

        // Add image, program, and arguments.
        argv.push(self.image.clone());
        argv.push(program.to_string());
        argv.extend(args.iter().cloned());

        ("container".to_string(), argv)
    }
}

/// Check if the `container` CLI is available on PATH.
pub fn available() -> bool {
    let path = match std::env::var_os("PATH") {
        Some(p) => p,
        None => return false,
    };
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join("container");
        if candidate.is_file() {
            return true;
        }
    }
    false
}

/// Observe the Apple Container BuildKit singleton without changing its
/// lifecycle. The CLI has no VAT/project ownership field, so every result is
/// intentionally advisory and shared. A failed or malformed observation never
/// falls through to an action command.
pub fn builder_advisory() -> AppleContainerBuilderAdvisory {
    let mut advisory = empty_builder_advisory("unavailable");
    if !available() {
        advisory.probe_errors.push(AppleContainerProbeError {
            probe: "builder_status".to_string(),
            message: "Apple Container CLI `container` is not on PATH".to_string(),
        });
        return advisory;
    }

    let builder = match command_stdout_within(
        "container",
        &["builder", "status", "--format", "json"],
        BUILDER_ADVISORY_TIMEOUT,
    )
    .and_then(|output| parse_builder_status(&output))
    {
        Ok(Some(builder)) => builder,
        Ok(None) => {
            advisory.state = "not_running".to_string();
            return advisory;
        }
        Err(error) => {
            advisory.state = "unknown".to_string();
            advisory.probe_errors.push(AppleContainerProbeError {
                probe: "builder_status".to_string(),
                message: error,
            });
            return advisory;
        }
    };

    let builder_id = builder.configuration.id.clone();
    advisory.state = builder.state;
    advisory.configuration = Some(builder.configuration);

    if advisory.state.eq_ignore_ascii_case("running") {
        match command_stdout_within(
            "container",
            &["stats", &builder_id, "--no-stream", "--format", "json"],
            BUILDER_ADVISORY_TIMEOUT,
        )
        .and_then(|output| parse_builder_stats(&output, &builder_id))
        {
            Ok(Some(stats)) => advisory.observed_stats = Some(stats),
            Ok(None) => {}
            Err(error) => advisory.probe_errors.push(AppleContainerProbeError {
                probe: "stats".to_string(),
                message: error,
            }),
        }
    }

    match command_stdout_within(
        "container",
        &["system", "df", "--format", "json"],
        BUILDER_ADVISORY_TIMEOUT,
    )
    .and_then(|output| parse_global_disk(&output))
    {
        Ok(disk) => advisory.global_disk = Some(disk),
        Err(error) => advisory.probe_errors.push(AppleContainerProbeError {
            probe: "system_df".to_string(),
            message: error,
        }),
    }

    advisory
}

fn empty_builder_advisory(state: &str) -> AppleContainerBuilderAdvisory {
    AppleContainerBuilderAdvisory {
        state: state.to_string(),
        ownership: "shared_unknown".to_string(),
        automatic_cleanup: false,
        configuration: None,
        observed_stats: None,
        global_disk: None,
        probe_errors: Vec::new(),
    }
}

struct ParsedBuilderStatus {
    state: String,
    configuration: AppleContainerBuilderConfiguration,
}

fn parse_builder_status(output: &str) -> Result<Option<ParsedBuilderStatus>, String> {
    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|error| format!("invalid JSON from `container builder status`: {error}"))?;
    let entries = json_entries(&value, "container builder status")?;
    if entries.is_empty() {
        return Ok(None);
    }
    let entry = entries
        .iter()
        .copied()
        .find(|entry| {
            entry
                .get("configuration")
                .and_then(|configuration| configuration.get("id"))
                .and_then(serde_json::Value::as_str)
                == Some("buildkit")
        })
        .unwrap_or(entries[0]);
    let configuration = entry
        .get("configuration")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "builder status missing object `configuration`".to_string())?;
    let id = configuration
        .get("id")
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| "builder status missing `configuration.id`".to_string())?;
    let resources = configuration
        .get("resources")
        .and_then(serde_json::Value::as_object);
    let state = entry
        .get("status")
        .and_then(|status| status.get("state"))
        .and_then(serde_json::Value::as_str)
        .filter(|state| !state.is_empty())
        .unwrap_or("unknown");

    Ok(Some(ParsedBuilderStatus {
        state: state.to_string(),
        configuration: AppleContainerBuilderConfiguration {
            id: id.to_string(),
            resources: AppleContainerBuilderResources {
                cpus: resources
                    .and_then(|resources| resources.get("cpus"))
                    .and_then(serde_json::Value::as_f64),
                memory_bytes: resources
                    .and_then(|resources| resources.get("memoryInBytes"))
                    .and_then(serde_json::Value::as_u64),
            },
        },
    }))
}

fn parse_builder_stats(
    output: &str,
    builder_id: &str,
) -> Result<Option<AppleContainerBuilderObservedStats>, String> {
    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|error| format!("invalid JSON from `container stats`: {error}"))?;
    let entries = json_entries(&value, "container stats")?;
    let Some(entry) = entries
        .iter()
        .copied()
        .find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some(builder_id))
    else {
        return Ok(None);
    };
    let stats = AppleContainerBuilderObservedStats {
        memory_usage_bytes: entry
            .get("memoryUsageBytes")
            .and_then(serde_json::Value::as_u64),
        memory_limit_bytes: entry
            .get("memoryLimitBytes")
            .and_then(serde_json::Value::as_u64),
        cpu_usage_usec: entry
            .get("cpuUsageUsec")
            .and_then(serde_json::Value::as_u64),
        process_count: entry
            .get("numProcesses")
            .and_then(serde_json::Value::as_u64),
    };
    if stats.memory_usage_bytes.is_none()
        && stats.memory_limit_bytes.is_none()
        && stats.cpu_usage_usec.is_none()
        && stats.process_count.is_none()
    {
        return Ok(None);
    }
    Ok(Some(stats))
}

fn parse_global_disk(output: &str) -> Result<AppleContainerGlobalDiskAdvisory, String> {
    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|error| format!("invalid JSON from `container system df`: {error}"))?;
    let root = value
        .as_object()
        .ok_or_else(|| "`container system df` returned a non-object JSON value".to_string())?;
    let containers = disk_usage(root.get("containers"));
    let images = disk_usage(root.get("images"));
    if containers.is_none() && images.is_none() {
        return Err("`container system df` omitted containers and images summaries".to_string());
    }
    Ok(AppleContainerGlobalDiskAdvisory {
        scope: "global_apple_container".to_string(),
        containers,
        images,
    })
}

fn disk_usage(value: Option<&serde_json::Value>) -> Option<AppleContainerDiskUsage> {
    let value = value?.as_object()?;
    Some(AppleContainerDiskUsage {
        total: value.get("total").and_then(serde_json::Value::as_u64),
        active: value.get("active").and_then(serde_json::Value::as_u64),
        size_bytes: value.get("sizeInBytes").and_then(serde_json::Value::as_u64),
        reclaimable_bytes: value.get("reclaimable").and_then(serde_json::Value::as_u64),
    })
}

fn json_entries<'a>(
    value: &'a serde_json::Value,
    command: &str,
) -> Result<Vec<&'a serde_json::Value>, String> {
    match value {
        serde_json::Value::Array(entries) => Ok(entries.iter().collect()),
        serde_json::Value::Object(_) => Ok(vec![value]),
        _ => Err(format!(
            "`{command}` returned JSON that was not an array or object"
        )),
    }
}

/// Check if the container system is up and responsive via `container system status`.
/// Returns true only if the probe succeeds within a bounded timeout.
pub fn system_up() -> bool {
    command_succeeds_within("container", &["system", "status"], Duration::from_secs(1))
}

/// Spawn a runtime probe without allowing an unresponsive CLI to bypass the
/// caller's readiness deadline. The child has no inherited output pipes, so a
/// hung descendant cannot keep this failure path open after it is killed.
fn command_succeeds_within(program: &str, args: &[&str], timeout: Duration) -> bool {
    use std::process::{Command, Stdio};
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };
    matches!(wait_for_child(&mut child, timeout, program), Ok(status) if status.success())
}

/// Capture a short command result with the same kill-and-reap timeout protocol
/// used by the runtime readiness probe. This is deliberately private: callers
/// receive advisory errors rather than process handles or lifecycle control.
fn command_stdout_within(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<String, String> {
    use std::process::{Command, Stdio};
    let command = format!("{program} {}", args.join(" "));
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("spawn `{command}`: {error}"))?;
    let status = wait_for_child(&mut child, timeout, &command)?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("wait `{command}`: {error}"))?;
    if status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err(format!("`{command}` exited with {status}"))
    } else {
        Err(format!("`{command}` failed: {stderr}"))
    }
}

fn wait_for_child(
    child: &mut std::process::Child,
    timeout: Duration,
    command: &str,
) -> Result<std::process::ExitStatus, String> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "`{command}` timed out after {}ms",
                    timeout.as_millis()
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("wait `{command}`: {error}"));
            }
        }
    }
}

/// Poll the container system until it responds or timeout elapses.
/// Returns Err with a message naming the elapsed timeout when unavailable.
pub fn ensure_system_started(timeout: std::time::Duration) -> Result<(), String> {
    poll_until_up(timeout, system_up)
}

/// Poll+timeout loop shared by `ensure_system_started`, parameterized over the probe
/// so the timeout behavior itself is testable without depending on whether the real
/// `container` CLI is installed or its system is actually running on the test host
/// (R4: deterministic on every host, never hangs indefinitely).
fn poll_until_up(timeout: std::time::Duration, probe: impl Fn() -> bool) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        if probe() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "container system did not respond within {:?}",
                timeout
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Sandbox;
    use std::path::PathBuf;

    fn test_backend() -> MicroVmBackend {
        let mut env = BTreeMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("BAZ".to_string(), "qux".to_string());

        MicroVmBackend {
            egress: EgressPolicy::Open,
            env,
            workdir: PathBuf::from("."),
            image: "ubuntu:latest".to_string(),
        }
    }

    #[test]
    fn resolve_builds_rootfs_mount_and_workdir() {
        let backend = test_backend();
        let rootfs = PathBuf::from("/tmp/rootfs");
        let (cmd, argv) = backend.resolve(&rootfs, "echo", &["hello".to_string()]);

        assert_eq!(cmd, "container");
        assert!(argv.len() >= 10);
        assert_eq!(argv[0], "run");
        assert_eq!(argv[1], "--rm");
        assert_eq!(argv[2], "-v");
        assert!(argv[3].contains("/tmp/rootfs") && argv[3].contains(":/workspace"));
        assert_eq!(argv[4], "-w");
        assert!(argv[5].contains("/workspace"));
    }

    #[test]
    fn resolve_env_flags_are_btreemap_ordered() {
        let backend = test_backend();
        let rootfs = PathBuf::from("/tmp/rootfs");
        let (_, argv) = backend.resolve(&rootfs, "sh", &[]);

        // Find all -e positions and check ordering.
        let env_pairs: Vec<(usize, &String)> = argv
            .iter()
            .enumerate()
            .filter_map(|(i, x)| {
                if x == "-e" {
                    Some((i, &argv[i + 1]))
                } else {
                    None
                }
            })
            .collect();

        // Should have two -e pairs: BAZ=qux comes before FOO=bar (alphabetical BTreeMap order).
        assert_eq!(env_pairs.len(), 2);
        assert!(env_pairs[0].1.starts_with("BAZ"));
        assert!(env_pairs[1].1.starts_with("FOO"));
    }

    #[test]
    fn resolve_open_egress_omits_network_flag() {
        let backend = MicroVmBackend {
            egress: EgressPolicy::Open,
            env: BTreeMap::new(),
            workdir: PathBuf::from("."),
            image: "ubuntu:latest".to_string(),
        };
        let (_, argv) = backend.resolve(&PathBuf::from("/tmp/rootfs"), "sh", &[]);

        assert!(!argv.contains(&"--network".to_string()));
    }

    #[test]
    fn resolve_deny_egress_sets_network_none() {
        let backend = MicroVmBackend {
            egress: EgressPolicy::Deny,
            env: BTreeMap::new(),
            workdir: PathBuf::from("."),
            image: "ubuntu:latest".to_string(),
        };
        let (_, argv) = backend.resolve(&PathBuf::from("/tmp/rootfs"), "sh", &[]);

        let network_idx = argv.iter().position(|x| x == "--network");
        assert!(network_idx.is_some());
        assert_eq!(argv[network_idx.unwrap() + 1], "none");
    }

    #[test]
    fn resolve_argv_tail_is_image_then_program_then_args() {
        let backend = MicroVmBackend {
            egress: EgressPolicy::Open,
            env: BTreeMap::new(),
            workdir: PathBuf::from("."),
            image: "alpine:latest".to_string(),
        };
        let (_, argv) = backend.resolve(
            &PathBuf::from("/tmp/rootfs"),
            "python",
            &[
                "train.py".to_string(),
                "--batch".to_string(),
                "32".to_string(),
            ],
        );

        let tail = &argv[argv.len() - 5..];
        assert_eq!(tail[0], "alpine:latest");
        assert_eq!(tail[1], "python");
        assert_eq!(tail[2], "train.py");
        assert_eq!(tail[3], "--batch");
        assert_eq!(tail[4], "32");
    }

    #[test]
    fn ensure_system_started_times_out_when_unavailable() {
        // R4: probe that never reports up must yield Err once the bounded timeout
        // elapses, deterministically on every host — this drives poll_until_up
        // directly with an always-false probe so the assertion never depends on
        // whether the real `container` CLI happens to be installed and running.
        let result = poll_until_up(std::time::Duration::from_millis(50), || false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("did not respond within"));
    }

    #[cfg(unix)]
    #[test]
    fn bounded_runtime_probe_kills_a_hung_status_command() {
        let started = Instant::now();
        assert!(
            !command_succeeds_within(
                "/bin/sh",
                &["-c", "exec /bin/sleep 5"],
                Duration::from_millis(50),
            ),
            "a hung runtime probe must fail rather than block readiness"
        );
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "bounded runtime probe exceeded its timeout"
        );
    }
}
// CODEGEN-END
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VatBuildPhase2DataModelAdditions {
    /// New apps/vat/src/commands/build.rs struct Args: file: Option<PathBuf> (defaults to `Dockerfile` inside the resolved context dir), context: Option<PathBuf> (defaults to the current directory), tag: Option<String> (defaults to `<context-dir-basename>:latest`, sanitized to a valid OCI reference — lowercased, non [a-z0-9._-] runs collapsed to `-` — resolved once in exec() before any subprocess is spawned; build_image() itself never guesses a tag, it always receives a concrete &str), build_args: Vec<(String,String)> (one pair per repeated --build-arg K=V flag, parsed via split_once('='), CLI-supplied order preserved — no BTreeMap reordering needed here since the input is already a deterministic Vec, unlike Phase 1's EnvSpec.env map), json: bool.
    #[serde(default)]
    pub build_args_struct: Option<serde_json::Value>,
    /// New apps/vat/src/commands/build.rs struct BuildReport (derives serde::Serialize): tag: String, dockerfile: String (resolved absolute path), context: String (resolved absolute path), build_args: BTreeMap<String,String> (sorted for deterministic JSON field ordering in the report, independent of the argv-ordering rule above), duration_ms: u64. Constructed only on a successful build — build_image()'s Result<BuildReport> Err variant covers every failure path (missing Dockerfile, container CLI/system unavailable, nonzero container build exit); there is no success:false variant.
    #[serde(default)]
    pub build_report_struct: Option<serde_json::Value>,
    /// New apps/vat/src/commands/build.rs fn container_build_command(dockerfile: &Path, tag: &str, build_args: &[(String,String)], context: &Path) -> Vec<String>. Pure, deterministic argv builder (no subprocess, no I/O) producing exactly: ["container", "build", "-f", <dockerfile>, "-t", <tag>, "--build-arg", "K=V", ... one --build-arg pair per entry in the given slice order ..., <context>] (R2), matching the real invocation Phase 0 verified (`container build -f "$WORKDIR/Dockerfile" -t vat-spike-test:latest "$WORKDIR"`). Unlike sandbox/microvm.rs's resolve() (which returns a (program, argv) tuple), this fn returns the program name ("container") as argv[0] itself.
    #[serde(default)]
    pub container_build_command_fn: Option<serde_json::Value>,
    /// New apps/vat/src/commands/build.rs fn build_image(context: &Path, dockerfile: &Path, tag: &str, build_args: &[(String,String)]) -> Result<BuildReport> — the in-process entry point Phase 3's `vat compose` will call directly for compose `build:` keys (not a shell-out to the vat binary). Validates the dockerfile path exists (AC3, no subprocess on failure), calls ensure_microvm_available(), builds argv via container_build_command(), spawns `container` with captured stdout/stderr (never inherited — the always-captured behavior a reusable in-process caller like compose needs), waits for exit, and returns Err on a nonzero exit or Ok(BuildReport) on success.
    #[serde(default)]
    pub build_image_fn: Option<serde_json::Value>,
    /// New apps/vat/src/commands/build.rs fn ensure_microvm_available() -> Result<()>, mirroring run.rs's ensure_docker_available: requires sandbox::microvm::available() (container binary on PATH); if sandbox::microvm::system_up() is not immediately true, waits via sandbox::microvm::ensure_system_started(<bounded timeout>) before failing. Fail-closed and clean — never auto-installs the container CLI, never silently proceeds without a responsive system.
    #[serde(default)]
    pub ensure_microvm_available_fn: Option<serde_json::Value>,
    /// New additive apps/vat/src/sandbox/microvm.rs fn: pub fn system_up() -> bool. Bounded-timeout `container system status` probe (spawn with stdout/stderr to null, short internal timeout, return status.success()), mirroring run.rs's docker_daemon_up(). Additive only — no change to MicroVmBackend, resolve(), or available() (Phase 1, untouched).
    #[serde(default)]
    pub microvm_system_up_fn: Option<serde_json::Value>,
    /// New additive apps/vat/src/sandbox/microvm.rs fn: pub fn ensure_system_started(timeout: Duration) -> Result<(), String>. Poll+timeout loop (Instant::now() deadline, short sleep between polls of system_up()) mirroring cluster.rs's run_capture() poll pattern; returns Err naming the elapsed timeout when the container system never reports up within the bound. Additive only, same non-interference guarantee as system_up() above.
    #[serde(default)]
    pub microvm_ensure_system_started_fn: Option<serde_json::Value>,
    /// apps/vat/Cargo.toml: serde_yaml (version 0.9, unchanged) loses `optional = true` and becomes an unconditional dependency; removed from the `emulator` feature's `dep:serde_yaml` list. Zero new crate, needed unconditionally ahead of Phase 3's compose YAML parsing (not used by this WI's own code).
    #[serde(default)]
    pub cargo_serde_yaml_promotion: Option<serde_json::Value>,
}
// CODEGEN-END
