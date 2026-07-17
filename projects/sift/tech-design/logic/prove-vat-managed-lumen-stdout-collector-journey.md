---
id: '1902'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-vat-lumen-observability-journey-contract
entry: prerequisites
nodes:
  prerequisites: { kind: decision, label: "current target/debug vat lumen and sift binaries exist" }
  config: { kind: process, label: "write temporary vat.toml with absolute binary paths and retained artifacts" }
  start: { kind: process, label: "VAT starts Lumen and Sift on allocated loopback ports" }
  probe: { kind: process, label: "re-executed Rust test probe sends traceparent and reads VAT_SERVICE_LUMEN_STDOUT_LOG" }
  collect: { kind: process, label: "probe executes target/debug/sift collect with checkpoint and quarantine paths" }
  query: { kind: process, label: "probe POSTs project local environment test trace query" }
  assert: { kind: decision, label: "one Lumen audit record preserves trace parent local span service event message and payload" }
  proof: { kind: process, label: "write observability-proof.json as VAT artifact" }
  fail: { kind: terminal, label: "nonzero VAT runner with retained logs state and diff" }
  done: { kind: terminal, label: "VAT result ok and proof artifact recorded" }
edges:
  - { from: prerequisites, to: fail, label: "no; print build command" }
  - { from: prerequisites, to: config, label: "yes" }
  - { from: config, to: start }
  - { from: start, to: probe }
  - { from: probe, to: collect }
  - { from: collect, to: query }
  - { from: query, to: assert }
  - { from: assert, to: fail, label: "no" }
  - { from: assert, to: proof, label: "yes" }
  - { from: proof, to: done }
---
flowchart TD
    prerequisites{all current debug binaries exist} -- no --> fail([fail with exact build command])
    prerequisites -- yes --> config[write temporary VAT contract]
    config --> start[start Lumen and Sift]
    start --> probe[runner sends traceparent and reads advertised stdout]
    probe --> collect[run real sift collect]
    collect --> query[query Sift by trace]
    query --> assert{all correlation and payload fields preserved}
    assert -- no --> fail
    assert -- yes --> proof[write retained proof artifact]
    proof --> done([VAT result ok])
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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sift-vat-lumen-observability-journey-verification
requirements:
  collector_cli:
    id: R3
    text: "The real Sift collector consumes only the advertised Lumen stdout path and public Sift endpoint."
    kind: integration
    risk: high
    verify: vat_managed_lumen_stdout_reaches_real_sift_query
  query_preserves:
    id: R4
    text: "Sift query preserves service identity structured payload message trace span and parent correlation."
    kind: integration
    risk: high
    verify: vat_managed_lumen_stdout_reaches_real_sift_query
  real_services:
    id: R1
    text: "VAT starts the current real Lumen and Sift binaries and a real runner probe."
    kind: functional
    risk: high
    verify: vat_managed_lumen_stdout_reaches_real_sift_query
  runbook:
    id: R5
    text: "A canonical runbook documents local and Kubernetes ownership flows plus the reproducible command."
    kind: documentation
    risk: medium
    verify: architecture_runbook_names_owned_boundaries_and_repro_command
  traceparent_stdout:
    id: R2
    text: "A fixed valid inbound traceparent appears in the VAT-advertised Lumen stdout path."
    kind: functional
    risk: high
    verify: vat_managed_lumen_stdout_reaches_real_sift_query
---
flowchart TD
    r1[R1 real services] --> vat_managed_lumen_stdout_reaches_real_sift_query[vat_managed_lumen_stdout_reaches_real_sift_query]
    r2[R2 traceparent stdout] --> vat_managed_lumen_stdout_reaches_real_sift_query
    r3[R3 collector cli] --> vat_managed_lumen_stdout_reaches_real_sift_query
    r4[R4 query preserves] --> vat_managed_lumen_stdout_reaches_real_sift_query
    r5[R5 runbook] --> architecture_runbook_names_owned_boundaries_and_repro_command[architecture_runbook_names_owned_boundaries_and_repro_command]
```
