---
id: "1526"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-microvm-host-endpoint-contract
entry: start
nodes:
  start: { kind: start, label: "prepare_service receives an image-backed service" }
  route: { kind: decision, label: "ServiceRuntime" }
  unchanged: { kind: process, label: "Auto, Docker, and Native retain docker_ready_probe and existing lifecycle behavior" }
  prepare: { kind: process, label: "prepare_microvm_service creates a VAT-owned container name, loopback mapping, and a MicroVm-specific readiness probe" }
  probe_kind: { kind: decision, label: "explicit ready_http present" }
  http: { kind: process, label: "substitute the allocated host port and require an HTTP 2xx or 3xx round trip through 127.0.0.1:published-port" }
  tcp_usable: { kind: process, label: "require a MicroVm-only TCP usability probe: connect, then distinguish immediate EOF or ECONNRESET from an open idle protocol connection" }
  start_service: { kind: process, label: "start_service persists Running evidence, including owned microvm_name, host, port, and log paths" }
  wait: { kind: decision, label: "wait_for_services probe outcome before timeout" }
  ready: { kind: process, label: "persist ProcessStatus::Ready and ready duration only after the host endpoint contract succeeds" }
  observe_failure: { kind: process, label: "persist Failed or Timeout; retain the last readiness error and collect best-effort container --version and container inspect evidence" }
  cleanup: { kind: process, label: "stop_services force-removes exactly service.microvm_name and persists the terminal service evidence" }
  error: { kind: terminal, label: "return nonzero microvm_published_endpoint_unusable with service id, host endpoint, known guest endpoint or unavailable, runtime evidence, and a runnable inspect/logs remediation" }
  detach_start: { kind: process, label: "compose up --detach writes status starting, spawns vat run, and records vat_id when it appears" }
  reconcile: { kind: decision, label: "load persisted vat test_run service states" }
  still_starting: { kind: terminal, label: "emit status starting when vat id or every Ready record is not yet available" }
  compose_ready: { kind: terminal, label: "persist and emit status ready only when every compose service is Ready" }
  compose_failed: { kind: process, label: "remove the compose registry running record after a terminal startup failure while preserving vat logs/state for diagnosis" }
  compose_error: { kind: terminal, label: "return nonzero terminal startup failure rather than status started" }
  success: { kind: terminal, label: "continue runner or compose lifecycle with verified service evidence" }
edges:
  - { from: start, to: route }
  - { from: route, to: unchanged, label: "not MicroVm" }
  - { from: route, to: prepare, label: "MicroVm" }
  - { from: unchanged, to: success }
  - { from: prepare, to: probe_kind }
  - { from: probe_kind, to: http, label: "yes" }
  - { from: probe_kind, to: tcp_usable, label: "no" }
  - { from: http, to: start_service }
  - { from: tcp_usable, to: start_service }
  - { from: start_service, to: wait }
  - { from: wait, to: ready, label: "success" }
  - { from: wait, to: observe_failure, label: "reset, EOF, timeout, or bad response" }
  - { from: ready, to: success }
  - { from: observe_failure, to: cleanup }
  - { from: cleanup, to: error }
  - { from: detach_start, to: reconcile }
  - { from: reconcile, to: still_starting, label: "pending" }
  - { from: reconcile, to: compose_ready, label: "all Ready" }
  - { from: reconcile, to: compose_failed, label: "terminal failure" }
  - { from: compose_failed, to: compose_error }
---
```
