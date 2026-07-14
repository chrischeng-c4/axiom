---
id: '1704'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: raft-host-peer-tls-contract
entry: resolve
nodes:
  resolve:
    kind: start
    label: "PeerTlsConfig::from_env(prefix)"
  absent:
    kind: terminal
    label: "Ok(None): explicit plaintext peer mode"
  invalid:
    kind: terminal
    label: "Err: partial, missing, or unusable TLS material fails startup"
  config:
    kind: process
    label: "Ok(Some): cert/key/CA paths and mTLS required policy"
  builders:
    kind: terminal
    label: "rustls_server_config and rustls_client_config delegate to service-tls"
edges:
  - { from: resolve, to: absent, label: "no variables set" }
  - { from: resolve, to: invalid, label: "invalid material" }
  - { from: resolve, to: config, label: "valid material" }
  - { from: config, to: builders }
---
flowchart TD
    resolve([from_env prefix]) --> absent([None: plaintext h2c])
    resolve --> invalid([Err: fail startup])
    resolve --> config[Some: cert key CA + required policy]
    config --> builders([rustls configs via service-tls])
```

The public contract is `raft_host::PeerTlsConfig`: `from_env(prefix)` returns
`None` only when all variables are absent, returns an error for partial or
unusable material, and exposes the validated paths and mTLS-required flag.
Its rustls builders delegate to `service-tls`; no private-key parsing or unsafe
operation is duplicated in service applications.

The contract explicitly does not change the Raft host's current h2c router or
peer dialing protocol. Consumers may validate and stage material now; TLS
termination requires a later acceptor/connector seam.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/raft-host/src/peer_tls.rs
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/raft-host/tech-design/semantic/source/libs-raft-host-src-lib-rs.md
    action: modify
    section: logic
    impl_mode: codegen
  - path: apps/tape/src/peer_tls.rs
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/tape/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: raft-host-peer-tls-contract-verification
requirements:
  all_or_nothing_material:
    id: R1
    text: "The shared peer TLS adapter distinguishes absent material, rejects partial or invalid material, and builds both rustls configuration directions from valid PEM material."
    kind: functional
    risk: medium
    verify: cargo test -p raft-host --test peer_tls
  application_compatibility:
    id: R2
    text: "Tape compiles against the raft-host adapter without direct service-tls or rustls configuration ownership."
    kind: regression
    risk: medium
    verify: cargo test -p tape --lib
---
flowchart TD
    r1[R1 all or nothing material] --> cargo_test_p_raft_host_test_peer_tls[cargo test -p raft-host --test peer_tls]
    r2[R2 application compatibility] --> cargo_test_p_tape_lib[cargo test -p tape --lib]
```
