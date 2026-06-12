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
  check_cap: { kind: decision, label: "empty or already cap?" }
  unchanged: { kind: terminal, label: "no hook rewrite" }
  reject_shell: { kind: decision, label: "contains pipeline, redirection, heredoc, or command chaining?" }
  parse_grep: { kind: decision, label: "simple recursive grep whitelist match?" }
  check_rg: { kind: decision, label: "rg executable is available on PATH?" }
  wrap_original: { kind: process, label: "cap run --label=original -- bash -c original" }
  build_rg: { kind: process, label: "translate safe grep args to rg args" }
  build_fallback: { kind: process, label: "build shell script: rg command || original command" }
  wrap_optimized: { kind: process, label: "cap run --label=original -- bash -c fallback-script" }
  run_rg: { kind: decision, label: "optimized rg command exits zero?" }
  return_rg: { kind: terminal, label: "return rg output/status" }
  run_original: { kind: terminal, label: "run original grep and return original status" }
edges:
  - { from: start, to: check_cap }
  - { from: check_cap, to: unchanged, label: "yes" }
  - { from: check_cap, to: reject_shell, label: "no" }
  - { from: reject_shell, to: wrap_original, label: "yes" }
  - { from: reject_shell, to: parse_grep, label: "no" }
  - { from: parse_grep, to: wrap_original, label: "no" }
  - { from: parse_grep, to: check_rg, label: "yes" }
  - { from: check_rg, to: wrap_original, label: "no" }
  - { from: check_rg, to: build_rg, label: "yes" }
  - { from: build_rg, to: build_fallback }
  - { from: build_fallback, to: wrap_optimized }
  - { from: wrap_optimized, to: run_rg }
  - { from: run_rg, to: return_rg, label: "success" }
  - { from: run_rg, to: run_original, label: "failure" }
---
flowchart TD
    start([PreToolUse Bash command]) --> check_cap{empty or already cap?}
    check_cap -- yes --> unchanged([no hook rewrite])
    check_cap -- no --> reject_shell{contains pipeline, redirection, heredoc, or command chaining?}
    reject_shell -- yes --> wrap_original[cap run --label=original -- bash -c original]
    reject_shell -- no --> parse_grep{simple recursive grep whitelist match?}
    parse_grep -- no --> wrap_original
    parse_grep -- yes --> check_rg{rg executable is available on PATH?}
    check_rg -- no --> wrap_original
    check_rg -- yes --> build_rg[translate safe grep args to rg args]
    build_rg --> build_fallback[build shell script: rg command || original command]
    build_fallback --> wrap_optimized[cap run --label=original -- bash -c fallback-script]
    wrap_optimized --> run_rg{optimized rg command exits zero?}
    run_rg -- success --> return_rg([return rg output/status])
    run_rg -- failure --> run_original([run original grep and return original status])
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
e2e_tests:
  - id: cap-hook-auto-command-optimizer-whitelist
    name: "cap hook auto command optimizer whitelist"
    capability_id: agent-hook-installation
    contract_id: hook-payload-rewrite-adapters
    category: behavior
    command: "cargo test -p cap hook -- --nocapture"
    assertions:
      - "simple recursive grep commands can be optimized to rg when rg is available"
      - "unsupported grep forms keep the original command payload"
      - "optimized payloads fall back to the original command when the optimized command exits unsuccessfully"
      - "cap run labels preserve the original command even when bash payload is optimized"
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Applicability covers the optimizer decision path, tool availability gate, cap-run wrapping, and optimized-command failure fallback to original command semantics.
- [changes] Change plan is scoped to hook rewrite behavior, focused tests, and user-facing docs; daemon throttling and non-hook cap run behavior stay out of scope.
- [e2e-test] Verification targets the focused hook tests that own rewrite payload behavior.
