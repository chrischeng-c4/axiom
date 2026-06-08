---
id: vat-source-projects-vat-src-sandbox-seatbelt-rs
summary: Source replay payload for projects/vat/src/sandbox/seatbelt.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/sandbox/seatbelt.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/sandbox/seatbelt.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SeatbeltBackend` | projects/vat/src/sandbox/seatbelt.rs | struct | pub | 20 |  |
| `available` | projects/vat/src/sandbox/seatbelt.rs | function | pub | 24 | available() -> bool |
## Source
<!-- type: source lang: rust -->

`````rust
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
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/sandbox/seatbelt.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-sandbox.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-sandbox-seatbelt-rs-source-replay-superseded>"
```
