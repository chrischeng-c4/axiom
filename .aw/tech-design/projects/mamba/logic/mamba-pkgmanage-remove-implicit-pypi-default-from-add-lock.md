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
id: mamba-pkgmanage-source-policy-contract
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
    D -- yes --> E["Resolve against explicit private/PyPI-compatible registry\n(no implicit pypi.org default)"]
    D -- no --> F{"add --offline with NAME==VERSION?"}
    F -- yes --> G["Record pinned dependency without network\n(no artifact URL/hash guarantee)"]
    F -- no --> H["Fail fast before manifest/lock writes\nDiagnostic names --index, MAMBA_FROZEN_INDEX, --index-url, or MAMBA_INDEX_URL"]

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
id: mamba-pkgmanage-no-implicit-pypi-contract-tests
requirements:
  R1:
    text: "add_no_source_requires_explicit_registry asserts no-source add fails without mutation."
    risk: high
    verify: "cargo test -p mamba --test pkgmgr add_no_source_requires_explicit_registry -- --nocapture"
  R2:
    text: "lock_no_source_requires_explicit_registry asserts no-source lock fails without partial lockfile."
    risk: high
    verify: "cargo test -p mamba --test pkgmgr lock_no_source_requires_explicit_registry -- --nocapture"
  R3:
    text: "The complete pkgmgr integration binary stays green after source-policy changes."
    risk: high
    verify: "cargo test -p mamba --test pkgmgr"
---
requirementDiagram
    requirement add_no_source_requires_explicit_registry {
      id: R1
      text: "Add fails fast when no frozen index, no explicit registry, and no offline pin are present."
      risk: high
      verifymethod: test
    }
    requirement lock_no_source_requires_explicit_registry {
      id: R2
      text: "Lock fails fast when dependencies exist but no frozen index or explicit registry is present."
      risk: high
      verifymethod: test
    }
    requirement pkgmgr_umbrella_green {
      id: R3
      text: "Frozen-index, explicit registry, sync, install, hash, cache, run, and validate regressions remain green."
      risk: high
      verifymethod: test
    }
```
