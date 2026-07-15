---
id: '1765'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-shared-labeled-prometheus-flow
entry: collect
nodes:
  collect:
    kind: start
    label: Pgpool collects live per-pool gauge values as shared labeled samples
  normalize:
    kind: process
    label: metrics-prometheus sorts every sample label set by label name
  escape:
    kind: process
    label: Escape backslash, double quote, and newline in every label value
  render:
    kind: process
    label: Emit HELP and TYPE once per supplied sample group, followed by deterministic labeled rows
  response:
    kind: terminal
    label: Pgpool serves the byte-compatible Prometheus 0.0.4 response
edges:
  - { from: collect, to: normalize }
  - { from: normalize, to: escape }
  - { from: escape, to: render }
  - { from: render, to: response }
---
flowchart LR
  collect[Collect live Pgpool gauges] --> normalize[Sort labels]
  normalize --> escape[Escape label values]
  escape --> render[Shared HELP TYPE and row rendering]
  render --> response([Serve unchanged metrics contract])
```
