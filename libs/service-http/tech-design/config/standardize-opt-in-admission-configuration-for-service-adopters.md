---
id: '1823'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: service-http-admission-config-contract
entry: input
nodes:
  input:
    kind: start
    label: "Prefix-scoped environment lookup"
  capacities:
    kind: process
    label: "Parse read write admin capacities"
  common:
    kind: process
    label: "Validate refill seconds and max keys"
  optional:
    kind: decision
    label: "Any class enabled?"
  none:
    kind: terminal
    label: "Return None"
  some:
    kind: terminal
    label: "Return shared controller"
edges:
  - { from: input, to: capacities }
  - { from: capacities, to: common }
  - { from: common, to: optional }
  - { from: optional, to: none, label: no }
  - { from: optional, to: some, label: yes }
---
flowchart TD
  input["Prefix-scoped lookup"] --> capacities["Parse class capacities"] --> common["Validate common settings"] --> optional{"Any class enabled?"}
  optional -->|no| none(["None: disabled"])
  optional -->|yes| some(["AdmissionController"])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-http/src/admission.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: AdmissionController
    description: "Expose a typed optional controller configuration parser for all service adopters. generator gap: missing-generator:service-http-admission-config (#1823)."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: service-http-admission-config-contract-verification
requirements:
  controller:
    id: R2
    text: "The parser creates policies that preserve the shared controller's real allow and deny semantics."
    kind: functional
    risk: high
    verify: libs/service-http/src/admission.rs::tests::config_builds_multi_class_controller
  disabled:
    id: R1
    text: "The parser returns no controller when all class capacity settings are absent."
    kind: regression
    risk: medium
    verify: libs/service-http/src/admission.rs::tests::config_without_capacities_is_disabled
  errors:
    id: R3
    text: "Invalid or orphaned common configuration is rejected with an exact key name."
    kind: negative
    risk: medium
    verify: libs/service-http/src/admission.rs::tests::config_rejects_invalid_or_orphaned_common_values
---
flowchart TD
    r1[R1 disabled] --> libs_service_http_src_admission_rs_tests_config_without_capacities_is_disabled[libs/service-http/src/admission.rs::tests::config_without_capacities_is_disabled]
    r2[R2 controller] --> libs_service_http_src_admission_rs_tests_config_builds_multi_class_controller[libs/service-http/src/admission.rs::tests::config_builds_multi_class_controller]
    r3[R3 errors] --> libs_service_http_src_admission_rs_tests_config_rejects_invalid_or_orphaned_common_values[libs/service-http/src/admission.rs::tests::config_rejects_invalid_or_orphaned_common_values]
```
