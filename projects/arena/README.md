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
rig     — drives ONE service with load → latency.   (arena reuses its loadgen)
meter   — measures ONE process → cpu/RSS.           (deferred runtime flavor)
```

`vat` runs `arena` as just another `[[runners]]` entry; `arena` is the middle
compare layer over `rig`/`meter`. It never generates load itself for the
runtime flavor — it composes meter's `--drive` seam — and for the service
flavor it reuses rig's open-loop loadgen.

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
| `[targets.<id>]` | `kind = "service"` (rig loadgen) + a `[targets.<id>.load]` shape |
| `[[cells]]` + `[cells.targets.<id>]` | one logical workload, per-target `request` (glue) + `gate` |

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

## Scope (v1) and what's deferred

**v1 ships the SERVICE flavor only**: HTTP targets driven by rig's loadgen,
compared on latency. Both compared targets must speak HTTP (lumen vs OpenSearch
is fair; **Postgres is not** — it speaks its native wire protocol, so it is the
deferred `command` flavor, not an HTTP service).

Service ratios are honest **end-to-end** latency and inherit the HTTP transport
floor by design — the correct comparison for a shipped HTTP server. Mark
floor-dominated cheap cells `gate = "exempt"`; arena never peeks at
engine-internal timings (that is glue, and the deferred runtime flavor).

Deferred: the runtime/meter flavor (`kind = "runtime"` via `meter profile
--drive` → cpu/RSS, for mamba-vs-CPython); the `command` flavor (opaque metric,
for pg); migrating lumen's full `perf_gate_vs_db.rs`; correctness-diff
comparison; concurrent driving.

## Build & test

```bash
cargo test -p arena                 # 11 tests: spec/compare units + stub-server pipeline e2e
projects/arena/build.sh debug       # installs ~/.cargo/bin/arena
arena run --spec projects/arena/examples/lumen-vs-opensearch.toml --update-baselines
```

The pipeline e2e (`tests/pipeline_e2e.rs`) drives the whole
measure→compare→gate→report→exit path against two in-test stub HTTP servers
(fast base, slow peer), proving WIN-breach→exit 2, exempt→no-gate, and
baseline recording without any real services.
