---
id: mamba-bug-run-argv-forward
summary: Fix `mamba run script.py a b c` so the embedded Python runtime sees `sys.argv == ["script.py", "a", "b", "c"]`. Wire the CLI's trailing positional args through `RunCommand → RuntimeInit → sys.argv`; cover with a conformance fixture that captures `sys.argv` from a script and asserts the list.
fill_sections: [dependency, logic, changes, test-plan]
---

# `mamba run` argv → `sys.argv` forwarding

## Dependency
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: mamba-bug-run-argv-forward-types
types:
  MambaCli:        { kind: struct }
  RunCommand:      { kind: struct }
  RuntimeInit:     { kind: struct }
  SysModule:       { kind: struct }
  ArgvList:        { kind: struct }
  ConformanceTest: { kind: struct }
edges:
  - { from: MambaCli,        to: RunCommand,      kind: owns,       label: "parses argv after `run`" }
  - { from: RunCommand,      to: RuntimeInit,     kind: invokes,    label: "forwards [script, args...]" }
  - { from: RuntimeInit,     to: SysModule,       kind: writes,     label: "installs sys.argv" }
  - { from: SysModule,       to: ArgvList,        kind: owns,       label: "argv attribute" }
  - { from: ConformanceTest, to: RunCommand,      kind: exercises,  label: "drives `mamba run` fixture" }
  - { from: ConformanceTest, to: SysModule,       kind: asserts_on, label: "checks sys.argv stdout" }
---
classDiagram
    class MambaCli
    class RunCommand
    class RuntimeInit
    class SysModule
    class ArgvList
    class ConformanceTest
    MambaCli --> RunCommand : owns
    RunCommand --> RuntimeInit : invokes
    RuntimeInit --> SysModule : writes
    SysModule --> ArgvList : owns
    ConformanceTest --> RunCommand : exercises
    ConformanceTest --> SysModule : asserts_on
```
## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-bug-run-argv-forward-logic
entry: start
nodes:
  start: { kind: start, label: "mamba run script.py a b c" }
  parse: { kind: decision, label: "argv parser" }
  bug: { kind: terminal, label: "sys.argv missing args (current)" }
  build: { kind: process, label: "build argv list including script" }
  init: { kind: process, label: "RuntimeInit.set_argv" }
  sys: { kind: process, label: "sys.argv populated" }
  done: { kind: terminal, label: "script reads sys.argv" }
edges:
  - { from: start, to: parse }
  - { from: parse, to: bug,   label: "drops trailing args (current)" }
  - { from: parse, to: build, label: "forward all trailing positionals (fix)" }
  - { from: build, to: init }
  - { from: init,  to: sys }
  - { from: sys,   to: done }
---
flowchart TD
    start([mamba run script.py a b c]) --> parse{argv parser}
    parse -->|drops trailing args today| bug[sys.argv missing args]
    parse -->|fix: forward all trailing positionals| build[build argv list]
    build --> init[RuntimeInit.set_argv]
    init --> sys[sys.argv populated]
    sys --> done([script reads sys.argv])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/src/cli/run.rs
    action: modify
    impl_mode: hand-written
    description: Stop dropping trailing positional args. Collect everything after the `<script>` token (including the script path itself as element 0) into a `Vec<String>` and pass it to the runtime init call. Today the runtime only sees the script path.
  - path: projects/mamba/src/runtime/init.rs
    action: modify
    impl_mode: hand-written
    description: Accept the argv vector in the runtime-init entry point and install it onto the `sys` module's `argv` attribute before user code runs. Today this slot is set to `["<script>"]` regardless of CLI input.
  - path: projects/mamba/tests/conformance/argv_forwarding.rs
    action: create
    impl_mode: hand-written
    description: New conformance test that spawns `mamba run` against a fixture script (`fixtures/argv_print.py`) which prints `sys.argv` as JSON; the test asserts the captured stdout matches the expected list across R1–R4 (zero args, multi args, whitespace-preserved args, script path identity).
  - path: projects/mamba/tests/conformance/fixtures/argv_print.py
    action: create
    impl_mode: hand-written
    description: One-liner fixture script — `import sys, json; print(json.dumps(sys.argv))` — used by the conformance test to capture the runtime's `sys.argv`.
```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: mamba-bug-run-argv-forward-verification
requirements:
  argv_multi:        { id: R1, text: "mamba run script.py a b c yields sys.argv == [script.py, a, b, c]",       kind: functional,  risk: high,   verify: test }
  argv_zero:         { id: R2, text: "mamba run script.py with zero extra args yields sys.argv == [script.py]", kind: functional,  risk: medium, verify: test }
  argv_whitespace:   { id: R3, text: "Whitespace-quoted argv element preserved as single sys.argv entry",       kind: functional,  risk: medium, verify: test }
  argv_zero_is_path: { id: R4, text: "sys.argv[0] is the user-typed script path (no canonicalization)",         kind: functional,  risk: high,   verify: test }
  argv_repl_safe:    { id: R6, text: "REPL and -c modes keep their existing argv shape (no regression)",        kind: functional,  risk: medium, verify: test }
