---
id: '1646'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-shared-stateful-foundation-adoption
entry: bootstrap
nodes:
  bootstrap: { kind: start, label: "lumen serve builds shared service identity and observability config" }
  tracing: { kind: process, label: "service-http initializes logs and optional OTLP traces; Lumen keeps only domain metrics" }
  raft: { kind: decision, label: "Raft mode enabled?" }
  tls: { kind: decision, label: "LUMEN_PEER TLS material configured?" }
  https_topology: { kind: process, label: "Build https peer topology on dedicated raft port" }
  mtls_host: { kind: process, label: "Spawn RaftHost with shared PeerTransport" }
  peer_listener: { kind: process, label: "Serve only Raft router on authenticated peer listener" }
  h2c_host: { kind: process, label: "Preserve existing plain h2c host and public-port router for unconfigured local compatibility" }
  app: { kind: process, label: "Compose shared auth, admission, probes, and Lumen domain routes" }
  workload: { kind: process, label: "Operator projects public and raft ports through shared StatefulSet primitives" }
  verify: { kind: terminal, label: "Run app adoption gates for auth, admission, TLS, workload, Rig, and OTLP ownership" }
edges:
  - { from: bootstrap, to: tracing }
  - { from: tracing, to: raft }
  - { from: raft, to: app, label: "no" }
  - { from: raft, to: tls, label: "yes" }
  - { from: tls, to: https_topology, label: "configured" }
  - { from: https_topology, to: mtls_host }
  - { from: mtls_host, to: peer_listener }
  - { from: peer_listener, to: app }
  - { from: tls, to: h2c_host, label: "not configured" }
  - { from: h2c_host, to: app }
  - { from: app, to: workload }
  - { from: workload, to: verify }
---
flowchart TD
    bootstrap([lumen serve builds shared service identity and observability config]) --> tracing[service-http initializes logs and optional OTLP traces; Lumen keeps only domain metrics]
    tracing --> raft{Raft mode enabled?}
    raft -->|no| app[Compose shared auth admission probes and Lumen domain routes]
    raft -->|yes| tls{LUMEN_PEER TLS material configured?}
    tls -->|configured| https_topology[Build https peer topology on dedicated raft port]
    https_topology --> mtls_host[Spawn RaftHost with shared PeerTransport]
    mtls_host --> peer_listener[Serve only Raft router on authenticated peer listener]
    peer_listener --> app
    tls -->|not configured| h2c_host[Preserve existing plain h2c host and public-port router for local compatibility]
    h2c_host --> app
    app --> workload[Operator projects public and raft ports through shared StatefulSet primitives]
    workload --> verify([Run app adoption gates for auth admission TLS workload Rig and OTLP ownership])
```
