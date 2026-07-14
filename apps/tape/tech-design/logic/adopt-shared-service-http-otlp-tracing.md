---
id: '1662'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-adopt-shared-otlp-tracing
entry: start
nodes:
  start: { kind: start, label: "tape serve resolves TAPE_OTLP_ENDPOINT and RUST_LOG" }
  config: { kind: process, label: "build shared HttpConfig with Tape logging defaults" }
  identity: { kind: process, label: "create ServiceIdentity tape plus build version" }
  init: { kind: process, label: "call service-http shared trace initializer before auth and server startup" }
  fallback: { kind: decision, label: "endpoint feature and exporter setup valid" }
  logging: { kind: process, label: "keep structured logging and start Tape" }
  otlp: { kind: process, label: "export shared request spans with W3C parents" }
  done: { kind: terminal, label: "Tape topic and subscription domain logic is unchanged" }
edges:
  - { from: start, to: config }
  - { from: config, to: identity }
  - { from: identity, to: init }
  - { from: init, to: fallback }
  - { from: fallback, to: logging, label: "none invalid or unavailable" }
  - { from: fallback, to: otlp, label: "enabled" }
  - { from: logging, to: done }
  - { from: otlp, to: done }
---
flowchart TD
    start([tape serve resolves TAPE_OTLP_ENDPOINT and RUST_LOG]) --> config[build shared HttpConfig with Tape logging defaults]
    config --> identity[create ServiceIdentity tape plus build version]
    identity --> init[call shared trace initializer before auth and server startup]
    init --> fallback{endpoint feature and exporter setup valid}
    fallback -->|none invalid or unavailable| logging[keep structured logging and start Tape]
    fallback -->|enabled| otlp[export shared request spans with W3C parents]
    logging --> done([Tape topic and subscription domain logic unchanged])
    otlp --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Add a Tape otel feature that enables shared service-http trace export.
  - path: apps/tape/src/bin/tape.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Map Tape serve configuration to shared logging and OTLP initialization.
  - path: apps/tape/tests/shared_otlp_tracing.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Lock Tape's shared tracing wiring and feature propagation boundary.
  - path: apps/tape/README.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: Record shared OTLP trace export in Tape observability capability evidence.
  - path: apps/tape/tech-design/semantic/source/apps-tape-src-bin-tape-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Capture the shared trace initializer and retained Tape domain boundary.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-adopt-shared-otlp-tracing-verification
requirements:
  feature_fallback:
    id: R2
    text: "Tape's optional otel feature enables the shared exporter while the default build remains logging-only and starts without an endpoint."
    kind: regression
    risk: high
    verify: cargo test -p tape --features otel --test shared_otlp_tracing -- --exact
  shared_initializer:
    id: R1
    text: "Tape maps TAPE_OTLP_ENDPOINT into service-http initialization with stable Tape identity before serving requests."
    kind: contract
    risk: high
    verify: cargo test -p tape --test shared_otlp_tracing -- --exact
---
flowchart TD
    r1[R1 shared initializer] --> cargo_test_p_tape_test_shared_otlp_tracing_exact[cargo test -p tape --test shared_otlp_tracing -- --exact]
    r2[R2 feature fallback] --> cargo_test_p_tape_features_otel_test_shared_otlp_tracing_exact[cargo test -p tape --features otel --test shared_otlp_tracing -- --exact]
```
