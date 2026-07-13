---
id: "1526"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-microvm-published-port-readiness
entry: start
nodes:
  start: { kind: start, label: "vat run or vat compose starts an image-backed service" }
  runtime: { kind: decision, label: "resolved service runtime" }
  existing: { kind: process, label: "native and Docker retain their existing preparation and readiness paths unchanged" }
  microvm_start: { kind: process, label: "start the VAT-owned Apple container service and persist a provisional Starting service record" }
  readiness_kind: { kind: decision, label: "configured readiness contract" }
  tcp_only: { kind: process, label: "perform the existing host TCP readiness check only when no application-level contract exists" }
  http_round_trip: { kind: process, label: "perform the configured ready_http request through the published host endpoint, not the guest address" }
  host_ok: { kind: decision, label: "published endpoint completes the required host-side round trip" }
  ready: { kind: process, label: "persist Ready evidence with host endpoint and only then expose the service as ready" }
  diagnose: { kind: process, label: "collect service id, host endpoint, known guest endpoint, container runtime/version, readiness error, and owned container name" }
  teardown: { kind: process, label: "remove only the VAT-owned Apple container and persist terminal Failed state" }
  fail: { kind: terminal, label: "return a nonzero actionable published-endpoint failure; never report Ready" }
  compose_start: { kind: process, label: "vat compose up --detach creates or retains only a Starting registry record while the background vat run gathers readiness evidence" }
  compose_poll: { kind: decision, label: "persisted service/run state" }
  compose_starting: { kind: terminal, label: "report status starting with vat id while evidence is still pending" }
  compose_ready: { kind: terminal, label: "report ready only after every owned service persisted Ready evidence" }
  compose_failed: { kind: process, label: "remove stale compose running state after terminal startup failure" }
  compose_fail: { kind: terminal, label: "return the terminal startup failure with its actionable diagnostics" }
  success: { kind: terminal, label: "service lifecycle continues with verified host-published readiness" }
edges:
  - { from: start, to: runtime }
  - { from: runtime, to: existing, label: "native or Docker" }
  - { from: runtime, to: microvm_start, label: "MicroVm" }
  - { from: existing, to: success }
  - { from: microvm_start, to: readiness_kind }
  - { from: readiness_kind, to: tcp_only, label: "no ready_http" }
  - { from: readiness_kind, to: http_round_trip, label: "ready_http configured" }
  - { from: tcp_only, to: host_ok }
  - { from: http_round_trip, to: host_ok }
  - { from: host_ok, to: ready, label: "yes" }
  - { from: host_ok, to: diagnose, label: "no, reset, timeout, or bad response" }
  - { from: ready, to: success }
  - { from: diagnose, to: teardown }
  - { from: teardown, to: fail }
  - { from: compose_start, to: compose_poll }
  - { from: compose_poll, to: compose_starting, label: "Starting" }
  - { from: compose_poll, to: compose_ready, label: "all Ready" }
  - { from: compose_poll, to: compose_failed, label: "terminal Failed" }
  - { from: compose_failed, to: compose_fail }
---
```
