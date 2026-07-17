---
id: '1872'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-active-run-service-log-path-handoff-contract
entry: validate_logs_root
nodes:
  validate_logs_root: { kind: process, label: "require the existing active logs directory to remain inside vat.dir" }
  normalize: { kind: process, label: "accept ASCII alphanumeric ids with internal dot dash underscore and map separators to underscore uppercase tokens" }
  unique: { kind: decision, label: "every required service produces one unique token" }
  derive: { kind: process, label: "derive <logs>/<id>.stdout.log and <logs>/<id>.stderr.log" }
  export: { kind: process, label: "VAT overwrites reserved VAT_LOGS_DIR and VAT_SERVICE_<TOKEN>_*_LOG values in run_env" }
  capture: { kind: process, label: "create or truncate both capture files before readiness completes and before runners start" }
  runner: { kind: process, label: "pass absolute host paths only to trusted same-run runner processes" }
  reject: { kind: terminal, label: "fail before any service starts" }
  done: { kind: terminal, label: "existing retention and VAT JSONL stdout behavior remain unchanged" }
edges:
  - { from: validate_logs_root, to: normalize }
  - { from: normalize, to: unique }
  - { from: unique, to: reject, label: "no or unsafe" }
  - { from: unique, to: derive, label: "yes" }
  - { from: derive, to: export }
  - { from: export, to: capture }
  - { from: capture, to: runner }
  - { from: runner, to: done }
---
flowchart TD
    validate_logs_root[validate active VAT logs root] --> normalize[validate ids and normalize environment tokens]
    normalize --> unique{tokens unique and ids safe}
    unique -- no --> reject([reject before service start])
    unique -- yes --> derive[derive stdout and stderr paths]
    derive --> export[publish reserved run environment]
    export --> capture[create capture files before runner]
    capture --> runner[trusted runner consumes absolute paths]
    runner --> done([retention and VAT stdout unchanged])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/src/commands/run.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: run_configured
  - path: apps/vat/tests/active_run_service_log_paths.rs
    action: create
    section: unit-test
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-active-run-service-log-path-handoff-verification
requirements:
  capture_before_runner:
    id: R3
    text: "VAT creates service capture files before the dependent runner starts."
    kind: functional
    risk: high
    verify: runner_follows_live_service_through_advertised_log_paths
  logs_directory:
    id: R1
    text: "The runner receives VAT_LOGS_DIR for the active run."
    kind: functional
    risk: medium
    verify: runner_follows_live_service_through_advertised_log_paths
  service_paths:
    id: R2
    text: "Each required service receives normalized stdout and stderr path variables that remain inside VAT_LOGS_DIR."
    kind: security
    risk: high
    verify: runner_follows_live_service_through_advertised_log_paths
  stdout_protocol:
    id: R4
    text: "The additive handoff does not replay captured child bytes into VAT JSONL stdout or alter retention."
    kind: regression
    risk: medium
    verify: runner_follows_live_service_through_advertised_log_paths
  unsafe_ids:
    id: R5
    text: "Unsafe ids and normalized environment-token collisions fail before service or runner start."
    kind: security
    risk: high
    verify: unsafe_or_colliding_service_log_environment_ids_are_rejected
---
flowchart TD
    r1[R1 logs directory] --> runner_follows_live_service_through_advertised_log_paths[runner_follows_live_service_through_advertised_log_paths]
    r2[R2 service paths] --> runner_follows_live_service_through_advertised_log_paths
    r3[R3 capture before runner] --> runner_follows_live_service_through_advertised_log_paths
    r4[R4 stdout protocol] --> runner_follows_live_service_through_advertised_log_paths
    r5[R5 unsafe ids] --> unsafe_or_colliding_service_log_environment_ids_are_rejected[unsafe_or_colliding_service_log_environment_ids_are_rejected]
```
