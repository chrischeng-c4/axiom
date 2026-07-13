// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/projects-lumen-build-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Build script: stamp `LUMEN_GIT_SHA` and `LUMEN_BUILT_AT` into the binary
//! so `GET /version` can report provenance.
//!
//! Both are best-effort: outside a git checkout (e.g. a source tarball) the
//! sha falls back to "unknown", and the handler degrades the same way via
//! `option_env!`. Nothing here fails the build. The actual stamping logic
//! (git short-sha, built-at epoch, target triple) lives in the shared
//! `libs/build-stamp` crate so keep/loom/lumen stop carrying near-identical
//! copies; this file only supplies lumen's `LUMEN` env-var prefix.

fn main() {
    build_stamp::stamp("LUMEN");
}
// CODEGEN-END
