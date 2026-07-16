---
id: '1827'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-shared-admission-adoption
entry: startup
nodes:
  startup: { kind: start, label: "Tape serve startup" }
  config: { kind: process, label: "Parse TAPE_ADMISSION config in service-http" }
  disabled: { kind: terminal, label: "No policies: existing unlimited router" }
  enabled: { kind: process, label: "Inject controller into Tape read/write/admin router" }
  reject: { kind: terminal, label: "429 plus Retry-After" }
edges:
  - { from: startup, to: config }
  - { from: config, to: disabled, label: absent }
  - { from: config, to: enabled, label: configured }
  - { from: enabled, to: reject, label: excess request }
---
flowchart TD
  startup["Tape serve"] --> config["Parse shared TAPE admission config"]
  config -->|absent| disabled(["Existing unlimited behavior"])
  config -->|configured| enabled["Inject shared controller into Tape classes"] --> reject(["429 Retry-After"])
```
