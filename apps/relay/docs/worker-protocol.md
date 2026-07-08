# relay worker-facing protocol (#108)

The worker is **out of scope** for relay. Any language (Python / TypeScript /
Rust) integrates purely over **HTTP/2 + OpenAPI** — there is no relay worker
SDK to depend on. This document is the *contract*; the machine-readable form is
served at `GET /openapi.json`.

## The worker loop

```text
loop:
  1. lease   = POST relay /v1/{subject}/lease   {consumer_id}        -> Lease | null
  2. if null: back off, then retry
  3. input   = GET  keep  /v1/inputs/{message_id}                    -> bytes   (keep, #114)
  4. result  = run(input)                                            (worker code)
       while running, periodically:
         POST relay /v1/{subject}/heartbeat {lease_id, epoch}        -> {extended}
  5. PUT  keep  /v1/results/{message_id} result                      (keep, #114)
  6. ack    = POST relay /v1/{subject}/ack {lease_id, epoch}         -> {acked, committed_seq}
```

Completion is reported back to the enqueuer. The `done == N -> next-node` logic
belongs to the orchestrator (loom, #116), **not** relay — relay knows nothing
about workflows.

## relay endpoints (this project)

| Verb | Endpoint | Purpose |
|------|----------|---------|
| lease | `POST /v1/{subject}/lease` | Lease the next ready entry (prefers redelivery). Returns a `Lease` carrying a monotonic `epoch`, or `null`. |
| heartbeat | `POST /v1/{subject}/heartbeat` | Extend a held lease (`lease_id` + `epoch`); proves the worker is alive. |
| ack | `POST /v1/{subject}/ack` | Complete a lease (`lease_id` + `epoch`). A stale `epoch` is a no-op. |

JSON is the documented shape; `lease` / `ack` / `heartbeat` also accept
`application/cbor` for the hot path.

## keep endpoints (cross-project, keep epic #121 / #114)

keep owns the claim-check store and publishes its own OpenAPI:

| Verb | Endpoint | Purpose |
|------|----------|---------|
| get input | `GET /v1/inputs/{id}` | Fetch a job's input payload. |
| put result | `PUT /v1/results/{id}` | Store a job's result. |

keep is a separate project and is not built here; these rows are the agreed
cross-project contract the two OpenAPI documents must satisfy.

## Fencing guarantee

`epoch` fences a stalled worker. If a lease expires and the reconciler (#109)
reclaims it, the entry is redelivered with a **higher epoch**. The original
worker's late `heartbeat` / `ack` (carrying the old epoch) is rejected, so **at
most one worker ever completes a given entry** — no double-completion, no lost
work.

## Reference worker

A throwaway reference worker exists only as a test
(`tests/worker_loop.rs`); it drives this loop over h2c against an in-process
relay. It is not shipped and is not a dependency.
