---
id: cap-hook-auto-command-optimizer-whitelist
summary: Auto-optimized command whitelist for cap Bash hook rewrites.
fill_sections: [logic, changes, e2e-test]
capability_refs:
  - id: agent-hook-installation
    role: primary
    gap: hook-payload-rewrite-adapters
    claim: hook-payload-rewrite-adapters
    coverage: partial
    rationale: "The Bash hook rewrite adapter owns command payload transformation before cap-run wrapping."
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: partial
    rationale: "The optimizer must keep the command inside the existing cap-run lease wrapper and preserve fallback semantics."
---

# Cap Hook Auto Command Optimizer Whitelist

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-hook-auto-command-optimizer-whitelist-logic
entry: start
nodes:
  start: { kind: start, label: "PreToolUse Bash command" }
  check_cap: { kind: decision, label: "already cap or empty?" }
  unchanged: { kind: terminal, label: "leave unchanged" }
  match_whitelist: { kind: decision, label: "matches optimizer whitelist?" }
  wrap_original: { kind: process, label: "wrap original command with cap run" }
  check_tool: { kind: decision, label: "replacement binary installed?" }
  build_optimized: { kind: process, label: "build optimized read-only command" }
  wrap_fallback: { kind: process, label: "wrap fallback script with cap run" }
  run_optimized: { kind: decision, label: "optimized command succeeds?" }
  optimized_result: { kind: terminal, label: "return optimized result" }
  original_result: { kind: terminal, label: "run original command and return original result" }
edges:
  - { from: start, to: check_cap }
  - { from: check_cap, to: unchanged, label: "yes" }
  - { from: check_cap, to: match_whitelist, label: "no" }
  - { from: match_whitelist, to: wrap_original, label: "no" }
  - { from: match_whitelist, to: check_tool, label: "yes" }
  - { from: check_tool, to: wrap_original, label: "no" }
  - { from: check_tool, to: build_optimized, label: "yes" }
  - { from: build_optimized, to: wrap_fallback }
  - { from: wrap_fallback, to: run_optimized }
  - { from: run_optimized, to: optimized_result, label: "yes" }
  - { from: run_optimized, to: original_result, label: "no" }
---
flowchart TD
    start([PreToolUse Bash command]) --> check_cap{already cap or empty?}
    check_cap -- yes --> unchanged([leave unchanged])
    check_cap -- no --> match_whitelist{matches optimizer whitelist?}
    match_whitelist -- no --> wrap_original[wrap original command with cap run]
    match_whitelist -- yes --> check_tool{replacement binary installed?}
    check_tool -- no --> wrap_original
    check_tool -- yes --> build_optimized[build optimized read-only command]
    build_optimized --> wrap_fallback[wrap fallback script with cap run]
    wrap_fallback --> run_optimized{optimized command succeeds?}
    run_optimized -- yes --> optimized_result([return optimized result])
    run_optimized -- no --> original_result([run original command and return original result])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cap/src/hook.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Add a conservative command optimizer step before maybe_rewrite builds the
      cap-run shell payload. The optimizer uses a hardcoded whitelist, checks
      that the replacement tool exists, keeps the cap label equal to the
      original command, and emits a fallback shell payload that runs the
      original command if the optimized command exits unsuccessfully. Initial
      whitelist scope is simple recursive grep forms that can use rg.

  - path: projects/cap/src/hook.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Extend hook rewrite tests to cover optimized rg payload construction,
      missing replacement tool fallback to the original payload, unsupported
      grep forms staying unchanged, pipeline grep staying unchanged, and the
      runtime fallback script shape.

  - path: projects/cap/README.md
    action: modify
    section: docs
    impl_mode: hand-written
    description: >
      Document that cap hook may automatically optimize a small whitelist of
      read-only commands when the faster replacement is installed, and that any
      optimizer miss or optimized-command failure falls back to the original
      command semantics.
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests: []
```
