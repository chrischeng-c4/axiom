---
id: '1805'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-peer-mtls-adoption
entry: load_peer_policy
nodes:
  load_peer_policy: { kind: start, label: Load optional Tape peer TLS configuration }
  config_valid: { kind: decision, label: Is peer mTLS configured completely and required }
  reject: { kind: terminal, label: Fail before any listener starts }
  plain_topology: { kind: process, label: Project existing http peer topology }
  plain_host: { kind: process, label: Spawn RaftHost with h2c and merge peer router onto public service app }
  secure_transport: { kind: process, label: Build shared PeerTransport and project https peer topology }
  secure_host: { kind: process, label: Spawn RaftHost with shared authenticated transport }
  secure_listener: { kind: process, label: Bind dedicated raft peer listener and serve only host router through PeerTransport }
  public_listener: { kind: process, label: Serve public HTTP h2c app without secure peer router }
  drain: { kind: terminal, label: Drain public server signal peer listener shutdown and await it }
edges:
  - { from: load_peer_policy, to: config_valid }
  - { from: config_valid, to: reject, label: invalid or partial }
  - { from: config_valid, to: plain_topology, label: absent }
  - { from: config_valid, to: secure_transport, label: configured }
  - { from: plain_topology, to: plain_host }
  - { from: plain_host, to: public_listener }
  - { from: secure_transport, to: secure_host }
  - { from: secure_host, to: secure_listener }
  - { from: secure_listener, to: public_listener }
  - { from: public_listener, to: drain }
---
flowchart TD
    load_peer_policy([Load peer TLS policy]) --> config_valid{Configured and required?}
    config_valid -->|invalid or partial| reject([Fail before listen])
    config_valid -->|absent| plain_topology[http topology]
    plain_topology --> plain_host[spawn h2c host and merge peer router]
    config_valid -->|configured| secure_transport[build shared PeerTransport and https topology]
    secure_transport --> secure_host[spawn authenticated host]
    secure_host --> secure_listener[serve host router on dedicated TLS listener]
    plain_host --> public_listener[serve public h2c app]
    secure_listener --> public_listener
    public_listener --> drain([drain public and peer listeners together])
```

The Tape adapter owns only configuration selection, topology scheme and listener lifecycle. `peer-tls` continues to own certificate material and `raft-runtime` continues to own mTLS handshakes, HTTPS peer RPCs and Raft routing. With no configured peer TLS, the current single-port h2c behavior remains byte-for-byte compatible. With configured peer mTLS, the public service port never exposes the Raft router; only the dedicated authenticated listener does.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/bin/tape.rs
    action: modify
    section: serve-peer-transport
    impl_mode: hand-written
  - path: apps/tape/src/raft.rs
    action: modify
    section: raft-transport-adapter
    impl_mode: hand-written
  - path: apps/tape/src/server.rs
    action: modify
    section: public-peer-route-isolation
    impl_mode: hand-written
  - path: apps/tape/src/operator/render.rs
    action: modify
    section: kubernetes-peer-port
    impl_mode: hand-written
  - path: apps/tape/Cargo.toml
    action: modify
    section: peer-transport-integration-test-dependencies
    impl_mode: hand-written
  - path: apps/tape/tests/raft_peer_mtls.rs
    action: create
    section: peer-transport-integration-test
    impl_mode: hand-written
  - path: apps/tape/tests/operator.rs
    action: modify
    section: kubernetes-peer-port-test
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: 1805-verification
requirements:
  compatibility:
    id: R3
    text: "With no peer TLS configuration, Tape keeps the existing public-port h2c Raft topology and peer route behavior."
    kind: regression
    risk: medium
    verify: raft_cluster::three_node_group_elects_replicates_forwards_and_fails_over
  kubernetes_port:
    id: R5
    text: "The Tape operator exposes a dedicated Raft container and headless-service port and injects TAPE_RAFT_PORT while preserving the client service on 7137."
    kind: integration
    risk: medium
    verify: operator::render_emits_expected_child_objects
  peer_rejection:
    id: R2
    text: "A client whose certificate is not trusted by the Tape peer CA cannot dispatch a Raft request to the authenticated listener."
    kind: security
    risk: high
    verify: raft_peer_mtls::untrusted_peer_is_rejected_before_tape_raft_router
  public_isolation:
    id: R4
    text: "When peer TLS is active, Tape's public application router excludes Raft peer routes while the dedicated peer listener owns them."
    kind: security
    risk: high
    verify: server::secure_peer_mode_does_not_expose_raft_routes_on_public_router
  secure_replication:
    id: R1
    text: "When complete Tape peer TLS material is configured, Tape constructs the shared PeerTransport, projects Raft peers as https URLs, and spawns the Raft host with that transport."
    kind: functional
    risk: high
    verify: raft_peer_mtls::trusted_tape_raft_peers_replicate_over_mtls
---
flowchart TD
    r1[R1 secure replication] --> raft_peer_mtls_trusted_tape_raft_peers_replicate_over_mtls[raft_peer_mtls::trusted_tape_raft_peers_replicate_over_mtls]
    r2[R2 peer rejection] --> raft_peer_mtls_untrusted_peer_is_rejected_before_tape_raft_router[raft_peer_mtls::untrusted_peer_is_rejected_before_tape_raft_router]
    r3[R3 compatibility] --> raft_cluster_three_node_group_elects_replicates_forwards_and_fails_over[raft_cluster::three_node_group_elects_replicates_forwards_and_fails_over]
    r4[R4 public isolation] --> server_secure_peer_mode_does_not_expose_raft_routes_on_public_router[server::secure_peer_mode_does_not_expose_raft_routes_on_public_router]
    r5[R5 kubernetes port] --> operator_render_emits_expected_child_objects[operator::render_emits_expected_child_objects]
```
