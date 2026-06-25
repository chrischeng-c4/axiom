---
id: mamba-pkgmanage-remove-implicit-pypi-default-from-add-lock
summary: |
  Remove the implicit public PyPI fallback from the mamba package-manager add
  and lock source policy while preserving frozen local indexes and explicit
  private/PyPI-compatible registry URLs.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-pkgmanage-source-policy
entry: invoked
nodes:
  invoked: { kind: start, label: "mamba add/lock invoked" }
  local_index: { kind: decision, label: "Local frozen index configured?" }
  explicit_registry: { kind: decision, label: "Explicit registry configured?" }
  offline_pin: { kind: decision, label: "add --offline with NAME==VERSION?" }
  resolve_local: { kind: process, label: "Resolve against frozen local index" }
  resolve_registry: { kind: process, label: "Resolve against explicit registry URL" }
  record_offline: { kind: process, label: "Record pinned dependency without network" }
  fail_no_source: { kind: terminal, label: "Fail before manifest/lock writes" }
  render_lock: { kind: terminal, label: "Render deterministic manifest/lock" }
edges:
  - { from: invoked, to: local_index }
  - { from: local_index, to: resolve_local, label: "yes" }
  - { from: local_index, to: explicit_registry, label: "no" }
  - { from: explicit_registry, to: resolve_registry, label: "yes" }
  - { from: explicit_registry, to: offline_pin, label: "no" }
  - { from: offline_pin, to: record_offline, label: "yes" }
  - { from: offline_pin, to: fail_no_source, label: "no" }
  - { from: resolve_local, to: render_lock }
  - { from: resolve_registry, to: render_lock }
  - { from: record_offline, to: render_lock }
---
flowchart TD
    A["mamba add/lock invoked"] --> B{"Local frozen index configured?\n--index or MAMBA_FROZEN_INDEX"}
    B -- yes --> C["Resolve against frozen local index\n(no network, deterministic files)"]
    B -- no --> D{"Explicit registry configured?\n--index-url or MAMBA_INDEX_URL"}
    D -- yes --> E["Resolve against explicit private/PyPI-compatible registry\n(preserve wiremock and private registry paths)"]
    D -- no --> F{"add --offline with NAME==VERSION?"}
    F -- yes --> G["Record pinned dependency without network\n(no artifact URL/hash guarantee)"]
    F -- no --> H["Fail fast before manifest/lock writes\nDiagnostic names --index, MAMBA_FROZEN_INDEX, or --index-url"]

    C --> I["Render deterministic mamba.toml/mamba.lock"]
    E --> I
    G --> I
    H --> J["No mutation: existing mamba.toml unchanged,\nno partial mamba.lock"]

    I --> K["Verify frozen-index add/lock tests and explicit --index-url mock tests"]
    J --> L["Verify no-source add/lock failure tests"]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: mamba-pkgmanage-no-implicit-pypi-unit-tests
requirements:
  R1:
    text: "add without local index or explicit registry fails before any manifest or lockfile mutation."
    risk: high
    verify: integration-test
  R2:
    text: "lock without local index or explicit registry fails before writing a partial lockfile."
    risk: high
    verify: integration-test
  R3:
    text: "frozen local index and explicit --index-url registry paths continue to pass."
    risk: high
    verify: regression-test
---
requirementDiagram
    requirement add_no_source {
      id: R1
      text: "mamba add bare_dep with no source fails fast and leaves mamba.toml/mamba.lock unchanged."
      risk: high
      verifymethod: test
    }
    requirement lock_no_source {
      id: R2
      text: "mamba lock with deps and no source fails fast and does not write a partial lockfile."
      risk: high
      verifymethod: test
    }
    requirement source_paths_preserved {
      id: R3
      text: "Existing frozen-index and explicit --index-url pkgmgr tests still pass."
      risk: high
      verifymethod: test
    }
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Source-policy decision tree is bounded to add/lock and distinguishes frozen local index, explicit registry URL, offline pinned add, and fail-fast no-source behavior.
- [unit-test] Test requirements cover both new negative cases and regression protection for frozen-index plus explicit `--index-url` paths.
