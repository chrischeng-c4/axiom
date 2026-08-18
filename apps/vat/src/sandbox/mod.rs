// CODEGEN-BEGIN
//! Pluggable isolation backends.
//!
//! The differentiator of vat is the state layer, not the isolation mechanism —
//! so isolation is a trait with swappable implementations. v1 ships:
//!
//! - [`process::ProcessBackend`] — run the command as a plain host process
//!   confined to the rootfs as its working directory. Zero friction, full
//!   native GPU/IO. The default.
//! - [`seatbelt::SeatbeltBackend`] — wrap the command in a macOS seatbelt
//!   profile (`sandbox-exec`) that confines writes to the rootfs while leaving
//!   the Metal GPU reachable (it's still a host process).
//!
//! A future Linux backend will add a namespaces + overlayfs implementation
//! behind this same trait; the VM path (Virtualization.framework) would slot
//! in here too — at the cost of the GPU story, which is the whole point of
//! *not* taking that path on Apple Silicon.

pub mod microvm;
pub mod process;
pub mod seatbelt;

use std::path::Path;

use crate::spec::{EgressPolicy, EnvSpec, Isolation};

/// An isolation backend resolves the user's command into the *actual* program
/// + argv to exec (e.g. seatbelt wraps it in `sandbox-exec`). The caller then
/// runs that resolved command inside the vat workspace with the spec env.
pub trait Sandbox {
    /// Short stable name, surfaced in events/state (`"process"`, `"seatbelt"`).
    fn name(&self) -> &'static str;

    /// Resolve `(program, args)` to the program + argv actually exec'd.
    /// `rootfs` is the vat's copy-on-write workspace (seatbelt scopes writes
    /// to it).
    fn resolve(&self, rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>);
}

/// Pick a backend for a spec. Fails closed: if the selected backend cannot
/// actually enforce a non-`Open` egress policy, this returns `Err` instead of
/// silently downgrading to unrestricted network access. `Isolation::None` +
/// `EgressPolicy::Open` (today's common case) is unaffected and always
/// succeeds; the workspace clone still applies regardless of isolation, so a
/// vat is never *less* isolated than plain `cd` + run on that front.
// HANDWRITE-BEGIN gap="missing-generator:logic:pick-fail-closed" tracker="#1300" reason="Logic section edge: pick() must fail closed (return Err) instead of warn-and-continue when the selected backend cannot enforce a non-Open egress policy (isolation=none, or seatbelt requested but unavailable) — hand-written backend-selection logic per issue #1300."
pub fn pick(spec: &EnvSpec) -> Result<Box<dyn Sandbox>, String> {
    match spec.isolation {
        Isolation::None => {
            if spec.egress != EgressPolicy::Open {
                return Err(format!(
                    "[network].egress is set to {:?}, but --isolation none cannot enforce it \
                     (no sandbox backend confines egress); use --isolation seatbelt or set \
                     egress to open.",
                    spec.egress
                ));
            }
            Ok(Box::new(process::ProcessBackend))
        }
        Isolation::Seatbelt => {
            if cfg!(target_os = "macos") && seatbelt::available() {
                Ok(Box::new(seatbelt::SeatbeltBackend {
                    egress: spec.egress,
                }))
            } else if spec.egress != EgressPolicy::Open {
                Err(format!(
                    "--isolation seatbelt was requested with [network].egress set to {:?}, \
                     but sandbox-exec is unavailable on this host; falling back to the process \
                     backend would silently drop egress enforcement, so refusing to run.",
                    spec.egress
                ))
            } else {
                eprintln!(
                    "vat: seatbelt isolation requested but unavailable on this host; \
                     using process backend (workspace is still copy-on-write)."
                );
                Ok(Box::new(process::ProcessBackend))
            }
        }
        Isolation::MicroVm => {
            // MicroVm requires a base image and fails closed on unsupported combinations.
            if spec.gpu == crate::spec::GpuRequest::Required {
                return Err(
                    "isolation=micro_vm cannot satisfy --gpu required: GPU passthrough is \
                     categorically impossible in an Apple Silicon microVM (Virtualization.framework \
                     architecture constraint, not a vat limitation)."
                        .to_string(),
                );
            }
            let Some(image) = spec.microvm_image.clone() else {
                return Err(
                    "isolation=micro_vm requires an OCI base image (--microvm-image <ref>); \
                     vat does not guess one."
                        .to_string(),
                );
            };
            if !microvm::available() {
                return Err(
                    "isolation=micro_vm requested but `container` CLI is not installed; \
                     install it and re-run `vat doctor`."
                        .to_string(),
                );
            }
            match spec.egress {
                EgressPolicy::Open | EgressPolicy::Deny => Ok(Box::new(microvm::MicroVmBackend {
                    egress: spec.egress,
                    env: spec.env.clone(),
                    workdir: spec.workdir.clone(),
                    image,
                })),
                EgressPolicy::LocalhostOnly => Err(
                    "isolation=micro_vm cannot yet enforce egress=localhost-only: the guest \
                     127.0.0.1 never reaches the host; the host is only reachable via a \
                     per-network container VM gateway IP that ordinary applications do not know \
                     to target (confirmed by the Phase 0 spike #1472). Use --isolation seatbelt, \
                     or switch --network to open or deny."
                        .to_string(),
                ),
            }
        }
    }
}
// HANDWRITE-END
// CODEGEN-END
