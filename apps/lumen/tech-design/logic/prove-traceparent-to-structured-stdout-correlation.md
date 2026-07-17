---
id: '1871'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-structured-stdout-traceparent-conformance
entry: spawn
nodes:
  spawn:
    kind: start
    label: "reserve a loopback port and spawn CARGO_BIN_EXE_lumen serve with embedded WAL and JSON logs"
  isolate:
    kind: process
    label: "remove RUST_LOG, LUMEN_LOG_FORMAT, and LUMEN_OTLP_ENDPOINT inheritance; set auth off"
  drain:
    kind: process
    label: "drain piped stdout concurrently so startup and request logs cannot block the child"
  ready:
    kind: decision
    label: "healthz ready before deadline and child still running?"
  fail_start:
    kind: terminal
    label: "fail with child status when readiness is not reached"
  valid:
    kind: process
    label: "create collection trace-valid with fixed version-00 traceparent"
  invalid:
    kind: process
    label: "create collection trace-invalid with malformed traceparent"
  missing:
    kind: process
    label: "create collection trace-missing without traceparent"
  stop:
    kind: process
    label: "kill and wait for child, join stdout reader, parse every nonempty line"
  contract:
    kind: decision
    label: "all lines are axiom.service.log.v1 and three audit events exist?"
  fail_contract:
    kind: terminal
    label: "fail on mixed framing, missing audit event, Sift coupling, or exporter requirement"
  assert_valid:
    kind: process
    label: "valid audit trace_id equals inbound; parent_span_id equals inbound parent; span_id is valid and distinct"
  assert_roots:
    kind: process
    label: "invalid and missing audits have valid nonzero local trace/span ids and no propagated parent"
  pass:
    kind: terminal
    label: "real Lumen stdout fixture proves exporter-independent request correlation"
edges:
  - { from: spawn, to: isolate }
  - { from: isolate, to: drain }
  - { from: drain, to: ready }
  - { from: ready, to: fail_start, label: "no" }
  - { from: ready, to: valid, label: "yes" }
  - { from: valid, to: invalid }
  - { from: invalid, to: missing }
  - { from: missing, to: stop }
  - { from: stop, to: contract }
  - { from: contract, to: fail_contract, label: "no" }
  - { from: contract, to: assert_valid, label: "yes" }
  - { from: assert_valid, to: assert_roots }
  - { from: assert_roots, to: pass }
---
flowchart TD
    spawn[spawn isolated real Lumen process] --> drain[drain stdout while polling healthz]
    drain --> ready{ready before deadline?}
    ready -->|no| fail([startup failure])
    ready -->|yes| valid[valid fixed traceparent request]
    valid --> invalid[malformed traceparent request]
    invalid --> missing[missing traceparent request]
    missing --> stop[stop process; parse all stdout JSONL]
    stop --> contract{schema and three audits present?}
    contract -->|no| failContract([contract failure])
    contract -->|yes| propagated[assert preserved trace and parent plus local span]
    propagated --> roots[assert safe local roots]
    roots --> pass([Lumen conformance proven])
```

The process command passes `--log-format json`, `--wal embedded`, and `--log-level info`; it configures no OTLP endpoint and no Sift address. Readiness traffic may produce events, but every stdout line must independently parse and identify `service.name=lumen`. The three requests use distinct collection ids so each successful request emits exactly one `collection_create_or_extend` audit event from the real handler. The fixed valid header is `00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01`. Invalid and missing cases must still return success, generate nonzero lowercase ids, and carry no propagated parent.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/tests/structured_stdout_traceparent.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Run the real Lumen binary, make valid, invalid, and missing traceparent HTTP writes, capture stdout concurrently, and assert the shared JSONL and correlation contracts.
```

The product source already has the required adopter seams: `serve` maps `--log-format json` into shared `service-http` initialization, `api::router` applies the shared trace layer outside all routes, and the collection handler emits a domain audit event. The bounded change adds executable proof instead of duplicating those shared implementations.
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-structured-stdout-traceparent-verification
requirements:
  independence:
    id: R5
    text: "The conformance process has no OTLP endpoint or Sift dependency and produces stdout consumable as independent JSONL records."
    kind: integration
    risk: medium
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
  invalid_missing:
    id: R4
    text: "Malformed and missing traceparent requests both succeed and emit valid locally rooted correlation."
    kind: regression
    risk: high
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
  outer_trace_layer:
    id: R2
    text: "The public collection route audit event runs inside the shared request span rather than a Lumen-local propagation implementation."
    kind: contract
    risk: high
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
  structured_mode:
    id: R1
    text: "A real Lumen process uses the shared JSON formatter and every captured operational stdout line conforms to axiom.service.log.v1."
    kind: functional
    risk: high
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
  valid_parent:
    id: R3
    text: "A fixed valid traceparent preserves its trace and parent span ids while Lumen creates a distinct nonzero local span id."
    kind: functional
    risk: high
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
---
flowchart TD
    r1[R1 structured mode] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact[cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact]
    r2[R2 outer trace layer] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact
    r3[R3 valid parent] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact
    r4[R4 invalid missing] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact
    r5[R5 independence] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact
```
