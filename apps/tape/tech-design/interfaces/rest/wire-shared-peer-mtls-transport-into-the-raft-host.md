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
