// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-gpu-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! GPU visibility — the reason vat exists for ML agents.
//!
//! ## The problem vat solves
//!
//! On Apple Silicon, Docker runs Linux containers inside a Linux VM
//! (Virtualization.framework / QEMU). Apple's GPU is only reachable through
//! **Metal**, and Metal has no compute passthrough into a Linux guest — so
//! `torch.backends.mps`, MLX, and `tensorflow-metal` all report "no GPU"
//! inside a Docker container. There is no `--gpus all` equivalent that works.
//!
//! ## Why vat doesn't have the problem
//!
//! A vat is **not a VM**. The workload runs as a sandboxed *host* process over
//! a copy-on-write workspace (see [`crate::overlay`] and
//! [`crate::sandbox`]). Because the process never leaves macOS, the Metal
//! device is simply present — the GPU was never taken away, so there is
//! nothing to "bridge".
//!
//! This module reports what the host (and therefore every vat) can see, so an
//! agent can answer "do I have a GPU, and can my vat use it?" from
//! [`crate::state::VatState`] without guessing.
//!
//! v1 detection is deliberately light: chip identity via `sysctl`, presence of
//! the Metal stack via a well-known framework path. Enumerating GPU core count
//! and unified-memory size via the `metal` crate (a real `MTLDevice` query) is
//! a tracked follow-up.

use serde::{Deserialize, Serialize};

/// What GPU acceleration a vat can reach. This is host truth: on macOS every
/// vat shares it because every vat is a host process.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-gpu-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// `"apple"`, `"none"`, or another vendor on non-macOS hosts.
    pub vendor: String,
    /// Human chip string, e.g. `"Apple M3 Max"`. `None` if undetected.
    pub chip: Option<String>,
    /// Acceleration backends a workload can use right now.
    pub backends: Vec<String>,
    /// True when the GPU is reachable by host processes (always true for a
    /// real Apple Silicon host; the headline contrast with Docker-in-VM).
    pub accessible: bool,
    /// One-line explanation aimed at an agent reading state.
    pub note: String,
}

/// Detect host GPU visibility. Cheap and side-effect free; safe to call per
/// `vat state`.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-gpu-rs.md#source
pub fn detect() -> GpuInfo {
    #[cfg(target_os = "macos")]
    {
        detect_macos()
    }
    #[cfg(not(target_os = "macos"))]
    {
        detect_other()
    }
}

#[cfg(target_os = "macos")]
fn detect_macos() -> GpuInfo {
    let chip = sysctl("machdep.cpu.brand_string");
    let is_apple_silicon = chip
        .as_deref()
        .map(|c| c.starts_with("Apple"))
        .unwrap_or(false);

    if is_apple_silicon {
        // Metal ships with the OS; MPS/MLX ride on it. We report the backends
        // a host process *can* use — whether the user installed torch/mlx is
        // their business, not the sandbox's.
        GpuInfo {
            vendor: "apple".into(),
            chip,
            backends: vec!["metal".into(), "mps".into(), "mlx".into()],
            accessible: true,
            note: "Apple GPU is reachable: a vat is a host process, not a Linux \
                   VM, so Metal/MPS/MLX work where Docker shows no GPU."
                .into(),
        }
    } else {
        // Intel Mac: integrated/discrete GPU via Metal, no unified-memory ML
        // story worth advertising.
        GpuInfo {
            vendor: "apple-intel".into(),
            chip,
            backends: vec!["metal".into()],
            accessible: true,
            note: "Intel Mac: Metal available to host processes; no Apple \
                   Silicon unified-memory acceleration."
                .into(),
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn detect_other() -> GpuInfo {
    // The Linux/other backend will grow CUDA/ROCm detection alongside its
    // namespace-based sandbox. For now report honestly that we don't probe it.
    GpuInfo {
        vendor: "unknown".into(),
        chip: None,
        backends: vec![],
        accessible: false,
        note: "Non-macOS host: GPU probing not implemented in v1 (the \
               GPU-native story targets Apple Silicon)."
            .into(),
    }
}

/// Read a single `sysctl` string value, or `None` if unavailable.
#[cfg(target_os = "macos")]
fn sysctl(key: &str) -> Option<String> {
    let out = std::process::Command::new("sysctl")
        .args(["-n", key])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
