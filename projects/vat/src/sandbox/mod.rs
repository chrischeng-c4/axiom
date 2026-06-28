// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-mod-rs.md#rust-source-unit
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

pub mod process;
pub mod seatbelt;

use std::path::Path;

use crate::spec::{EgressPolicy, EnvSpec, Isolation};

/// An isolation backend resolves the user's command into the *actual* program
/// + argv to exec (e.g. seatbelt wraps it in `sandbox-exec`). The caller then
/// runs that resolved command inside the vat workspace with the spec env.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-mod-rs.md#source
pub trait Sandbox {
    /// Short stable name, surfaced in events/state (`"process"`, `"seatbelt"`).
    fn name(&self) -> &'static str;

    /// Resolve `(program, args)` to the program + argv actually exec'd.
    /// `rootfs` is the vat's copy-on-write workspace (seatbelt scopes writes
    /// to it).
    fn resolve(&self, rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>);
}

/// Pick a backend for a spec. Falls back to the process backend on any
/// platform that doesn't support the requested isolation, after warning —
/// the workspace clone still applies, so the vat is never *less* isolated than
/// plain `cd` + run.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-mod-rs.md#source
pub fn pick(spec: &EnvSpec) -> Box<dyn Sandbox> {
    match spec.isolation {
        Isolation::None => {
            if spec.egress != EgressPolicy::Open {
                eprintln!(
                    "vat: warning: [network].egress confinement requires --isolation seatbelt; \
                     running without egress enforcement."
                );
            }
            Box::new(process::ProcessBackend)
        }
        Isolation::Seatbelt => {
            if cfg!(target_os = "macos") && seatbelt::available() {
                Box::new(seatbelt::SeatbeltBackend {
                    egress: spec.egress,
                })
            } else {
                if spec.egress != EgressPolicy::Open {
                    eprintln!(
                        "vat: warning: seatbelt unavailable; [network].egress confinement \
                         is not enforced."
                    );
                }
                eprintln!(
                    "vat: seatbelt isolation requested but unavailable on this host; \
                     using process backend (workspace is still copy-on-write)."
                );
                Box::new(process::ProcessBackend)
            }
        }
    }
}
// CODEGEN-END
