// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-src-sandbox-mod-rs.md#rust-source-unit
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
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-sandbox-mod-rs.md#source
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
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-sandbox-mod-rs.md#source
// HANDWRITE-BEGIN gap="missing-generator:logic:pick-fail-closed" tracker="pending-tracker" reason="Logic section edge: pick() must fail closed (return Err) instead of warn-and-continue when the selected backend cannot enforce a non-Open egress policy (isolation=none, or seatbelt requested but unavailable) — hand-written backend-selection logic per issue #1300."
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
    }
}
// HANDWRITE-END
// CODEGEN-END
