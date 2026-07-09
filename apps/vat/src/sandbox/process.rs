// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-src-sandbox-process-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Host-process backend.
//!
//! The default and simplest sandbox: the command runs as an ordinary macOS (or
//! Linux) process whose working directory is the vat's copy-on-write rootfs.
//! There is no syscall confinement here — that is intentional. It keeps the
//! workload fully native, which is exactly why the Apple GPU is reachable
//! (nothing is virtualized). Disposability comes from the COW workspace:
//! whatever the command writes lands in the rootfs and can be diffed,
//! snapshotted, forked, or thrown away.
//!
//! `ProcessBackend` cannot enforce any [network].egress policy (there is no
//! syscall confinement here at all) — see issue #1300. Because of that,
//! `sandbox::pick` is the *only* place that is allowed to construct this
//! backend, and it does so only after checking that the run's egress policy
//! is `Open`; a non-`Open` egress policy makes `pick` fail closed with an
//! error instead of silently handing back a `ProcessBackend` that would run
//! with unrestricted network access. `resolve` below stays a pure passthrough
//! on purpose — it has no isolation/egress decision to make.

use std::path::Path;

use crate::sandbox::Sandbox;

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-sandbox-process-rs.md#source
// HANDWRITE-BEGIN gap="missing-generator:logic:process-backend-no-egress-enforcement" tracker="pending-tracker" reason="Logic section edge: document that ProcessBackend has no egress-enforcement capability and that callers must go through sandbox::pick, which now fails closed instead of warning, per issue #1300."
pub struct ProcessBackend;

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-sandbox-process-rs.md#source
impl Sandbox for ProcessBackend {
    fn name(&self) -> &'static str {
        "process"
    }

    fn resolve(&self, _rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>) {
        // Run the command verbatim; cwd/env are applied by the caller.
        // Egress enforcement is *not* this backend's job — see the module doc
        // comment above: `sandbox::pick` gates whether this backend is ever
        // constructed for a non-Open egress policy.
        (program.to_string(), args.to_vec())
    }
}
// HANDWRITE-END
// CODEGEN-END
