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
