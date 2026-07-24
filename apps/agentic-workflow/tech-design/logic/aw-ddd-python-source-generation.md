---
id: aw-ddd-python-source-generation
summary: "Emit deterministic DDD-grouped Python source and separate native unit-test inventory from PythonTdIr."
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: partial
    rationale: "Python TD source emission consumes the checked semantic inventory without changing the generic lifecycle."
---

# DDD Python Source Generation

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-ddd-python-source-generation
entry: emit
nodes:
  emit: { kind: start, label: "consume checked PythonTdIr" }
  paths: { kind: process, label: "write deterministic src bounded-context paths and package markers" }
  declarations: { kind: process, label: "render classes/functions from target-neutral declarations" }
  unit: { kind: process, label: "write native unittest inventory separately under tests/unit" }
  gap: { kind: decision, label: "unsupported declaration semantics?" }
  ownership: { kind: decision, label: "every existing collision byte-identical or generator-owned?" }
  stage: { kind: process, label: "copy unrelated files and render the complete candidate set in a sibling stage" }
  swap: { kind: process, label: "atomically swap staged tree with rollback" }
  output: { kind: terminal, label: "return generated file manifest and digest" }
  reject: { kind: terminal, label: "report generator gap or unowned collision before mutation; never emit EC" }
edges:
  - { from: emit, to: paths }
  - { from: paths, to: declarations }
  - { from: declarations, to: unit }
  - { from: unit, to: gap }
  - { from: gap, to: ownership, label: "none" }
  - { from: gap, to: reject, label: "present" }
  - { from: ownership, to: stage, label: "owned or absent" }
  - { from: ownership, to: reject, label: "unowned collision" }
  - { from: stage, to: swap }
  - { from: swap, to: output }
---
flowchart TD
  emit([checked IR]) --> paths[DDD src paths]
  paths --> declarations[render declarations]
  declarations --> unit[separate unittest inventory]
  unit --> gap{unsupported?}
  gap -->|no| ownership{owned collision set?}
  gap -->|yes| reject([explicit gap])
  ownership -->|yes| stage[sibling staging tree]
  ownership -->|no| reject
  stage --> swap[atomic swap plus rollback]
  swap --> output([manifest plus digest])
```

Generation uses only the canonical PythonTdIr. It creates package markers and
deterministic source skeletons under `src/`, plus a minimal installable
`pyproject.toml`; unit tests live under `tests/unit` and verify generated
declaration imports. It deliberately does not write
`external-contracts/`, does not copy reference source bodies, and does not
convert an unmodelled declaration into HANDWRITE without an explicit gap.
Function bodies are an explicit `python-td-function-body` HANDWRITE gap until
the semantic IR models their implementation.

Every emitted file carries the stable
`aw.python-td-native-target.v1` ownership sentinel. Before the first target
mutation, the shared materializer validates every candidate path and rejects
an existing collision unless its bytes are identical or already carry that
sentinel. Accepted updates copy the existing tree to a sibling staging
directory, preserve unrelated files, render the complete write set there, and
swap it into place with rollback.

The public target entrypoint is `aw cb gen --target python --source-root
<python-project> --output-dir <target>`. It is intentionally separate from
slug-based lifecycle generation: it consumes a checked Python project as input
and emits only its target package and native unit tests.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-ddd-python-source-generation-unit-tests
requirements:
  cold_determinism: { id: R1, text: "Both reference IRs produce byte-identical cold Python targets.", kind: regression, risk: high, verify: "cargo test -p agentic-workflow --test python_td_target -- --nocapture" }
  native_tests: { id: R2, text: "Generated unittest suites are separate from EC and pass under CPython.", kind: contract, risk: high, verify: "cargo test -p agentic-workflow --test python_td_target -- --nocapture" }
  unowned_collision: { id: R3, text: "Rust, Python, and TypeScript existing projects reject every unowned collision before mutation.", kind: safety, risk: critical, verify: "cargo test -p agentic-workflow --test python_td_target native_targets_reject_existing_projects_before_any_write -- --exact" }
  owned_update: { id: R4, text: "Generator-owned updates preserve unrelated existing-project files.", kind: regression, risk: high, verify: "cargo test -p agentic-workflow --test python_td_target native_targets_update_owned_files_and_preserve_unrelated_files -- --exact" }
elements:
  python_td_target_generates_deterministic_packages_and_native_tests: { kind: test, type: "rs/#[test]" }
  native_targets_reject_existing_projects_before_any_write: { kind: test, type: "rs/#[test]" }
  native_targets_update_owned_files_and_preserve_unrelated_files: { kind: test, type: "rs/#[test]" }
relations:
  - { from: python_td_target_generates_deterministic_packages_and_native_tests, verifies: cold_determinism }
  - { from: python_td_target_generates_deterministic_packages_and_native_tests, verifies: native_tests }
  - { from: native_targets_reject_existing_projects_before_any_write, verifies: unowned_collision }
  - { from: native_targets_update_owned_files_and_preserve_unrelated_files, verifies: owned_update }
---
requirementDiagram
  requirement R1 { id: R1 text: "cold deterministic Python target" risk: high verifymethod: test }
  requirement R2 { id: R2 text: "separate native unit test" risk: high verifymethod: test }
  requirement R3 { id: R3 text: "unowned collisions fail before mutation" risk: critical verifymethod: test }
  requirement R4 { id: R4 text: "owned updates preserve unrelated files" risk: high verifymethod: test }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/services/python_td_target.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Render PythonTdIr to deterministic, explicitly owned package skeletons and unittest inventory."
  - path: apps/agentic-workflow/src/services/python_td.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Preflight the complete native target write set and atomically preserve unrelated existing-project files."
  - path: apps/agentic-workflow/src/services/python_td_rust_target.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Mark every Rust native target artifact with the shared generator owner."
  - path: apps/agentic-workflow/src/services/python_td_typescript_target.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Mark every TypeScript native target artifact with the shared generator owner."
  - path: apps/agentic-workflow/src/cli/cb.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Route aw td gen --target python to the native Python TD emitter."
  - path: apps/agentic-workflow/src/cli/td.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Keep target-native generation offline by skipping issue lifecycle guards."
  - path: apps/agentic-workflow/tests/python_td_target.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Cold-generate both reference IRs, compare manifests, and run generated CPython unit tests."
```
