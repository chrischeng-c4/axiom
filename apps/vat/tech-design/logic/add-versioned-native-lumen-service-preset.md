---
id: '1813'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-versioned-native-lumen-preset-logic
entry: start
nodes:
  start: { kind: start, label: "prepare lumen preset" }
  selector: { kind: decision, label: "version omitted or lumen tag" }
  invalid: { kind: terminal, label: "reject malformed non-lumen selector" }
  cache: { kind: decision, label: "verified cached binary exists" }
  latest: { kind: process, label: "discover newest lumen release tag" }
  fetch: { kind: process, label: "download target archive into VAT cache" }
  checksum: { kind: decision, label: "published sha256 matches when supplied" }
  unavailable: { kind: terminal, label: "emit native lumen unavailable remediation" }
  runtime: { kind: decision, label: "runtime auto or native" }
  reject_runtime: { kind: terminal, label: "reject docker and microvm no fallback" }
  start_lumen: { kind: process, label: "run cached lumen serve on loopback port" }
  ready: { kind: process, label: "wait for GET readyz unless overridden" }
  export: { kind: process, label: "export LUMEN_URL and generic endpoint vars" }
  runner: { kind: process, label: "run VAT runner then teardown child" }
  done: { kind: terminal, label: "record evidence and return exit code" }
edges:
  - { from: start, to: selector }
  - { from: selector, to: invalid, label: "invalid" }
  - { from: selector, to: cache, label: "pinned or latest" }
  - { from: cache, to: runtime, label: "hit" }
  - { from: cache, to: latest, label: "latest missing" }
  - { from: latest, to: fetch }
  - { from: fetch, to: checksum }
  - { from: checksum, to: unavailable, label: "missing or mismatch" }
  - { from: checksum, to: runtime, label: "verified" }
  - { from: runtime, to: reject_runtime, label: "docker or microvm" }
  - { from: runtime, to: start_lumen, label: "auto or native" }
  - { from: start_lumen, to: ready }
  - { from: ready, to: export }
  - { from: export, to: runner }
  - { from: runner, to: done }
---
flowchart TD
    start([prepare lumen preset]) --> selector{version omitted or lumen tag}
    selector -- invalid --> invalid([reject malformed non-lumen selector])
    selector -- pinned or latest --> cache{verified cached binary exists}
    cache -- hit --> runtime{runtime auto or native}
    cache -- latest missing --> latest[discover newest lumen release tag]
    latest --> fetch[download target archive into VAT cache]
    fetch --> checksum{published sha256 matches when supplied}
    checksum -- missing or mismatch --> unavailable([emit native lumen unavailable remediation])
    checksum -- verified --> runtime
    runtime -- docker or microvm --> reject_runtime([reject docker and microvm no fallback])
    runtime -- auto or native --> start_lumen[run cached lumen serve on loopback port]
    start_lumen --> ready[wait for GET readyz unless overridden]
    ready --> export[export LUMEN URL and generic endpoint vars]
    export --> runner[run VAT runner then teardown child]
    runner --> done([record evidence and return exit code])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/tech-design/logic/add-versioned-native-lumen-service-preset.md
    action: modify
    section: logic
    impl_mode: hand-written
    reason: Define the versioned native Lumen preset lifecycle and public contract.
  - path: apps/vat/Cargo.toml
    action: modify
    section: changes
    impl_mode: codegen
    reason: Declare only the archive and checksum dependencies required by the VAT-owned Lumen release cache.
  - path: apps/vat/src/lib.rs
    action: modify
    section: changes
    impl_mode: codegen
    reason: Expose the dedicated Lumen release resolver module.
  - path: apps/vat/src/lumen_release.rs
    action: create
    section: changes
    impl_mode: codegen
    reason: Isolate release-tag normalization, latest discovery, target archive download, checksum verification, atomic cache materialization, and cached binary lookup.
  - path: apps/vat/src/config.rs
    action: modify
    section: changes
    impl_mode: codegen
    reason: Add the lumen preset token and native-only runtime/version validation.
  - path: apps/vat/src/commands/run.rs
    action: modify
    section: changes
    impl_mode: codegen
    reason: Resolve the cached Lumen binary, construct loopback serve command, readiness, exports, and teardown evidence without image fallback.
  - path: apps/vat/src/commands/doctor.rs
    action: modify
    section: changes
    impl_mode: codegen
    reason: Report versioned native Lumen cache/download readiness and remediation.
  - path: apps/vat/tests/vat_lumen_preset.rs
    action: create
    section: unit-test
    impl_mode: codegen
    reason: Cover deterministic resolver/cache/runtime behavior and an opt-in real Lumen process contract.
  - path: apps/vat/README.md
    action: modify
    section: changes
    impl_mode: hand-written
    reason: Document version pinning, latest selection, native-only execution, LUMEN_URL, and the no-persistent-data boundary.
```
