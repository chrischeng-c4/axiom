---
id: '2455'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-macos-agent-path
entry: request
nodes:
  request: { kind: start, label: "agent launch request" }
  inherited: { kind: process, label: "search inherited PATH in order" }
  resolved: { kind: decision, label: "executable found?" }
  fallback: { kind: process, label: "append bounded account-local CLI paths" }
  spawn: { kind: terminal, label: "spawn resolved agent through PTY" }
  missing: { kind: terminal, label: "recoverable unavailable error" }
edges:
  - { from: request, to: inherited }
  - { from: inherited, to: resolved }
  - { from: resolved, to: spawn, label: "yes" }
  - { from: resolved, to: fallback, label: "no" }
  - { from: fallback, to: spawn }
  - { from: fallback, to: missing, label: "not found" }
---
flowchart LR
    request([Agent launch]) --> inherited[Inherited PATH]
    inherited --> found{Found?}
    found -->|yes| spawn([PTY spawn])
    found -->|no| fallback[Account-local CLI paths]
    fallback --> resolved{Found?}
    resolved -->|yes| spawn
    resolved -->|no| missing([Recoverable error])
```

PtyRuntime retains inherited PATH lookup order. On macOS only, if that lookup does not find a bare agent program, it appends a bounded account-local fallback list derived from HOME: .local/bin, .cargo/bin, and /opt/homebrew/bin. Duplicate directories are ignored; an explicit absolute or relative program path never uses fallback search.

The resolver accepts an injected home directory and search path in tests. It validates executable regular files exactly as today and returns the same UnavailableBinary error when no candidate exists. The sidecar does not execute shell startup files, parse user configuration, install a tool, or mutate PATH globally. Stable and Beta bundle the same Rust sidecar, so both receive this resolver behavior.
