---
id: relay-keep-worker-facing-contract
summary: The worker-facing contract for polyglot workers — relay's lease/ack/heartbeat OpenAPI plus keep's get-input/put-result, and the worker loop that ties them. Deliverable is the contract (a doc + a test-only reference worker), not a worker. relay stays standalone.
fill_sections: [unit-test, changes]
---

# relay + keep — worker-facing OpenAPI contract (polyglot worker integration)

The worker is **out of scope**: any language integrates over HTTP/2 + OpenAPI.
This deliverable is the contract, not a worker — a hand-written contract
document (`docs/worker-protocol.md`) plus a test-only reference worker that
drives the loop over h2c. relay's machine-readable contract is the OpenAPI
served at `/openapi.json` (lease / ack / heartbeat, from #113 / #115); keep's
get-input / put-result is a cross-project contract owned by the keep epic.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-worker-contract-test-plan
entry: suite
nodes:
  suite:
    kind: start
    label: "worker-loop contract test (reference worker over h2c)"
  t_loop:
    kind: process
    label: "publish 3; a reference worker loops lease -> run -> heartbeat -> ack until lease==null"
  a_loop:
    kind: terminal
    label: "assert all 3 are processed exactly once and committed_seq reaches 2"
  t_heartbeat:
    kind: process
    label: "during run the worker POSTs heartbeat(lease_id, epoch)"
  a_heartbeat:
    kind: terminal
    label: "assert heartbeat extended=true while the lease is held"
  t_openapi:
    kind: process
    label: "GET /openapi.json"
  a_openapi:
    kind: terminal
    label: "assert lease, ack and heartbeat paths are present (machine contract)"
edges:
  - { from: suite, to: t_loop, label: "case: full loop" }
  - { from: t_loop, to: a_loop }
  - { from: suite, to: t_heartbeat, label: "case: heartbeat" }
  - { from: t_heartbeat, to: a_heartbeat }
  - { from: suite, to: t_openapi, label: "case: openapi contract" }
  - { from: t_openapi, to: a_openapi }
---
flowchart TD
    suite([worker contract suite]) --> t_loop[reference worker lease->run->ack loop]
    t_loop --> a_loop([each processed once, committed=2])
    suite --> t_heartbeat[heartbeat during run]
    t_heartbeat --> a_heartbeat([extended=true])
    suite --> t_openapi[GET /openapi.json]
    t_openapi --> a_openapi([lease/ack/heartbeat present])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/tests/worker_loop.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Throwaway reference worker (test-only): drives the lease / heartbeat / ack loop over h2c against an in-process relay, validating the worker-facing contract and the served OpenAPI (lease/ack/heartbeat)."
```

# Reviews

### Review 1
**Verdict:** approved

- [unit-test] A reference worker drives lease -> run -> heartbeat -> ack over h2c and asserts each entry is processed exactly once, plus the served OpenAPI lists lease/ack/heartbeat. Validates the worker-facing contract end-to-end. Applicable.
- [changes] One test-only reference worker; the machine contract is the already-served OpenAPI and the human contract is the TD intro / docs. No worker shipped in the lib. Applicable.
