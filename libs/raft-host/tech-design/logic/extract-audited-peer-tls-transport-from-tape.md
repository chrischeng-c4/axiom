---
id: '1704'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: raft-host-peer-tls-configuration
entry: env
nodes:
  env:
    kind: start
    label: "Service-specific peer TLS environment prefix"
  resolve:
    kind: process
    label: "raft-host PeerTlsConfig resolves the shared service-tls configuration"
  validate:
    kind: decision
    label: "Complete cert/key/CA material present and valid?"
  plain:
    kind: terminal
    label: "No material: explicit plaintext h2c mode"
  typed:
    kind: process
    label: "Validated typed rustls client/server configuration"
  adopt:
    kind: terminal
    label: "Tape consumes shared adapter; no local unsafe environment test surface"
edges:
  - { from: env, to: resolve }
  - { from: resolve, to: plain, label: "none configured" }
  - { from: resolve, to: validate, label: "material configured" }
  - { from: validate, to: typed, label: "valid" }
  - { from: validate, to: plain, label: "invalid: fail startup" }
  - { from: typed, to: adopt }
---
flowchart TD
    env([service prefix]) --> resolve[raft-host PeerTlsConfig resolves service-tls]
    resolve -->|none| plain([explicit plaintext h2c])
    resolve -->|configured| validate{complete and valid material?}
    validate -->|valid| typed[typed rustls client/server configuration]
    validate -->|invalid| fail([startup error])
    typed --> adopt([Tape consumes shared adapter])
```

`raft-host` owns the shared peer-transport configuration boundary. It delegates
PEM parsing, client/server config construction, and mTLS policy validation to
`service-tls`, preserving the existing all-or-nothing material contract. Tape
will consume this typed adapter instead of maintaining a local wrapper and
unsafe environment-mutating tests.

This slice deliberately does not claim TLS termination on the current h2c peer
router: that transport seam is absent today. A configured peer policy is
validated and represented by the host; actual acceptor/connector wiring remains
a later transport change.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/raft-host/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: libs/raft-host/src/peer_tls.rs
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/raft-host/tech-design/semantic/source/libs-raft-host-src-lib-rs.md
    action: modify
    section: logic
    impl_mode: codegen
  - path: libs/raft-host/tests/peer_tls.rs
    action: create
    section: unit-test
    impl_mode: hand-written
  - path: apps/tape/Cargo.toml
    action: modify
    section: changes
    impl_mode: hand-written
  - path: apps/tape/src/peer_tls.rs
    action: modify
    section: changes
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: raft-host-peer-tls-configuration-verification
requirements:
  shared_config_contract:
    id: R1
    text: "raft-host exposes a prefix-scoped peer TLS configuration that preserves service-tls validation and rustls builder behavior."
    kind: functional
    risk: medium
    verify: cargo test -p raft-host --test peer_tls
  tape_adapter_migration:
    id: R2
    text: "Tape consumes the raft-host peer TLS adapter without retaining local unsafe environment-mutating test code."
    kind: regression
    risk: medium
    verify: cargo test -p tape --lib
---
flowchart TD
    r1[R1 shared config contract] --> cargo_test_p_raft_host_test_peer_tls[cargo test -p raft-host --test peer_tls]
    r2[R2 tape adapter migration] --> cargo_test_p_tape_lib[cargo test -p tape --lib]
```
