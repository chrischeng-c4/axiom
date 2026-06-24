---
id: projects-jet-logic-jet-build-lib-preserve-modules-iife-library-output-modes-md
fill_sections: [logic]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: library-build-mode
    coverage: partial
    rationale: "preserve_modules and IIFE library output complete the jet build --lib output-mode matrix (previously typed TODO bails)."
---

# jet build --lib: preserve_modules + IIFE Output Modes

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-lib-output-modes
entry: start
nodes:
  start:    { kind: start,    label: "build_library per entry (A1)" }
  mode:     { kind: decision, label: "output mode" }
  bundle:   { kind: process,  label: "single-file: bundle entry (existing default)" }
  preserve: { kind: process,  label: "preserve_modules: one file per source module, mirror tree, re-export entry" }
  iife:     { kind: process,  label: "iife: wrap as global-var IIFE (global name), externals as globals" }
  emit:     { kind: process,  label: "write outputs under out_dir" }
  done:     { kind: terminal, label: "LibBuildResult per (entry x format/mode)" }
edges:
  - { from: start,    to: mode }
  - { from: mode,     to: bundle,   label: "single" }
  - { from: mode,     to: preserve, label: "preserve-modules" }
  - { from: mode,     to: iife,     label: "iife" }
  - { from: bundle,   to: emit }
  - { from: preserve, to: emit }
  - { from: iife,     to: emit }
  - { from: emit,     to: done }
---
flowchart TD
    start([build_library per entry]) --> mode{output mode}
    mode -->|single| bundle[bundle entry single file]
    mode -->|preserve-modules| preserve[one file per module, mirror tree]
    mode -->|iife| iife[global-var IIFE]
    bundle --> emit[write outputs]
    preserve --> emit
    iife --> emit
    emit --> done([LibBuildResult])
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Applicability sound: per-entry build branches on output mode (single-file existing default / preserve_modules one-file-per-module / IIFE global), then writes outputs. Builds on A1; .d.ts (LF2) and CJS edges (LF3) out of scope.
