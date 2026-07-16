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
  load_peer_policy: { kind: start, label: Load TAPE_PEER_TLS_CERT, TAPE_PEER_TLS_KEY, TAPE_PEER_TLS_CA, and TAPE_PEER_MTLS through the Tape peer-tls adapter }
  config_valid: { kind: decision, label: Is peer TLS absent, or complete with mTLS required }
  reject: { kind: terminal, label: Reject partial material, missing required material, unreadable PEM, or a non-mTLS transport before binding listeners }
  plain_topology: { kind: process, label: Build existing http ClusterTopology using the public serve port }
  plain_host: { kind: process, label: Spawn the existing h2c RaftHost and retain the Raft router on the public app }
  secure_transport: { kind: process, label: Build the shared PeerTransport and an https ClusterTopology using TAPE_RAFT_PORT }
  secure_host: { kind: process, label: Spawn TapeRaft through RaftHost::spawn_with_peer_transport }
  secure_listener: { kind: process, label: Bind the dedicated peer port and serve only TapeRaft::router through PeerTransport::serve }
  public_listener: { kind: process, label: Serve the public HTTP h2c app; exclude Raft routes when the secure peer listener exists }
  drain: { kind: terminal, label: Public drain signals peer listener shutdown and waits for its result }
edges:
  - { from: load_peer_policy, to: config_valid }
  - { from: config_valid, to: reject, label: invalid }
  - { from: config_valid, to: plain_topology, label: absent }
  - { from: config_valid, to: secure_transport, label: complete and required }
  - { from: plain_topology, to: plain_host }
  - { from: plain_host, to: public_listener }
  - { from: secure_transport, to: secure_host }
  - { from: secure_host, to: secure_listener }
  - { from: secure_listener, to: public_listener }
  - { from: public_listener, to: drain }
---
flowchart TD
    load_peer_policy([load optional peer mTLS policy]) --> config_valid{absent or complete mTLS?}
    config_valid -->|invalid| reject([fail before listen])
    config_valid -->|absent| plain_topology[http public-port topology]
    plain_topology --> plain_host[h2c RaftHost and public peer routes]
    config_valid -->|complete and required| secure_transport[shared PeerTransport plus https raft-port topology]
    secure_transport --> secure_host[spawn host with peer transport]
    secure_host --> secure_listener[dedicated authenticated Raft listener]
    plain_host --> public_listener[public h2c app]
    secure_listener --> public_listener[public app excludes Raft routes]
    public_listener --> drain([drain public and peer listeners])
```

Tape owns only selection and lifecycle at the service boundary. `peer-tls` owns environment validation, PEM parsing, client/server Rustls configuration, and the required-mTLS posture. `raft-runtime` owns HTTPS peer dialing, mutual-auth handshakes, Raft RPC routing, and reloadable transport state. Tape must not copy either implementation.

No configured peer TLS retains the existing public-port h2c topology and router composition. A complete configuration must also set `TAPE_PEER_MTLS=on`; otherwise `PeerTransport::from_config` fails before either listener starts. In secure mode Tape derives peer URLs with `ClusterTopology::from_env_with_scheme(..., "https")`, uses the dedicated `--raft-port` / `TAPE_RAFT_PORT` (7138 by default), creates TapeRaft with the shared transport, and hands only `TapeRaft::router()` to `PeerTransport::serve`. The public `service-http` app continues to serve data, probes, and HTTP h2c but deliberately excludes `/raft/*` and `/raftz`.

The operator follows Lumen's port separation: its StatefulSet declares named `http` and `raft` ports, its headless Service resolves both ports for pod peers, and it injects `TAPE_RAFT_PORT`. It neither parses certificates nor introduces a Tape-specific certificate controller; the established `TAPE_PEER_*` environment contract remains the common integration seam.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/bin/tape.rs
    action: modify
    section: serve-peer-transport
    impl_mode: hand-written
    anchor: resolve_journal_store
    description: Select the existing h2c topology when peer TLS is absent, otherwise build the shared required-mTLS transport, use an https topology on TAPE_RAFT_PORT, run the dedicated peer listener, and join it to public drain.
  - path: apps/tape/src/raft.rs
    action: modify
    section: raft-transport-adapter
    impl_mode: hand-written
    anchor: impl TapeRaft
    description: Add additive TapeRaft constructors that delegate secure hosts to RaftHost::spawn_with_peer_transport while preserving existing h2c constructors.
  - path: apps/tape/src/server.rs
    action: modify
    section: public-peer-route-isolation
    impl_mode: hand-written
    anchor: router
    description: Make public router composition explicitly choose whether Raft peer routes are merged, retaining the current default for h2c and excluding them for the secure peer listener.
  - path: apps/tape/src/operator/render.rs
    action: modify
    section: kubernetes-peer-service
    impl_mode: hand-written
    anchor: render
    description: Include the named raft port in the headless service through the shared multi-port helper while retaining the client service on the public data-plane port only.
  - path: apps/tape/src/operator/render.rs
    action: modify
    section: kubernetes-peer-workload
    impl_mode: hand-written
    anchor: statefulset
    description: Declare the named 7138 raft container port and inject TAPE_RAFT_PORT into the StatefulSet runtime environment.
  - path: apps/tape/tests/raft_peer_mtls.rs
    action: create
    section: peer-transport-integration-test
    impl_mode: hand-written
    description: Exercise real TapeRaft replication over trusted shared mTLS transports and prove an untrusted certificate is rejected before the Raft router handles a request.
  - path: apps/tape/tests/operator.rs
    action: modify
    section: kubernetes-peer-port-test
    impl_mode: hand-written
    anchor: render_emits_expected_child_objects
    description: Assert the StatefulSet, headless service, and runtime environment carry the Raft port while the client Service remains public-data-plane only.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: 1805-verification
requirements:
  compatibility:
    id: R3
    text: "With no configured peer TLS, Tape preserves the existing public-port h2c Raft topology, forwarding, election, replication, and failover behavior."
    kind: regression
    risk: medium
    verify: raft_cluster::three_node_group_elects_replicates_forwards_and_fails_over
  kubernetes_port:
    id: R5
    text: "The Tape operator renders named http and raft ports on the StatefulSet and headless Service, injects TAPE_RAFT_PORT, and keeps the client Service on 7137 only."
    kind: integration
    risk: medium
    verify: operator::render_emits_expected_child_objects
  peer_rejection:
    id: R2
    text: "A certificate not chained to Tape's configured peer CA is rejected before it can reach the Tape Raft router."
    kind: security
    risk: high
    verify: raft_peer_mtls::untrusted_peer_is_rejected_before_tape_raft_router
  public_isolation:
    id: R4
    text: "Secure peer mode excludes Raft peer routes from Tape's public application router while the dedicated mTLS listener serves them."
    kind: security
    risk: high
    verify: server::secure_peer_mode_does_not_expose_raft_routes_on_public_router
  secure_replication:
    id: R1
    text: "Complete required Tape peer TLS configuration creates the shared PeerTransport, projects https peer URLs on TAPE_RAFT_PORT, and lets trusted TapeRaft peers elect and replicate over that listener."
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
