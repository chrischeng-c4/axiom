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
      Change maybe_rewrite so it computes an execution payload separately from
      the original label. Keep first_program_is_cap and empty-command handling
      unchanged. Add a best-effort optimizer helper that only examines simple
      whole-command input with no shell metacharacters or command chaining. The
      first whitelist entry recognizes grep -R/-r style recursive searches with
      supported flags and positional pattern/path arguments, then emits an rg
      command only when rg is discoverable on PATH. Unsupported flags,
      non-recursive grep, pipeline grep, redirection, heredoc, and parse
      uncertainty return None so maybe_rewrite wraps the original command.

  - path: projects/cap/src/hook.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      For an optimized command, build the bash payload as a fallback script:
      run the optimized command first, and if it exits non-zero, run the
      original command. The cap label remains the original command in both
      optimized and unoptimized cases. This keeps failed optimizer attempts
      from changing the caller-visible command semantics.

  - path: projects/cap/src/hook.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Add unit tests for grep -R to rg conversion, rg unavailable fallback,
      original label preservation, optimized bash payload fallback shape,
      unsupported grep flags, non-recursive grep, and pipeline/chained shell
      commands staying on the original payload.

  - path: projects/cap/README.md
    action: modify
    section: docs
    impl_mode: hand-written
    description: >
      Document the auto optimizer as a small installed-tool-dependent
      whitelist. State that cap leaves commands untouched when it cannot prove
      a safe rewrite, and that optimized commands include runtime fallback to
      the original command on non-zero optimized exit.
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
