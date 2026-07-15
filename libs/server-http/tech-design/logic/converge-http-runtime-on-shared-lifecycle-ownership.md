---
id: '1776'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-http-runtime-lifecycle-ownership
entry: start
nodes:
  start: { kind: start, label: "HTTP runtime request" }
  lifecycle: { kind: process, label: "server-lifecycle owns bind, drain/readiness, shutdown, budgets, and connection metric events" }
  tcp: { kind: process, label: "server-tcp owns listener admission, per-connection supervision, metric callbacks, and bounded drain" }
  http: { kind: process, label: "server-http owns the only HTTP listener facade and maps each accepted TCP stream to HTTP/1.1+h2c" }
  transport: { kind: process, label: "transport-h2c owns outbound clients/pools plus per-connection h2c protocol machinery, never a listener loop" }
  service: { kind: process, label: "service-http owns probes, OpenAPI/docs, HTTP errors, and request-policy adapters" }
  apps: { kind: terminal, label: "Lumen, Tape, Keep, Relay, Courier, and Pgpool compose one lifecycle contract without route changes" }
  shutdown: { kind: terminal, label: "One shutdown signal flips shared drain state, stops admission, and drains supervised connections" }
edges:
  - { from: start, to: lifecycle }
  - { from: lifecycle, to: tcp }
  - { from: tcp, to: http }
  - { from: http, to: transport }
  - { from: http, to: service }
  - { from: service, to: apps }
  - { from: lifecycle, to: shutdown }
  - { from: tcp, to: shutdown }
---
flowchart TD
  start([HTTP runtime request]) --> lifecycle[server-lifecycle: bind, drain/readiness, shutdown, budgets, metric events]
  lifecycle --> tcp[server-tcp: accept, admission, supervision, metric callbacks, bounded drain]
  tcp --> http[server-http: sole listener facade and HTTP connection dispatch]
  http --> transport[transport-h2c: outbound pools plus per-connection HTTP/1.1+h2c protocol]
  http --> service[service-http: probes, OpenAPI/docs, errors, request policy]
  service --> apps([service and tool consumers preserve public routes])
  lifecycle --> shutdown([shared drain state stops admission])
  tcp --> shutdown
```
