---
id: vat-source-projects-vat-src-sandbox-seatbelt-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/sandbox/seatbelt.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/sandbox/seatbelt.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/sandbox/seatbelt.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
use crate::spec::EgressPolicy;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-seatbelt-rs.md#source
pub struct SeatbeltBackend {
    /// Outbound network egress policy baked into the generated profile.
    pub egress: EgressPolicy,
}

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
        let profile = profile_for(rootfs, self.egress);
        let mut argv = vec!["-p".to_string(), profile, program.to_string()];
        argv.extend(args.iter().cloned());
        ("sandbox-exec".to_string(), argv)
    }
}

/// Build a seatbelt profile string confining writes to `rootfs` + temp, and —
/// per the egress policy — restricting outbound network. With
/// [`EgressPolicy::Open`] the profile is byte-identical to the write-only
/// confinement (no network lines), so existing seatbelt runs are unchanged.
fn profile_for(rootfs: &Path, egress: EgressPolicy) -> String {
    let root = rootfs.display();
    // (allow default) then deny writes, then re-allow writes only under the
    // rootfs subtree and temp. Reads stay open so interpreters/toolchains
    // resolve their libraries.
    let mut profile = format!(
        "(version 1)\n\
         (allow default)\n\
         (deny file-write*)\n\
         (allow file-write* (subpath \"{root}\"))\n\
         (allow file-write* (subpath \"/private/tmp\"))\n\
         (allow file-write* (subpath \"/private/var/folders\"))\n\
         (allow file-write* (subpath \"/tmp\"))\n"
    );
    // Network egress: only OUTBOUND network is filtered; reads/file/GPU stay as
    // above. localhost stays reachable so vat's local emulators + http-mock
    // proxy still work (the routing in v1/v2 targets 127.0.0.1).
    match egress {
        EgressPolicy::Open => {}
        EgressPolicy::LocalhostOnly => profile.push_str(
            "(deny network*)\n\
             (allow network* (remote ip \"localhost:*\"))\n\
             (allow network* (remote unix-socket))\n",
        ),
        EgressPolicy::Deny => profile.push_str("(deny network*)\n"),
    }
    profile
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn root() -> PathBuf {
        PathBuf::from("/vat/rootfs")
    }

    /// `open` must be byte-identical to the pre-egress write-confinement profile.
    #[test]
    fn open_profile_is_write_confinement_only() {
        let expected = "(version 1)\n\
             (allow default)\n\
             (deny file-write*)\n\
             (allow file-write* (subpath \"/vat/rootfs\"))\n\
             (allow file-write* (subpath \"/private/tmp\"))\n\
             (allow file-write* (subpath \"/private/var/folders\"))\n\
             (allow file-write* (subpath \"/tmp\"))\n";
        assert_eq!(profile_for(&root(), EgressPolicy::Open), expected);
        // No network directive at all under `open`.
        assert!(!profile_for(&root(), EgressPolicy::Open).contains("network"));
    }

    #[test]
    fn localhost_only_denies_then_allows_localhost() {
        let p = profile_for(&root(), EgressPolicy::LocalhostOnly);
        assert!(p.contains("(deny network*)"));
        assert!(p.contains("(allow network* (remote ip \"localhost:*\"))"));
        // Write-confinement still present.
        assert!(p.contains("(allow file-write* (subpath \"/vat/rootfs\"))"));
    }

    #[test]
    fn deny_blocks_all_outbound_without_localhost_allow() {
        let p = profile_for(&root(), EgressPolicy::Deny);
        assert!(p.contains("(deny network*)"));
        assert!(!p.contains("allow network"));
    }
}

/// Minimal PATH lookup (no extra deps).
fn which(bin: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|candidate| candidate.is_file())
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/sandbox/seatbelt.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/sandbox/seatbelt.rs` captured during #39 vat standardization.
```
