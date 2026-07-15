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