elements:
  test_argv_forwarding_multi_arg:    { kind: test, type: "rs/#[test]" }
  test_argv_forwarding_zero_arg:     { kind: test, type: "rs/#[test]" }
  test_argv_forwarding_whitespace:   { kind: test, type: "rs/#[test]" }
  test_argv_zero_is_script_path:     { kind: test, type: "rs/#[test]" }
  test_repl_argv_unchanged:          { kind: test, type: "rs/#[test]" }
relations:
  - { from: test_argv_forwarding_multi_arg,    verifies: argv_multi }
  - { from: test_argv_forwarding_zero_arg,     verifies: argv_zero }
  - { from: test_argv_forwarding_whitespace,   verifies: argv_whitespace }
  - { from: test_argv_zero_is_script_path,     verifies: argv_zero_is_path }
  - { from: test_repl_argv_unchanged,          verifies: argv_repl_safe }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "mamba run script.py a b c yields sys.argv == [script.py, a, b, c]"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "mamba run script.py with zero extra args yields sys.argv == [script.py]"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "Whitespace-quoted argv element preserved as single sys.argv entry"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "sys.argv[0] is the user-typed script path (no canonicalization, no binary-name leak)"
      risk: high
      verifymethod: test
    }
    requirement R6 {
      id: R6
      text: "REPL and -c modes keep their existing argv shape (no regression)"
      risk: medium
      verifymethod: test
    }
    element test_argv_forwarding_multi_arg { type: "rs/#[test]" }
    element test_argv_forwarding_zero_arg { type: "rs/#[test]" }
    element test_argv_forwarding_whitespace { type: "rs/#[test]" }
    element test_argv_zero_is_script_path { type: "rs/#[test]" }
    element test_repl_argv_unchanged { type: "rs/#[test]" }
    test_argv_forwarding_multi_arg - verifies -> R1
    test_argv_forwarding_zero_arg - verifies -> R2
    test_argv_forwarding_whitespace - verifies -> R3
    test_argv_zero_is_script_path - verifies -> R4
    test_repl_argv_unchanged - verifies -> R6
```

# Reviews

### Review 1
**Verdict:** approved

- [dependency] Six-type class diagram cleanly captures the data flow MambaCli → RunCommand → RuntimeInit → SysModule → ArgvList plus the ConformanceTest cross-cutting concern; edges are typed (owns/invokes/writes/asserts_on) so codegen has unambiguous relationships.
- [logic] Flowchart distinguishes the current-buggy path (parser drops trailing args → terminal `sys.argv missing args`) from the fix path (build full argv list → RuntimeInit.set_argv → sys.argv populated); decision/terminal/process node kinds match Mermaid Plus contract.
- [changes] Four-file change list is minimal and targeted — two `modify` (CLI run.rs + runtime init.rs) plus two `create` (conformance test + argv_print.py fixture); each entry names the today-state and the post-fix invariant, no incidental refactoring leaks in.
- [test-plan] R1–R4 + R6 covered by five `rs/#[test]` elements with verifies edges; R5 (indirect sys.argv read parity) is intentionally elided because it's a CPython invariant — acceptable per scope.
