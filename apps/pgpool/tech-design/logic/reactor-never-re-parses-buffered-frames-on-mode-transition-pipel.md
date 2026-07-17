---
id: '1878'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: reactor-buffered-frame-resume
entry: readable_or_resumed_client
nodes:
  ingest: { kind: process, label: "drain socket into FrameReader when readable" }
  parse: { kind: process, label: "drain buffered frames while client_can_read" }
  stop: { kind: terminal, label: "stop parsing and retain socket backpressure" }
  resume: { kind: process, label: "mode transition resumes buffered parsing without new socket readiness" }
edges:
  - { from: readable_or_resumed_client, to: ingest, label: socket readable }
  - { from: ingest, to: parse }
  - { from: readable_or_resumed_client, to: resume, label: read-enabled transition }
  - { from: resume, to: parse }
  - { from: parse, to: stop, label: waiting or pending }
  - { from: parse, to: parse, label: next buffered frame remains permitted }
---
flowchart TD
  readable["socket readable"] --> ingest["drain socket into FrameReader"]
  ingest --> parse["drain buffered frames while client_can_read"]
  parse --> waiting{"frame makes client Waiting or pending?"}
  waiting -->|yes| stop["stop parsing and keep socket read backpressure"]
  waiting -->|no| parse

  transition["mode transition enables reads"] --> buffered["resume buffered parse without socket readiness"]
  buffered --> parse
  transition --- events["ReadyForQuery Idle, startup replay, auth challenge"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/reactor/runtime.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: read_client
  - path: apps/pgpool/tests/trust_startup_replay.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: backend_first_relay_keeps_pipelined_query_out_of_resetting_backend
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: reactor-buffered-frame-resume-verification
requirements:
  backpressure_preserved:
    id: R2
    text: "A pipelined next query remains out of the active backend until ReadyForQuery and reset complete, preserving the existing waiting/pending backpressure boundary."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test trust_startup_replay backend_first_relay_keeps_pipelined_query_out_of_resetting_backend
  saturated_pipeline_completes:
    id: R1
    text: "A reactor client that writes two and three simple-query frames in one segment while the sole backend is busy receives every response without sending additional bytes."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test trust_startup_replay reactor_saturated_pipelined_queries_resume_without_new_socket_read
  startup_pipeline_completes:
    id: R1
    text: "A StartupMessage and its first simple query written in one segment complete after handshake without an extra client write."
    kind: functional
    risk: high
    verify: cargo test -p pgpool --test trust_startup_replay reactor_pipelined_startup_and_first_query_complete
---
flowchart TD
    r1[R1 saturated pipeline completes] --> cargo_test_p_pgpool_test_trust_startup_replay_reactor_saturated_pipelined_queries_resume_without_new_socket_read[cargo test -p pgpool --test trust_startup_replay reactor_saturated_pipelined_queries_resume_without_new_socket_read]
    r1[R1 startup pipeline completes] --> cargo_test_p_pgpool_test_trust_startup_replay_reactor_pipelined_startup_and_first_query_complete[cargo test -p pgpool --test trust_startup_replay reactor_pipelined_startup_and_first_query_complete]
    r2[R2 backpressure preserved] --> cargo_test_p_pgpool_test_trust_startup_replay_backend_first_relay_keeps_pipelined_query_out_of_resetting_backend[cargo test -p pgpool --test trust_startup_replay backend_first_relay_keeps_pipelined_query_out_of_resetting_backend]
```
