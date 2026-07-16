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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: service-http-admission-config-verification
requirements:
  disabled:
    id: R1
    text: "No configured class capacities leave admission disabled without changing existing service behavior."
    kind: regression
    risk: medium
    verify: libs/service-http/src/admission.rs::tests::config_without_capacities_is_disabled
  invalid:
    id: R3
    text: "Malformed values and common settings without enabled classes fail with the exact environment key in the error."
    kind: negative
    risk: medium
    verify: libs/service-http/src/admission.rs::tests::config_rejects_invalid_or_orphaned_common_values
  valid:
    id: R2
    text: "A valid prefix config creates independently enabled read/write/admin policies with shared refill and key bounds."
    kind: functional
    risk: high
    verify: libs/service-http/src/admission.rs::tests::config_builds_multi_class_controller
---
flowchart TD
    r1[R1 disabled] --> libs_service_http_src_admission_rs_tests_config_without_capacities_is_disabled[libs/service-http/src/admission.rs::tests::config_without_capacities_is_disabled]
    r2[R2 valid] --> libs_service_http_src_admission_rs_tests_config_builds_multi_class_controller[libs/service-http/src/admission.rs::tests::config_builds_multi_class_controller]
    r3[R3 invalid] --> libs_service_http_src_admission_rs_tests_config_rejects_invalid_or_orphaned_common_values[libs/service-http/src/admission.rs::tests::config_rejects_invalid_or_orphaned_common_values]
```
