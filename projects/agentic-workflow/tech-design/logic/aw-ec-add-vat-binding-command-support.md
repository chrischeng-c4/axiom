---
id: aw-ec-add-vat-binding-command-support
summary: Add vat as a supported AW EC binding tool so project EC categories can run vat-managed environment runners through aw health --verify-ec.
fill_sections: [logic, unit-test]
capability_refs:
  - id: project-local-td-and-ec-gates
    role: primary
    gap: ec-tool-binding-dispatch
    claim: ec-tool-binding-dispatch
    coverage: partial
    rationale: "Extends the existing EC binding dispatch from arena/rig/meter to include vat-managed runner environments."
---

# TD: aw EC vat binding command support

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
  A[EcBinding::command] --> B{tool}
  B -->|arena| C[require spec and emit arena run --spec <spec>]
  B -->|rig| D[require dir and emit rig run --dir <dir>]
  B -->|meter| E[require meter and emit meter run --target <meter>]
  B -->|vat| F{dir present?}
  F -->|yes| G[emit vat run <dir>]
  F -->|no| H[emit vat run]
  B -->|unknown| I[failed EC command with supported-tool list]
```
