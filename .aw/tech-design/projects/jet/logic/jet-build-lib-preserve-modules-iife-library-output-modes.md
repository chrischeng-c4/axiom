---
id: projects-jet-logic-jet-build-lib-preserve-modules-iife-library-output-modes-md
fill_sections: [logic, changes]
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

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/bundler/lib_build.rs"
    action: modify
    section: logic
    description: |
      Implement preserve_modules (emit one output file per source module mirroring
      the source tree, externals externalized, entry re-exports) and IIFE library
      output (global-var wrapper, configurable global name), replacing the typed
      TODO bails. Return per-output entries in LibBuildResult.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/types.rs"
    action: modify
    section: logic
    description: |
      Support OutputFormat::Iife in library emission + a library_global_name
      option; preserve_modules already on BundleOptions.
    impl_mode: hand-written
  - path: "projects/jet/src/cli.rs"
    action: modify
    section: cli
    description: |
      Accept iife in --format and a --global-name flag (and [lib] config) for
      jet build --lib; thread preserve_modules through.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/library_build.rs"
    action: modify
    section: unit-test
    description: |
      Tests: preserve_modules emits one file per module (consumer imports a deep
      module); --format iife emits a loadable global IIFE; single-file default
      and app-mode unchanged.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (jet-lib-output-modes) complete + deterministic: start -> mode decision (single/preserve-modules/iife, all labeled) -> respective emit -> write -> terminal LibBuildResult. All nodes reachable; terminal real. Builds on A1.
