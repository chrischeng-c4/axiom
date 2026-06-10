# rig

**Declarative test-scenario harness engine for the cclab ecosystem.**

`rig` runs declarative SCENARIOS (e2e behavior, TOML step DSL) and LOAD
profiles (open-loop QPS × workers) against a real serving process, judges
them with assertions and declarative pins (floors/ratchets vs a baseline
store), and prints ONE agent-readable JSON report (`rig.report/1`).

It is the extracted, domain-free essence of mamba's
`tests/harness/cpython` harness: the fixture-record contract
(path==record + lint), declarative gates, verdict bucketing
(pass/xfail/skip), and child-process timeout policy — generalized so any
project can consume them by writing TOML scenarios, not bash.

## Division of labor

```text
vat   — environment: services (postgres/redis/nats/...), COW workspace,
        readiness probes. rig declares needs; vat satisfies them.
rig   — case orchestration: scenario steps, assertions, load generation,
        pin gates, verdicts, the report.
meter — resource attribution (profiling). rig never claims it.
```

## Mental model

```text
rig run --dir tests/rig/scenarios [--vat] [--pins tests/rig/config/pins]
  discover *.toml -> lint records (path==record)
  per scenario: interpolate {{vars}} -> execute steps under TimeoutPolicy
    http | sample | assert | wait_until | measure_rss | exec | sleep
  kind = "load": open-loop generator -> p50/p99/error_rate/achieved_qps
  gate pins (floor / ratchet vs .rig/baselines.json)
  bucket verdicts (pass / xfail / skip; xpass = graduate-to-pass signal)
  print ONE RigReport JSON -> exit 0 clean / 1 findings / 2 regression / 3+ tool error
```

## Verbs

| verb | status | behavior |
|---|---|---|
| `rig run [--scenario <f> \| --dir <d>] [--pins <d>] [--update-baselines] [--vat]` | v0 (in progress) | discover → lint → execute → gate → one JSON report |
| `rig lint [--dir <d>]` | v0 (in progress) | record-contract check only, no execution |
| `rig report` | v0 | re-project `.rig/last-report.json` (read-only) |
| `rig spec` / `rig llm` | v1 | offline self-description / agent playbook |

## Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| scenario-engine | axiom#5 | in-progress | none | none | not-ready | record contract, step DSL, verdicts, report |
| load-pins | axiom#5 | in-progress | none | none | not-ready | open-loop loadgen, floor/ratchet pins, JSON baseline store |
| vat-wrapped-runs | axiom#5 | planned | none | none | not-ready | `--vat` JSONL checkpoint seam |

## Non-goals (v0)

- kind/k8s environment provisioning (scenario DSL can express the
  assertions today; vat has no kind preset yet — lands with vat, not rig)
- multi-host / distributed load generation
- HTML reports (`--human` stderr summary only)
- fixture GENERATION tooling (generate→fill loops stay project-side)
- resource attribution (meter owns it)
- closed-loop (latency-coupled) load

## First consumer

lumen: `projects/lumen/tests/rig/scenarios/` ports `scripts/chaos.sh`
(partition recovery, packet-loss p99) and `scripts/soak.sh` (two-window
RSS plateau) to scenarios, plus one `load/search_qps` pin
(`config/pins/search_p99.toml`).
