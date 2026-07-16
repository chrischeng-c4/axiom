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

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-http/src/admission.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add a typed, service-prefix admission configuration parser that produces an optional shared AdmissionController. generator gap: missing-generator:service-http-admission-config (#1823)."
```
