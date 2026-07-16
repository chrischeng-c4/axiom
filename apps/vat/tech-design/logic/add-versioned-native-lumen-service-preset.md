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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-versioned-native-lumen-preset-verification
requirements:
  lumen_fail_closed:
    id: R4
    text: "Docker and MicroVM runtime requests, bad cache/download/checksum states, and absent native resolution emit actionable failures without Docker/image fallback."
    kind: negative
    risk: high
    verify: lumen_preset_failure_tests
  lumen_preset_config:
    id: R1
    text: "ServicePreset deserializes lumen and validation accepts only an omitted version or a lumen@X.Y.Z release selector with auto/native runtime."
    kind: functional
    risk: medium
    verify: config_lumen_preset_validation
  lumen_real_binary:
    id: R5
    text: "An opt-in real Lumen gate starts a cached or installed release binary, observes readyz and LUMEN_URL from the runner, and confirms VAT teardown."
    kind: integration
    risk: high
    verify: lumen_preset_real_binary_e2e
  lumen_release_cache:
    id: R2
    text: "The resolver selects the latest lumen tag or exact pinned tag, rejects malformed selectors, validates supplied SHA-256, and atomically reuses VAT-owned cached binaries without calling global upgrade."
    kind: regression
    risk: high
    verify: lumen_release_cache_tests
  lumen_runtime_contract:
    id: R3
    text: "A prepared Lumen service runs cached lumen serve on loopback, waits on readyz by default, and exports LUMEN_URL plus generic service endpoint variables."
    kind: functional
    risk: high
    verify: lumen_preset_plan_tests
---
flowchart TD
    r1[R1 lumen preset config] --> config_lumen_preset_validation[config_lumen_preset_validation]
    r2[R2 lumen release cache] --> lumen_release_cache_tests[lumen_release_cache_tests]
    r3[R3 lumen runtime contract] --> lumen_preset_plan_tests[lumen_preset_plan_tests]
    r4[R4 lumen fail closed] --> lumen_preset_failure_tests[lumen_preset_failure_tests]
    r5[R5 lumen real binary] --> lumen_preset_real_binary_e2e[lumen_preset_real_binary_e2e]
```
