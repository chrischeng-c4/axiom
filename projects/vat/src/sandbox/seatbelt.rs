// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-sandbox.md#schema
// CODEGEN-BEGIN
//! macOS seatbelt backend.
//!
//! Wraps the command in `sandbox-exec` with a generated profile that allows
//! broad reads (so toolchains resolve) but confines **writes** to the vat's
//! rootfs and the system temp dirs. The GPU is untouched: a seatbelt'd process
//! is still a host process, so Metal/MPS/MLX keep working — the contrast with
//! Docker's Linux VM holds even under isolation.
//!
//! `sandbox-exec` is deprecated by Apple but remains functional and is the
//! pragmatic v1 mechanism. A future backend may move to the Endpoint Security
//! / App Sandbox entitlement route; this trait boundary makes that swap local.

use std::path::Path;

use crate::sandbox::Sandbox;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-seatbelt-rs.md#source
pub struct SeatbeltBackend;

/// Is `sandbox-exec` present on this host?
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-seatbelt-rs.md#source
pub fn available() -> bool {
    which("sandbox-exec").is_some()
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-seatbelt-rs.md#source
impl Sandbox for SeatbeltBackend {
    fn name(&self) -> &'static str {
        "seatbelt"
    }

    fn resolve(&self, rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>) {
        // Wrap the command in `sandbox-exec -p <profile> -- <program> <args>`.
        let profile = profile_for(rootfs);
        let mut argv = vec!["-p".to_string(), profile, program.to_string()];
        argv.extend(args.iter().cloned());
        ("sandbox-exec".to_string(), argv)
    }
}

/// Build a seatbelt profile string confining writes to `rootfs` + temp.
fn profile_for(rootfs: &Path) -> String {
    let root = rootfs.display();
    // (allow default) then deny writes, then re-allow writes only under the
    // rootfs subtree and temp. Reads stay open so interpreters/toolchains
    // resolve their libraries.
    format!(
        "(version 1)\n\
         (allow default)\n\
         (deny file-write*)\n\
         (allow file-write* (subpath \"{root}\"))\n\
         (allow file-write* (subpath \"/private/tmp\"))\n\
         (allow file-write* (subpath \"/private/var/folders\"))\n\
         (allow file-write* (subpath \"/tmp\"))\n"
    )
}

/// Minimal PATH lookup (no extra deps).
fn which(bin: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|candidate| candidate.is_file())
}
// CODEGEN-END
