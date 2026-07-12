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
