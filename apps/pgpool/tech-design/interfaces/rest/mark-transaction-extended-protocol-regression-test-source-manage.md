---
id: '1922'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-transaction-regression-source-ownership
entry: test_source
nodes:
  test_source: { kind: start, label: "Existing extended-protocol integration regression test" }
  managed: { kind: process, label: "Attach SPEC-MANAGED source mirror" }
  ownership: { kind: process, label: "Wrap test implementation in HANDWRITE ownership" }
  verify: { kind: terminal, label: "Test passes and AW ownership audit is clean" }
edges:
  - { from: test_source, to: managed }
  - { from: managed, to: ownership }
  - { from: ownership, to: verify }
---
flowchart LR
    test_source([Extended-protocol regression]) --> managed[SPEC-MANAGED mirror]
    managed --> ownership[HANDWRITE ownership]
    ownership --> verify([Targeted test and code-check pass])
```

The follow-up changes ownership metadata only. The existing regression continues proving error-then-close for both transaction engines; its new source mirror and HANDWRITE block let AW audit that proof without changing wire behavior.
