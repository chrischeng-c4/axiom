# arena

**N-target competitive comparison runner — fan one workload across targets, ratio + ratchet-gate, one agent-readable JSON report.**

`arena` runs the SAME logical workload against N targets, reduces each to one
comparable scalar, computes `ratio = peer / base`, classifies each cell
WIN / EXEMPT / TARGET, gates WIN cells against a ratcheted per-host baseline,
and prints one `arena.report/1` document.

It is the missing "compare N" layer of the ecosystem. Two places in this repo
hand-rolled the same machine — lumen's `tests/perf_gate_vs_db.rs` (lumen vs
Postgres/OpenSearch latency) and mamba's `tools/perf_baseline.py` (mamba vs
CPython cpu/RSS). arena extracts the comparison essence so neither has to.

## Ecosystem layering

```text
vat     — environment: provisions each target's services + a COW workspace.
arena   — comparison: holds the workload spec, fans out, ratios + gates.   <-- this
rig     — drives ONE target with load → latency. Owns the transports (http,
          postgres). "how to talk to a target" lives here, not in arena.
meter   — measures ONE process → cpu/RSS.           (deferred runtime flavor)
```

`vat` runs `arena` as just another `[[runners]]` entry; `arena` is the middle
compare layer over `rig`/`meter`. arena knows **zero** protocols — it asks rig
for each target's number through one entry, and rig's transport (HTTP or
Postgres) owns the wire. Every transport runs on rig's ONE open-loop scheduler
(same warmup/percentile/honesty), so a thin Rust client's overhead is a
near-constant floor across targets and the ratio reflects the backend +
protocol, not the client library — the whole point of comparing lumen vs pg
fairly. Protocol cost is kept (it is part of what each backend delivers);
floor-dominated cheap cells are marked `exempt`.

## Mental model

```text
arena run --spec arena.toml
  for each cell:
    measure base, then each peer (SEQUENTIALLY — concurrent drivers poison the ratio)
      service target -> rig loadgen -> p99_ms
    ratio = peer / base   (lower-is-better metric; ratio > 1 => base wins)
    classify vs gate: win | exempt | target
    gate WIN cells vs max(1.0, ratchet * baseline)   (host-scoped .arena/baselines.json)
  print ONE arena.report/1 JSON -> exit 0 clean / 1 findings / 2 regression / 3 usage / 4 missing-tool / 5 io
```

## Spec — `arena.toml`

The cell name is the join key; each `[cells.targets.<id>]` sub-table is the
**opaque glue payload** (the SAME query as lumen-JSON vs OS-DSL) that arena
passes straight into the load profile and never reads. `gate`/`reason`/`floor`
live on the **peer** side, so per-peer classification (base WINS vs peer A but
is EXEMPT vs peer B on one cell) is representable. See
`examples/lumen-vs-opensearch.toml`.

| Field | Meaning |
|-------|---------|
| `base` | the target every ratio divides BY (`ratio = peer/base`) |
| `metric` | comparable scalar, lower-is-better (`p99_ms` default); per-cell overridable |
| `ratchet` | baseline ratchet (default 0.8 — must hold `max(1.0, 0.8*baseline)`) |
| `[targets.<id>]` | `kind = "http"` (alias `service`) or `kind = "postgres"` (+ `dsn`), and a `[targets.<id>.load]` shape |
| `[[cells]]` + `[cells.targets.<id>]` | one logical workload: per-target `request` (http glue) or `query` (pg SQL) + `gate` |

Transports (rig owns these; arena just selects):

- **http** (alias `service`) — thin ureq client; the cell payload is a
  `request = { method, url, body }`. Author it from the service's published
  OpenAPI as a human reference.
- **postgres** — `tokio-postgres` prepared statement (rig `postgres` feature);
  the target carries a `dsn`, the cell payload is a `query` SQL string. pg has
  no OpenAPI — the SQL is hand-written.

Gate classes (a direct port of lumen's `judge()`):
- **win** — base must beat the peer by `max(1.0, ratchet*baseline)`; a breach is
  a `pin_regression` finding → **exit 2** (build-failing).
- **target** — aspirational floor; reported RED below it but never gates.
- **exempt** — reported, never compared for pass/fail (the pressure valve for
  "fair comparison impossible here", e.g. a floor-dominated cheap cell).

## Verbs

| Command | Effect |
|---------|--------|
| `arena run --spec <toml> [--update-baselines] [--strict]` | measure → ratio → gate → one report |
| `arena report` | re-project `.arena/last-report.json` (no measurement) |
| `arena spec` / `arena llm` | offline self-describer / playbook (v1 stubs) |

First run records the baseline: `arena run --spec <toml> --update-baselines`.
Later runs gate against it; a regression below the ratcheted floor exits 2.

## As a vat runner

`vat` brings up every target's services, then runs arena as a runner that talks
to all of them (the model already supports "N services up + one runner"):

```toml
[[services]]
id = "lumen"
cmd = ["../../target/release/lumen", "serve"]
ready_http = "http://127.0.0.1:7373/healthz"
[[services]]
id = "opensearch"
# ... raw cmd or external instance on :9200 ...

[[runners]]
id = "arena-search"
requires = ["lumen", "opensearch"]
cmd = ["../../target/debug/arena", "run", "--spec", "tests/arena/search.toml"]
timeout_s = 600
```

## Scope and what's deferred

**Transports: `http` and `postgres`.** Latency comparison across an HTTP
service and a SQL backend on one scheduler — `lumen` (http) vs `pg` (postgres),
and lumen vs OpenSearch (both http). Ratios are honest **end-to-end** latency
with a thin Rust client; the protocol floor is kept (it is part of the cost),
and floor-dominated cheap cells are `gate = "exempt"`. arena never peeks at
engine-internal timings — that is glue, and the transport's job is the wire.

Deferred: the runtime/meter flavor (`meter profile --drive` → cpu/RSS, for
mamba-vs-CPython); other transports (redis, opaque command); OpenAPI-driven
request scaffolding (lumen's published OpenAPI is a human reference for now);
migrating lumen's full `perf_gate_vs_db.rs`; correctness-diff comparison;
concurrent driving; http/2 (the loadgen client is h1 ureq today).

## Build & test

```bash
cargo test -p arena                 # spec/compare units + stub-server pipeline e2e
                                    # (+ a real-pg cross-transport test that skips if no local pg)
projects/arena/build.sh debug       # installs ~/.cargo/bin/arena
arena run --spec projects/arena/examples/lumen-vs-pg.toml --update-baselines
```

The pipeline e2e (`tests/pipeline_e2e.rs`) drives the whole
measure→compare→gate→report→exit path against two in-test stub HTTP servers
(fast base, slow peer) — proving WIN-breach→exit 2, exempt→no-gate, and
baseline recording without any real services — plus a cross-transport test that
compares a stub HTTP target against a real local Postgres (`SELECT 1`),
skipping gracefully when no pg is running.
