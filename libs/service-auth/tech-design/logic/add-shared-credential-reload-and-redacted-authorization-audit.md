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
