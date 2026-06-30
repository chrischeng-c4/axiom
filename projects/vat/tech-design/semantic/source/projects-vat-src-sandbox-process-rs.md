---
id: vat-source-projects-vat-src-sandbox-process-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/sandbox/process.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/sandbox/process.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/sandbox/process.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ProcessBackend` | projects/vat/src/sandbox/process.rs | struct | pub | 18 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/sandbox/process.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/sandbox/process.rs` captured during #39 vat standardization.
```
