---
id: '1641'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: reloadable-role-map-auth
entry: request
nodes:
  request: { kind: start, label: "Authenticate request against current registry snapshot" }
  credential: { kind: decision, label: "Bearer resolves in current snapshot?" }
  reject: { kind: process, label: "Emit redacted authentication deny event without bearer bytes" }
  principal: { kind: process, label: "Return audited principal bound to shared event sink" }
  authorize: { kind: process, label: "Authorize resource role and emit allow/deny decision" }
  reload: { kind: start, label: "Explicit registry reload request" }
  parse: { kind: decision, label: "Replacement registry parses and validates?" }
  preserve: { kind: process, label: "Keep last-known-good snapshot and emit reload failure" }
  swap: { kind: process, label: "Atomically swap validated snapshot and advance revision" }
  done: { kind: terminal, label: "No event or principal contains raw credentials" }
edges:
  - { from: request, to: credential }
  - { from: credential, to: reject, label: "no" }
  - { from: credential, to: principal, label: "yes or open mode" }
  - { from: principal, to: authorize }
  - { from: reject, to: done }
  - { from: authorize, to: done }
  - { from: reload, to: parse }
  - { from: parse, to: preserve, label: "no" }
  - { from: parse, to: swap, label: "yes" }
  - { from: preserve, to: done }
  - { from: swap, to: done }
---
flowchart TD
    request([Authenticate request]) --> credential{Bearer resolves?}
    credential -->|no| reject[Emit redacted deny]
    credential -->|yes or open| principal[Return audited principal]
    principal --> authorize[Authorize and emit decision]
    reload([Reload request]) --> parse{Valid replacement?}
    parse -->|no| preserve[Keep last-known-good]
    parse -->|yes| swap[Atomic snapshot swap]
    reject --> done([No raw credentials])
    authorize --> done
    preserve --> done
    swap --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-auth/src/reload.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Reloadable validated role-map snapshots, audited principals, redacted auth events, and backend-neutral event sinks."
  - path: libs/service-auth/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose the shared reload/audit module and public contract."
  - path: libs/service-auth/tech-design/semantic/source/libs-service-auth-src-lib-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep the canonical semantic source mirror aligned with the runtime module surface."
  - path: libs/service-auth/Cargo.toml
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Add the tracing dependency used by the shared redacted event sink."
  - path: libs/service-auth/README.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Publish the credential lifecycle and authorization-audit capability rooted at #1641."
  - path: apps/lumen/src/auth.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Adopt ReloadableRoleMapVerifier and expose explicit validated reload while preserving Lumen AuthContext."
  - path: apps/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep Lumen auth semantic source aligned with the shared reloadable verifier adoption."
  - path: apps/tape/src/auth.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Adopt the same reloadable verifier and audited-principal authorization helper."
  - path: apps/tape/src/server.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Run Tape data-plane auth with the shared reloadable verifier/principal types."
  - path: apps/tape/tests/service_auth.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Verify Tape rotation and authorization remain compatible through the shared lifecycle."
```
