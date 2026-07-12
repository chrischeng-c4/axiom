// HANDWRITE-BEGIN gap="missing-generator:e2e-test:950cee14" tracker="pending-tracker" reason="AC3/AC4: new fail-closed integration test file covering GpuRequest::Required, missing microvm_image, EgressPolicy::LocalhostOnly, and container-unavailable pick() rejections, plus the dedicated run.rs gpu_satisfied() preflight rejection case proving the second independent fail-closed layer (R4)."

//! Integration tests for MicroVm sandbox backend fail-closed behavior.
//! AC3: Verify that pick() rejects categorically impossible combinations:
//! - GpuRequest::Required with Isolation::MicroVm
//! - Isolation::MicroVm without spec.microvm_image set
//! - Isolation::MicroVm with EgressPolicy::LocalhostOnly
//! - Isolation::MicroVm when container CLI is not available
//! AC4 (the second, independent fail-closed layer — the run.rs
//! `gpu_satisfied()` preflight helper) is covered by a unit test colocated
//! with that private helper: `commands::run::tests::
//! gpu_satisfied_rejects_microvm_required_before_workspace_clone`.

use vat::sandbox;
use vat::spec::{EgressPolicy, EnvSpec, GpuRequest, Isolation};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn gpu_required_rejected() {
    let spec = EnvSpec {
        base: None,
        workdir: PathBuf::from("."),
        env: BTreeMap::new(),
        setup: Vec::new(),
        isolation: Isolation::MicroVm,
        egress: EgressPolicy::Open,
        gpu: GpuRequest::Required,
        limits: Default::default(),
        microvm_image: Some("ubuntu:latest".to_string()),
    };

    let result = sandbox::pick(&spec);
    assert!(result.is_err(), "pick() should reject GPU required with MicroVm");
    match result {
        Err(err) => {
            assert!(err.contains("micro_vm"), "error must mention isolation mode: {}", err);
            assert!(err.contains("gpu"), "error must mention GPU: {}", err);
        }
        Ok(_) => panic!("pick() should have returned an error"),
    }
}

#[test]
fn missing_image_rejected() {
    let spec = EnvSpec {
        base: None,
        workdir: PathBuf::from("."),
        env: BTreeMap::new(),
        setup: Vec::new(),
        isolation: Isolation::MicroVm,
        egress: EgressPolicy::Open,
        gpu: GpuRequest::None,
        limits: Default::default(),
        microvm_image: None,
    };

    let result = sandbox::pick(&spec);
    assert!(result.is_err(), "pick() should reject missing microvm_image");
    match result {
        Err(err) => {
            assert!(
                err.contains("microvm_image") || err.contains("OCI"),
                "error must mention missing image: {}",
                err
            );
        }
        Ok(_) => panic!("pick() should have returned an error"),
    }
}

#[test]
fn localhost_only_rejected_with_gateway_reasoning() {
    let spec = EnvSpec {
        base: None,
        workdir: PathBuf::from("."),
        env: BTreeMap::new(),
        setup: Vec::new(),
        isolation: Isolation::MicroVm,
        egress: EgressPolicy::LocalhostOnly,
        gpu: GpuRequest::None,
        limits: Default::default(),
        microvm_image: Some("ubuntu:latest".to_string()),
    };

    let result = sandbox::pick(&spec);
    assert!(result.is_err(), "pick() should reject LocalhostOnly with MicroVm");
    match result {
        Err(err) => {
            // Per Phase 0 spike #1472: gateway IP messaging, not generic "no bridge"
            assert!(
                err.contains("gateway") || err.contains("127.0.0.1"),
                "error must explain guest 127.0.0.1 and gateway IP mechanism: {}",
                err
            );
        }
        Ok(_) => panic!("pick() should have returned an error"),
    }
}

#[test]
fn container_unavailable_rejected() {
    // This test only runs meaningfully when container is actually not installed.
    // When container IS installed, this becomes a no-op success.
    if sandbox::microvm::available() {
        eprintln!("container CLI is available; skipping container-unavailable test");
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
        microvm_image: Some("ubuntu:latest".to_string()),
    };

    let result = sandbox::pick(&spec);
    assert!(result.is_err(), "pick() should reject when container unavailable");
    match result {
        Err(err) => {
            assert!(
                err.contains("container") || err.contains("installed"),
                "error must mention container CLI: {}",
                err
            );
        }
        Ok(_) => panic!("pick() should have returned an error"),
    }
}

// HANDWRITE-END
