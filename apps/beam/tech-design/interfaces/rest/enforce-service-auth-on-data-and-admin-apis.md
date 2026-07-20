---
id: '2150'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: beam-service-auth
entry: start
nodes:
  start: { kind: start, label: "API Request received" }
  extract_token: { kind: process, label: "Extract Bearer Token (libs/service-auth)" }
  validate_token: { kind: decision, label: "Is Token Valid?" }
  reject_unauth: { kind: terminal, label: "Reject (401 Unauthorized)" }
  check_role: { kind: decision, label: "Role/Scope Allowed?" }
  reject_forbidden: { kind: terminal, label: "Reject (403 Forbidden)" }
  enforce_limits: { kind: process, label: "Apply request/body limits" }
  execute_api: { kind: process, label: "Execute Data/Admin API" }
  done: { kind: terminal, label: "Return Response" }
edges:
  - { from: start, to: extract_token }
  - { from: extract_token, to: validate_token }
  - { from: validate_token, to: reject_unauth, label: "No" }
  - { from: validate_token, to: check_role, label: "Yes" }
  - { from: check_role, to: reject_forbidden, label: "No" }
  - { from: check_role, to: enforce_limits, label: "Yes" }
  - { from: enforce_limits, to: execute_api }
  - { from: execute_api, to: done }
---
flowchart TD
    start([Start]) --> extract_token[Extract Bearer Token]
    extract_token --> validate_token{Token Valid?}
    validate_token -->|No| reject_unauth([401 Unauthorized])
    validate_token -->|Yes| check_role{Role Allowed?}
    check_role -->|No| reject_forbidden([403 Forbidden])
    check_role -->|Yes| enforce_limits[Enforce Request Limits]
    enforce_limits --> execute_api[Execute API]
    execute_api --> done([Return Response])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/beam/src/main.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: ServeArgs
  - path: apps/beam/src/service.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: router_with_state
  - path: apps/beam/tests/security_hardening.rs
    action: create
    section: unit-test
    impl_mode: hand-written
  - path: apps/beam/tests/service.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: service_end_to_end
```
