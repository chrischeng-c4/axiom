---
id: '1643'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-peer-mtls-transport
entry: configure
nodes:
  configure: { kind: start, label: "Load peer cert, private key, CA roots, and required mTLS posture" }
  build: { kind: decision, label: "Both rustls server/client configs and HTTPS client build?" }
  preserve: { kind: process, label: "Reject malformed replacement and retain last-known-good generation" }
  publish: { kind: process, label: "Atomically publish client, acceptor, ALPN h2, and generation" }
  accept: { kind: start, label: "Accept peer TCP connection with current server config" }
  client_identity: { kind: decision, label: "Client chain trusted and client certificate presented?" }
  reject_client: { kind: terminal, label: "Reject before HTTP or Raft dispatch" }
  serve: { kind: process, label: "Serve authenticated TLS stream through generic HTTP/2 connection seam" }
  dial: { kind: start, label: "Dial https peer using current client config and expected DNS identity" }
  server_identity: { kind: decision, label: "Server chain trusted, valid, and identity matched?" }
  reject_server: { kind: terminal, label: "Fail closed before sending Raft RPC" }
  dispatch: { kind: terminal, label: "Dispatch Raft traffic over mutually authenticated HTTP/2" }
edges:
  - { from: configure, to: build }
  - { from: build, to: preserve, label: "no" }
  - { from: build, to: publish, label: "yes" }
  - { from: accept, to: client_identity }
  - { from: client_identity, to: reject_client, label: "no" }
  - { from: client_identity, to: serve, label: "yes" }
  - { from: serve, to: dispatch }
  - { from: dial, to: server_identity }
  - { from: server_identity, to: reject_server, label: "no" }
  - { from: server_identity, to: dispatch, label: "yes" }
---
flowchart TD
    configure([Load material]) --> build{Both configs valid?}
    build -->|no| preserve[Keep last-known-good]
    build -->|yes| publish[Atomic generation swap]
    accept([Accept peer]) --> client_identity{Trusted client cert?}
    client_identity -->|no| reject_client([Reject before HTTP])
    client_identity -->|yes| serve[Serve TLS stream as HTTP/2]
    dial([Dial HTTPS peer]) --> server_identity{Trusted matching server?}
    server_identity -->|no| reject_server([Fail closed])
    server_identity -->|yes| dispatch([Raft RPC])
    serve --> dispatch
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/raft-runtime/src/peer_transport.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Reloadable last-known-good mTLS client/server snapshot, raw TLS connect/accept seams, HTTPS client, and TLS HTTP/2 listener."
  - path: libs/raft-runtime/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose the peer mTLS transport contract."
  - path: libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-lib-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep the raft-runtime public semantic source aligned."
  - path: libs/raft-runtime/src/host.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add spawn_with_peer_transport and route outgoing Raft HTTPS requests through the reloadable shared client."
  - path: libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-host-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep the host semantic source aligned with TLS transport adoption."
  - path: libs/raft-runtime/src/cluster.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Support caller-selected http or https peer URL projection while preserving the existing http default."
  - path: libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-cluster-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep cluster topology semantic source aligned."
  - path: libs/raft-runtime/Cargo.toml
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Add peer-tls, rustls, tokio-rustls, transport server feature, and rcgen test dependencies."
  - path: libs/raft-runtime/README.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Publish the shared peer mTLS capability rooted at #1643."
  - path: libs/transport-h2c/src/server.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Generalize the per-connection HTTP/2 server from TcpStream to any Tokio AsyncRead/AsyncWrite stream, including rustls."
  - path: libs/transport-h2c/tech-design/semantic/source/libs-transport-h2c-src-server-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep the generic server I/O semantic source aligned."
  - path: libs/peer-tls/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Advertise h2 ALPN from both peer rustls configs."
  - path: libs/peer-tls/tech-design/semantic/source/libs-peer-tls-src-lib-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep peer TLS material semantic source aligned."
  - path: apps/lumen/src/tls.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose Lumen's thin conversion into the shared raft-runtime PeerTransport."
  - path: apps/lumen/tech-design/semantic/source/apps-lumen-src-tls-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep Lumen TLS semantic source aligned."
  - path: apps/tape/src/peer_tls.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose Tape's identical thin conversion into the shared runtime transport."
  - path: libs/raft-runtime/tests/peer_mtls.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Ephemeral CA/certificate tests for mutual success, hostname mismatch, untrusted client rejection, and explicit reload preservation."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: shared-peer-mtls-transport-verification
requirements:
  app_adapters:
    id: R5
    text: "Lumen and Tape peer TLS adapters construct the same raft_runtime::PeerTransport without app-local acceptor or connector implementations."
    kind: integration
    risk: medium
    verify: cargo test -p lumen tls && cargo test -p tape peer_tls
  compatibility:
    id: R4
    text: "Existing RaftHost::spawn and ClusterTopology::from_env retain plain h2c behavior; opt-in callers use spawn_with_peer_transport and https topology."
    kind: compatibility
    risk: high
    verify: cargo test -p raft-runtime
  invalid_identity:
    id: R2
    text: "Hostname mismatch and an untrusted client certificate fail during the TLS handshake before any Raft HTTP route is dispatched."
    kind: security
    risk: high
    verify: cargo test -p raft-runtime --test peer_mtls
  mutual_auth:
    id: R1
    text: "A peer connection succeeds only when the server identity matches and both certificates chain to the configured CA; the accepted TLS stream can serve the shared HTTP/2 transport."
    kind: security
    risk: high
    verify: cargo test -p raft-runtime --test peer_mtls
  reload:
    id: R3
    text: "A valid explicit reload atomically advances the transport generation for new dials/accepts, while malformed replacement material preserves the last-known-good generation."
    kind: regression
    risk: high
    verify: cargo test -p raft-runtime --test peer_mtls
---
flowchart TD
    r1[R1 mutual auth] --> cargo_test_p_raft_runtime_test_peer_mtls[cargo test -p raft-runtime --test peer_mtls]
    r2[R2 invalid identity] --> cargo_test_p_raft_runtime_test_peer_mtls
    r3[R3 reload] --> cargo_test_p_raft_runtime_test_peer_mtls
    r4[R4 compatibility] --> cargo_test_p_raft_runtime[cargo test -p raft-runtime]
    r5[R5 app adapters] --> cargo_test_p_lumen_tls_cargo_test_p_tape_peer_tls[cargo test -p lumen tls && cargo test -p tape peer_tls]
```
