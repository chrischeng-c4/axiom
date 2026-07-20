---
id: '2146'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: beam-test-gate-execution-policy
entry: start
nodes:
  start: { kind: start, label: "cargo test -p beam" }
  detect: { kind: process, label: "Check if BEAM_REQUIRED_GATES environment variable is set" }
  service_test: { kind: process, label: "Run service integration test" }
  gpu_test: { kind: process, label: "Run GPU vs CPU parity test" }
  service_bind: { kind: decision, label: "Can bind local port 127.0.0.1:0?" }
  gpu_adapter: { kind: decision, label: "Is GPU adapter present?" }
  service_skip: { kind: terminal, label: "Print skip warning, return Ok" }
  service_fail: { kind: terminal, label: "Panic (exit code non-zero)" }
  gpu_skip: { kind: terminal, label: "Print skip warning, return Ok" }
  gpu_fail: { kind: terminal, label: "Panic (exit code non-zero)" }
  emit_service_ev: { kind: process, label: "Emit JSON evidence: category=service, transport=h2c, assertions=N" }
  emit_gpu_ev: { kind: process, label: "Emit JSON evidence: category=gpu, adapter=Name, assertions=M" }
  test_pass: { kind: terminal, label: "Test suite passes (exit code 0)" }
edges:
  - { from: start, to: detect }
  - { from: detect, to: service_test }
  - { from: detect, to: gpu_test }
  - { from: service_test, to: service_bind }
  - { from: service_bind, to: emit_service_ev, label: "Yes" }
  - { from: service_bind, to: service_fail, label: "No (required)" }
  - { from: service_bind, to: service_skip, label: "No (optional)" }
  - { from: gpu_test, to: gpu_adapter }
  - { from: gpu_adapter, to: emit_gpu_ev, label: "Yes" }
  - { from: gpu_adapter, to: gpu_fail, label: "No (required)" }
  - { from: gpu_adapter, to: gpu_skip, label: "No (optional)" }
  - { from: emit_service_ev, to: test_pass }
  - { from: emit_gpu_ev, to: test_pass }
---
flowchart TD
    start([cargo test -p beam]) --> detect[Check BEAM_REQUIRED_GATES env]
    detect --> service_test[Run service integration test]
    detect --> gpu_test[Run GPU vs CPU parity test]
    service_test --> service_bind{Can bind port?}
    service_bind -->|Yes| emit_service_ev[Emit JSON evidence: service, h2c, N assertions]
    service_bind -->|No & Required| service_fail[Panic & fail gate]
    service_bind -->|No & Optional| service_skip[Print skip, return success]
    gpu_test --> gpu_adapter{GPU adapter present?}
    gpu_adapter -->|Yes| emit_gpu_ev[Emit JSON evidence: GPU, adapter info, M assertions]
    gpu_adapter -->|No & Required| gpu_fail[Panic & fail gate]
    gpu_adapter -->|No & Optional| gpu_skip[Print skip, return success]
    emit_service_ev --> test_pass([Pass])
    emit_gpu_ev --> test_pass
```
