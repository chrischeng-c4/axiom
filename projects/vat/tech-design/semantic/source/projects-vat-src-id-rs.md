---
id: vat-source-projects-vat-src-id-rs
summary: Source replay payload for projects/vat/src/id.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/id.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/id.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `fresh` | projects/vat/src/id.rs | function | pub | 17 | fresh() -> String |
## Source
<!-- type: source lang: rust -->

`````rust
//! Vat identifiers.
//!
//! An id is short, lowercase, and greppable: `vat-` + a base36 stamp derived
//! from the wall clock and pid. Collisions are astronomically unlikely for a
//! local, single-user tool; if two vats ever land on the same id, [`store`]
//! refuses to clobber an existing directory.
//!
//! [`store`]: crate::store

use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a fresh vat id, e.g. `vat-7f3k1q9`.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-id-rs.md#source
pub fn fresh() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // Mix in the pid so two vats created within the same nanosecond tick
    // (e.g. a fork burst) still diverge.
    let mixed = nanos ^ ((process::id() as u128) << 80);
    format!("vat-{}", base36(mixed as u64 & 0xff_ffff_ffff))
}

/// Lowercase base36 of a u64 (no leading-zero padding; ids are opaque).
fn base36(mut n: u64) -> String {
    const ALPHABET: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".to_string();
    }
    let mut buf = Vec::new();
    while n > 0 {
        buf.push(ALPHABET[(n % 36) as usize]);
        n /= 36;
    }
    buf.reverse();
    String::from_utf8(buf).expect("base36 alphabet is ascii")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_ids_have_prefix_and_differ() {
        let a = fresh();
        assert!(a.starts_with("vat-"), "got {a}");
        // The clock advances between calls, so ids differ in practice.
        let b = fresh();
        assert_ne!(a, b);
    }

    #[test]
    fn base36_is_stable() {
        assert_eq!(base36(0), "0");
        assert_eq!(base36(35), "z");
        assert_eq!(base36(36), "10");
    }
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/id.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-src.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-id-rs-source-replay-superseded>"
```
