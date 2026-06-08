---
id: sdd-generate-from-td-ast-bench
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# TDAst Dispatch Bench Stub

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/benches/dispatch_perf.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->

```rust
//! Bench scaffolding for R10: TDAst-based dispatch performance.
//!
//! Stage 2 keeps this as a stub - the perf comparison vs. legacy dispatch
//! lands once Stage 2B migrates the generators end-to-end. Compiles as a
//! `--bin` so cargo doesn't need a bench target wired up.
//!
//! @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic

fn main() {
    println!("dispatch_perf: bench scaffolding only; see Stage 2B follow-up.");
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/benches/dispatch_perf.rs
    action: create
    section: source
    impl_mode: codegen
    description: "Bench harness stub for the future TDAst dispatch performance comparison."
```
