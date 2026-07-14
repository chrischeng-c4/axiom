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
