---
id: '2146'
summary: >
  Make Beam's configured service and GPU test gates execution-honest by
  verifying actual networking and GPU capabilities rather than returning
  success upon encountering missing infrastructure.
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "beam-test-gate-execution-policy"
    coverage: full
    rationale: >
      This TD establishes the execution-honest test policy for Beam, requiring
      optional local skips to be separate from required production verification.
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: beam-test-gate-evidence-contract
entry: start
nodes:
  start: { kind: start, label: "cargo test -p beam" }
  env_check: { kind: process, label: "Check std::env::var('BEAM_REQUIRED_GATES')" }
  run_service: { kind: process, label: "Run service_end_to_end" }
  run_gpu: { kind: process, label: "Run gpu_matches_cpu_oracle" }
  service_bind_ok: { kind: decision, label: "TcpListener::bind ok?" }
  gpu_ok: { kind: decision, label: "GpuContext::new ok?" }
  service_skip: { kind: terminal, label: "Print skip message & return success (if BEAM_REQUIRED_GATES is unset)" }
  gpu_skip: { kind: terminal, label: "Print skip message & return success (if BEAM_REQUIRED_GATES is unset)" }
  service_fail: { kind: terminal, label: "Panic (if BEAM_REQUIRED_GATES is set)" }
  gpu_fail: { kind: terminal, label: "Panic (if BEAM_REQUIRED_GATES is set)" }
  service_evidence: { kind: process, label: "Emit JSON: {category: 'service', transport: 'h2c', assertions: count}" }
  gpu_evidence: { kind: process, label: "Emit JSON: {category: 'gpu', adapter: name, backend: backend, assertions: count}" }
  done: { kind: terminal, label: "All assertions verified successfully" }
edges:
  - { from: start, to: env_check }
  - { from: env_check, to: run_service }
  - { from: env_check, to: run_gpu }
  - { from: run_service, to: service_bind_ok }
  - { from: service_bind_ok, to: service_evidence, label: "Yes" }
  - { from: service_bind_ok, to: service_fail, label: "No & BEAM_REQUIRED_GATES set" }
  - { from: service_bind_ok, to: service_skip, label: "No & BEAM_REQUIRED_GATES unset" }
  - { from: run_gpu, to: gpu_ok }
  - { from: gpu_ok, to: gpu_evidence, label: "Yes" }
  - { from: gpu_ok, to: gpu_fail, label: "No & BEAM_REQUIRED_GATES set" }
  - { from: gpu_ok, to: gpu_skip, label: "No & BEAM_REQUIRED_GATES unset" }
  - { from: service_evidence, to: done }
  - { from: gpu_evidence, to: done }
---
flowchart TD
    start([cargo test -p beam]) --> env_check[Check BEAM_REQUIRED_GATES env]
    env_check --> run_service[Run service_end_to_end]
    env_check --> run_gpu[Run gpu_matches_cpu_oracle]
    run_service --> service_bind_ok{TcpListener::bind ok?}
    service_bind_ok -->|Yes| service_evidence[Emit JSON: category=service, transport=h2c, assertions=count]
    service_bind_ok -->|No & Set| service_fail[Panic & fail gate]
    service_bind_ok -->|No & Unset| service_skip[Print skip, return success]
    run_gpu --> gpu_ok{GpuContext::new ok?}
    gpu_ok -->|Yes| gpu_evidence[Emit JSON: category=gpu, adapter=name, backend=backend, assertions=count]
    gpu_ok -->|No & Set| gpu_fail[Panic & fail gate]
    gpu_ok -->|No & Unset| gpu_skip[Print skip, return success]
    service_evidence --> done([Done])
    gpu_evidence --> done
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the `beam-test-gate-execution-policy` work root under the `long-running-stability` capability."
  - path: apps/beam/aw.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Update the `test_cmd` to run the full test suite with `BEAM_REQUIRED_GATES=1`."
  - path: apps/beam/tests/service.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: service_end_to_end
    description: "Panic on socket bind failure when `BEAM_REQUIRED_GATES` is set, and emit JSON evidence."
  - path: apps/beam/tests/gpu_matches_cpu.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: gpu_matches_cpu_oracle
    description: "Panic on missing GPU adapter when `BEAM_REQUIRED_GATES` is set, and emit JSON evidence."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: beam-test-gate-execution-policy-verification
requirements:
  honest_gate_failure:
    id: R2
    text: "Panic and fail the test run if required GPU context or service listener prerequisites are missing when `BEAM_REQUIRED_GATES` is set."
    kind: functional
    risk: high
    verify: cargo test -p beam --test service --test gpu_matches_cpu
  machine_readable_evidence:
    id: R3
    text: "Emit structured machine-readable JSON evidence to stdout recording actual adapter, transport, and executed assertion count."
    kind: functional
    risk: medium
    verify: cargo test -p beam --test service --test gpu_matches_cpu
  test_gate_configuration:
    id: R1
    text: "Configure the test gate in `apps/beam/aw.toml` to execute all Beam unit and integration tests with `BEAM_REQUIRED_GATES=1`."
    kind: functional
    risk: high
    verify: aw health --project beam tests --verify-tests
---
flowchart TD
    r1[R1 test gate configuration] --> aw_health_project_beam_tests_verify_tests[aw health --project beam tests --verify-tests]
    r2[R2 honest gate failure] --> cargo_test_p_beam_test_service_test_gpu_matches_cpu[cargo test -p beam --test service --test gpu_matches_cpu]
    r3[R3 machine readable evidence] --> cargo_test_p_beam_test_service_test_gpu_matches_cpu
```
