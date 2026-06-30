---
id: vat-source-projects-vat-src-commands-gpu-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/commands/gpu.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/commands/gpu.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/gpu.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | projects/vat/src/commands/gpu.rs | function | pub | 16 | exec(json: bool) -> Result<ExitCode> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `vat gpu` — report the GPU every vat on this host can reach.
//!
//! The fastest way for an agent (or a curious human) to confirm the headline
//! claim: on Apple Silicon this prints an accessible Metal device, where the
//! same probe inside a Docker container reports nothing.

use std::process::ExitCode;

use anyhow::Result;

use crate::gpu;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-gpu-rs.md#source
pub fn exec(json: bool) -> Result<ExitCode> {
    let info = gpu::detect();
    if json {
        crate::commands::print_json(&info, false)?;
        return Ok(ExitCode::SUCCESS);
    }
    let chip = info.chip.as_deref().unwrap_or("unknown");
    let mark = if info.accessible {
        "✓ accessible"
    } else {
        "✗ not accessible"
    };
    println!("vendor   {}", info.vendor);
    println!("chip     {chip}");
    println!("backends {}", info.backends.join(", "));
    println!("status   {mark}");
    println!("note     {}", info.note);
    Ok(ExitCode::SUCCESS)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/gpu.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/commands/gpu.rs` captured during #39 vat standardization.
```
