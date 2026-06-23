# loom

DAG workflow scheduler. loom composes per-task lifecycles into a **dynamic DAG**
and is the **complex** member of the trio: `keep` (result store) and `relay`
(broker) are deliberately simple primitives, and all orchestration complexity is
concentrated here. Rust, k8s/cloud-native, multicore, RAM+disk. loom owns its
**own** sharded, strongly-consistent workflow state. Epic: #106.

## Architecture (control plane, never a data path)

loom coordinates with small messages only; payload bytes never traverse it
(claim-check via keep). See the boundaries ADR (#165) and runner selection (#164).

```
client ──▶ loom (submit / status / result-ref)   +  keep (payload bytes, direct)     ✗ relay
worker ──▶ relay (lease / ack / heartbeat)        +  keep (input / result bytes)       ✗ loom
loom   ──▶ relay (publish task, observe acks)     +  keep (read result-ref)            ✱ never touches worker
```

- **client** knows loom (control) + keep (data, claim-check); never relay.
- **worker** knows relay + keep; never loom. loom observes completion via relay acks.
- **relay / keep** stay simple and passive; the only role that coordinates both is loom.
- Transparency is protocol-level (HTTP/2 + OpenAPI, generated clients — no bespoke SDK).

### Lifecycle (one node)

loom picks the next ready node → assembles input refs from keep → publishes the
task to relay (tagged with its `runner` class) → a worker leases, runs, writes
the result to keep, acks relay → loom updates its DAG state → repeat. Fan-in
barrier = done counter (by task id, not attempt).

### Binary (one binary, role-per-subcommand, #164)

| subcommand | role |
|------------|------|
| `loom controller` | scheduler + sharded DAG state; serves the client control API |
| `loom worker` | resident pull-loop worker harness (`runner=resident`) |
| `loom run-task` | single-shot in-Job task entrypoint (`runner=k8s-job`) |
| `loom job-controller` | relay → k8s Job bridge (the only component touching the k8s API) |

## Boundaries

- `relay` (broker) = delivery only; `keep` (store) = payload/result only; loom = orchestration.
- A task's `runner` (resident / k8s-job / local) is task-level metadata; loom selects it, relay routes by it, loom never owns execution.
- Worker execution is polyglot (relay + keep OpenAPI); not built here beyond the reference harness.
- All interfaces HTTP/2 + OpenAPI. Depends on the relay epic (#120) + keep epic (#121).

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Workflow Orchestration | 116 | planned | none | candidate | not_ready | DAG/stage state, next-node, dynamic fan-out/fan-in, fan-in barrier |
| Workflow Data Model | 112 | planned | none | candidate | not_ready | WorkflowRun / Node / Stage + claim-check input/result refs |
| State Durability | 110 | planned | none | candidate | not_ready | sharded, strongly-consistent DAG state; per-shard raft + crash recovery (#123) |
| Runner & Execution Selection | 164 | planned | none | candidate | not_ready | per-task runner metadata; resident / k8s-job / local; loom selects, never owns execution |
| Client Control API | 165 | planned | none | candidate | not_ready | thin HTTP/2 submit / status / result-ref; sync run_id + validation + introspection + authz |
| Worker Harness | 164 | planned | none | candidate | not_ready | thin lease/heartbeat/keep-IO/ack-exactly-once harness; Rust reference, polyglot via OpenAPI |
| Fair Dispatch | 107 | planned | none | candidate | not_ready | weighted fair-share + lazy bounded materialization + quota |
| Competitive Perf Gate | 127 | planned | none | candidate | not_ready | frontier + dormant-axis benchmark vs comparable schedulers |
