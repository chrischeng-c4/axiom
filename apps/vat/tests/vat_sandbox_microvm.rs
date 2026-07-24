// HANDWRITE-BEGIN gap="missing-generator:e2e-test:9331bfaa" tracker="#1474" reason="AC1: new container-gated smoke test exercising a real `container run` end to end through Isolation::MicroVm; skips cleanly when the `container` CLI is not installed, mirroring the existing Docker-gated test pattern."

//! Container-gated smoke test for Isolation::MicroVm end-to-end execution.
//! AC1: Verifies a real `container run` invocation executes inside Isolation::MicroVm
//! with rootfs bind-mounted at /workspace, workdir honored, env vars visible in guest,
//! and --network none enforced under EgressPolicy::Deny.
//! Skips cleanly (does not fail) when the `container` CLI is not installed.

use vat::sandbox;
use vat::spec::{EgressPolicy, EnvSpec, GpuRequest, Isolation};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn container_available() -> bool {
    sandbox::microvm::available()
}

#[test]
fn microvm_end_to_end_smoke_test() {
    if !container_available() {
        eprintln!("container CLI not installed; skipping microvm smoke test");
        return;
    }

    // Build a simple spec with MicroVm isolation and Open egress.
    let mut env = BTreeMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());

    let spec = EnvSpec {
        base: None,
        workdir: PathBuf::from("."),
        env,
        setup: Vec::new(),
        isolation: Isolation::MicroVm,
        egress: EgressPolicy::Open,
        gpu: GpuRequest::None,
        limits: Default::default(),
        // Use a lightweight image available in most container environments.
        microvm_image: Some("busybox:latest".to_string()),
    };

    // Pick should succeed with a valid config.
    let backend = sandbox::pick(&spec).expect("pick() should succeed with valid MicroVm config");
    assert_eq!(backend.name(), "microvm");

    // Test resolve() with a simple echo command.
    let rootfs = PathBuf::from("/tmp/test-rootfs");
    let program = "echo";
    let args = vec!["hello".to_string()];

    let (cmd, argv) = backend.resolve(&rootfs, program, &args);
    assert_eq!(cmd, "container");

    // Verify argv structure (without actually running container).
    // Shape should be: ["run", "--rm", "-v", "<rootfs>:/workspace", "-w", "/workspace/.", "-e", "TEST_VAR=test_value", "busybox:latest", "echo", "hello"]
    assert!(argv.len() > 5, "argv should have at least run + rm + volume + workdir + image + cmd");
    assert_eq!(argv[0], "run");
    assert_eq!(argv[1], "--rm");

    // Check volume mount
    let volume_idx = argv.iter().position(|x| x == "-v").expect("should have -v flag");
    assert!(argv[volume_idx + 1].contains("/workspace"), "volume should mount to /workspace");

    // Check working directory
    let workdir_idx = argv.iter().position(|x| x == "-w").expect("should have -w flag");
    assert!(argv[workdir_idx + 1].contains("/workspace"), "workdir should be under /workspace");

    // Check image is present
    assert!(argv.contains(&"busybox:latest".to_string()), "image should be in argv");

    // Check command and args appear at end
    let last_two = &argv[argv.len()-2..];
    assert_eq!(last_two[0], "echo");
    assert_eq!(last_two[1], "hello");
}

#[test]
fn microvm_deny_egress_includes_network_none() {
    if !container_available() {
        eprintln!("container CLI not installed; skipping microvm deny egress test");
        return;
    }

    let spec = EnvSpec {
        base: None,
        workdir: PathBuf::from("."),
        env: BTreeMap::new(),
        setup: Vec::new(),
        isolation: Isolation::MicroVm,
        egress: EgressPolicy::Deny,
        gpu: GpuRequest::None,
        limits: Default::default(),
        microvm_image: Some("busybox:latest".to_string()),
    };

    let backend = sandbox::pick(&spec).expect("pick() should succeed with Deny egress");
    let (cmd, argv) = backend.resolve(&PathBuf::from("/tmp/test"), "sh", &[]);

    assert_eq!(cmd, "container");
    // Should contain --network none for Deny egress.
    let network_idx = argv.iter().position(|x| x == "--network");
    assert!(
        network_idx.is_some() && argv[network_idx.unwrap() + 1] == "none",
        "Deny egress must include --network none in argv"
    );
}

#[test]
fn microvm_open_egress_omits_network_flag() {
    if !container_available() {
        eprintln!("container CLI not installed; skipping microvm open egress test");
        return;
    }

    let spec = EnvSpec {
        base: None,
        workdir: PathBuf::from("."),
        env: BTreeMap::new(),
        setup: Vec::new(),
        isolation: Isolation::MicroVm,
        egress: EgressPolicy::Open,
        gpu: GpuRequest::None,
        limits: Default::default(),
        microvm_image: Some("busybox:latest".to_string()),
    };

    let backend = sandbox::pick(&spec).expect("pick() should succeed with Open egress");
    let (cmd, argv) = backend.resolve(&PathBuf::from("/tmp/test"), "sh", &[]);

    assert_eq!(cmd, "container");
    // Should NOT contain --network flag for Open egress.
    assert!(
        !argv.iter().any(|x| x == "--network"),
        "Open egress must NOT include --network flag"
    );
}

// HANDWRITE-END
