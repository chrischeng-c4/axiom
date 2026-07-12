---
id: vat-source-projects-vat-src-sandbox-mod-rs
summary: >
  rust-source-unit TD AST payload for apps/vat/src/sandbox/mod.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized apps/vat/src/sandbox/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/vat/src/sandbox/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `pick` | apps/vat/src/sandbox/mod.rs | function | pub | 49 | pick(spec: &EnvSpec) -> Result<Box<dyn Sandbox>, String> |
| `process` | apps/vat/src/sandbox/mod.rs | module | pub | 20 |  |
| `seatbelt` | apps/vat/src/sandbox/mod.rs | module | pub | 21 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/src/sandbox/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `apps/vat/src/sandbox/mod.rs` captured during #39 vat standardization.
  - path: apps/vat/tech-design/semantic/source/projects-vat-src-sandbox-mod-rs.md
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1474 AC6: this doc had drifted stale — it still described the pre-#1300
      warn-and-fallback `pick(spec: &EnvSpec) -> Box<dyn Sandbox>` (unconditional
      return, eprintln!-and-continue on an unenforceable egress policy) instead
      of the fail-closed `pick(spec: &EnvSpec) -> Result<Box<dyn Sandbox>, String>`
      that has been the real source since #1300 (hard `Err` when a non-`Open`
      egress policy cannot be enforced). Overview symbols table and Source section
      corrected to match `apps/vat/src/sandbox/mod.rs` exactly, HANDWRITE markers
      included. Phase 1 of the microVM epic (#1471) adds an `Isolation::MicroVm`
      branch to this same `pick()` on top of this corrected baseline (see WI
      #1474's own TD `changes:` section for that follow-on edit).
```
