// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-process-rs.md#rust-source-unit
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

use std::path::Path;

use crate::sandbox::Sandbox;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-process-rs.md#source
pub struct ProcessBackend;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-process-rs.md#source
impl Sandbox for ProcessBackend {
    fn name(&self) -> &'static str {
        "process"
    }

    fn resolve(&self, _rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>) {
        // Run the command verbatim; cwd/env are applied by the caller.
        (program.to_string(), args.to_vec())
    }
}
// CODEGEN-END
