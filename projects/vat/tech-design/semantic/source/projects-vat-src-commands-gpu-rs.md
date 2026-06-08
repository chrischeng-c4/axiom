---
id: vat-source-projects-vat-src-commands-gpu-rs
summary: Source replay payload for projects/vat/src/commands/gpu.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/commands/gpu.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/gpu.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | projects/vat/src/commands/gpu.rs | function | pub | 16 | exec(json: bool) -> Result<ExitCode> |
## Source
<!-- type: source lang: rust -->

`````rust
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
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/commands/gpu.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-commands.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-commands-gpu-rs-source-replay-superseded>"
```
