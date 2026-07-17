---
id: '1872'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-active-run-service-log-path-handoff-logic
entry: resolve_services
nodes:
  resolve_services: { kind: process, label: "resolve the ordered required service set" }
  normalize_ids: { kind: process, label: "validate each service id and derive its unique VAT_SERVICE token" }
  collision: { kind: decision, label: "normalized token collision or unsafe id" }
  publish_paths: { kind: process, label: "publish VAT_LOGS_DIR and per-service stdout/stderr paths into run_env" }
  start_services: { kind: process, label: "create capture files and start services" }
  wait_ready: { kind: process, label: "wait for service readiness" }
  start_runner: { kind: process, label: "start trusted same-run runner with the published environment" }
  fail: { kind: terminal, label: "reject configuration before service or runner start" }
  done: { kind: terminal, label: "runner may follow captured stdout while services remain alive" }
edges:
  - { from: resolve_services, to: normalize_ids }
  - { from: normalize_ids, to: collision }
  - { from: collision, to: fail, label: "yes" }
  - { from: collision, to: publish_paths, label: "no" }
  - { from: publish_paths, to: start_services }
  - { from: start_services, to: wait_ready }
  - { from: wait_ready, to: start_runner }
  - { from: start_runner, to: done }
---
flowchart TD
    resolve_services[resolve ordered required services] --> normalize_ids[validate ids and derive unique environment tokens]
    normalize_ids --> collision{unsafe id or token collision}
    collision -- yes --> fail([reject before start])
    collision -- no --> publish_paths[publish VAT_LOGS_DIR and service log paths]
    publish_paths --> start_services[create capture files and start services]
    start_services --> wait_ready[wait until services are ready]
    wait_ready --> start_runner[start trusted same-run runner]
    start_runner --> done([runner follows captured stdout])
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
