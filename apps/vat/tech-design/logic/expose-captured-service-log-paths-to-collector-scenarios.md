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
