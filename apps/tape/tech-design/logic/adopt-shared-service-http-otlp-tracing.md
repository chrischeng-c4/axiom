---
id: '1662'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-shared-otlp-contract
entry: args
nodes:
  args: { kind: start, label: "Tape ServeArgs owns bind grace auth and TAPE_OTLP_ENDPOINT" }
  config: { kind: process, label: "create service-http config with Rust log compatible defaults" }
  identity: { kind: process, label: "service identity is tape plus package version" }
  initializer: { kind: process, label: "shared initializer installs logging or optional OTLP tracing" }
  span: { kind: process, label: "existing shared request trace layer receives W3C parent propagation" }
  domain: { kind: terminal, label: "Tape authentication journal and topic routes remain domain owned" }
edges:
  - { from: args, to: config }
  - { from: config, to: identity }
  - { from: identity, to: initializer }
  - { from: initializer, to: span }
  - { from: span, to: domain }
---
flowchart TD
    args([Tape ServeArgs owns bind grace auth and TAPE_OTLP_ENDPOINT]) --> config[create service-http config with Rust log compatible defaults]
    config --> identity[service identity is tape plus package version]
    identity --> initializer[shared initializer installs logging or optional OTLP tracing]
    initializer --> span[existing shared request trace layer receives W3C parent propagation]
    span --> domain([Tape authentication journal and topic routes remain domain owned])
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
    description: Lock Tape shared tracing wiring and feature propagation.
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
