// SPEC-MANAGED: apps/vat/tech-design/logic/vat-microvm-phase-1-isolation-microvm-sandbox-backend-for-vat-ru.md#schema
// CODEGEN-BEGIN
//! MicroVM sandbox backend for Apple Silicon via the `container` CLI.

use std::collections::BTreeMap;
use std::path::Path;

use crate::spec::EgressPolicy;

/// MicroVmBackend runs a workload inside an ephemeral Apple `container` microVM.
/// @spec apps/vat/tech-design/logic/vat-microvm-phase-1-isolation-microvm-sandbox-backend-for-vat-ru.md#schema
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
            .filter_map(|(i, x)| if x == "-e" { Some((i, &argv[i + 1])) } else { None })
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
        let (_, argv) = backend.resolve(&PathBuf::from("/tmp/rootfs"), "python", &[
            "train.py".to_string(),
            "--batch".to_string(),
            "32".to_string(),
        ]);

        let tail = &argv[argv.len() - 5..];
        assert_eq!(tail[0], "alpine:latest");
        assert_eq!(tail[1], "python");
        assert_eq!(tail[2], "train.py");
        assert_eq!(tail[3], "--batch");
        assert_eq!(tail[4], "32");
    }
}
// CODEGEN-END
// SPEC-MANAGED: apps/vat/tech-design/interfaces/cli/vat-microvm-phase-2-vat-build-dockerfile-build-via-container-cli.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// @spec apps/vat/tech-design/interfaces/cli/vat-microvm-phase-2-vat-build-dockerfile-build-via-container-cli.md#schema
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
