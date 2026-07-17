---
id: '1902'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-vat-lumen-observability-journey-logic
entry: build
nodes:
  build: { kind: process, label: "build current VAT, Lumen, and Sift binaries" }
  vat: { kind: process, label: "start real Lumen and Sift as VAT-managed services" }
  request: { kind: process, label: "VAT runner sends fixed traceparent to Lumen HTTP" }
  captured: { kind: decision, label: "trace id appears in VAT-advertised Lumen stdout" }
  collect: { kind: process, label: "runner invokes real sift collect on that exact path" }
  query: { kind: process, label: "runner queries real Sift logging API by trace id" }
  preserve: { kind: decision, label: "service payload message span parent and trace preserved" }
  artifact: { kind: process, label: "write bounded proof artifact and retain VAT evidence" }
  fail: { kind: terminal, label: "fail the real runner with captured diagnostics" }
  done: { kind: terminal, label: "local stdout architecture verified end to end" }
edges:
  - { from: build, to: vat }
  - { from: vat, to: request }
  - { from: request, to: captured }
  - { from: captured, to: fail, label: "no" }
  - { from: captured, to: collect, label: "yes" }
  - { from: collect, to: query }
  - { from: query, to: preserve }
  - { from: preserve, to: fail, label: "no" }
  - { from: preserve, to: artifact, label: "yes" }
  - { from: artifact, to: done }
---
flowchart TD
    build[build current VAT Lumen Sift] --> vat[start real services through VAT]
    vat --> request[send fixed traceparent to Lumen]
    request --> captured{trace in advertised stdout}
    captured -- no --> fail([fail with retained evidence])
    captured -- yes --> collect[run real sift collect]
    collect --> query[query Sift logs by trace]
    query --> preserve{correlation and payload preserved}
    preserve -- no --> fail
    preserve -- yes --> artifact[write bounded proof artifact]
    artifact --> done([local architecture verified])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/tests/vat_lumen_observability_e2e.rs
    action: create
    section: unit-test
    impl_mode: hand-written
  - path: projects/sift/observability/structured-stdout.md
    action: create
    section: logic
    impl_mode: hand-written
```
