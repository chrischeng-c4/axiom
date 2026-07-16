---
id: '1823'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: service-http-admission-config
entry: env
nodes:
  env:
    kind: start
    label: "Service prefix and environment values"
  parse:
    kind: process
    label: "Parse optional class capacities and common settings"
  disabled:
    kind: terminal
    label: "No capacities: admission disabled"
  controller:
    kind: process
    label: "Build shared AdmissionController policies"
  error:
    kind: terminal
    label: "Return named configuration error"
edges:
  - { from: env, to: parse }
  - { from: parse, to: disabled, label: no class }
  - { from: parse, to: controller, label: valid }
  - { from: parse, to: error, label: invalid }
---
flowchart TD
  env["Service prefix + environment"] --> parse["Parse optional capacities and common settings"]
  parse -->|no class capacity| disabled(["Disabled by default"])
  parse -->|valid| controller["Build shared AdmissionController"]
  parse -->|invalid| error(["Named config error"])
```
