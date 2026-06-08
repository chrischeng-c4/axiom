# 1482-patrol-handoff (anchor for #1482–#1532 3p-conformance cluster)

**Issue:** #1482 — conformance(mamba/3p): urllib3 — vendor tests pass + perf ≥1.0× + 95% surface
**Branch:** `issue-1482` (materialization only)
**Stop reason:** Same blockers as #1483 (handed off ~14:55 today) — entire 49-issue 3p-conformance cluster shares 4 infra dependencies that must land before any per-lib issue is 1-tick autopilot scope

## The cluster

49 open `conformance(mamba/3p): <lib>` issues span #1482–#1532, all `priority:p2 / phase:created`, all use the same body template (Problem / R1–R4 / Scope / RefCtx / DoD-3-gates). Sample numbers and libraries:

```
1482 urllib3        1483 certifi        1484 charset-normalizer  1486 idna
1487 requests       1488 httpx          1489 aiohttp             1490 aiofiles
1491 cryptography   1492 pyOpenSSL      1493 attrs                1494 typing-extensions
1495 pydantic       1496 pydantic-core  1497 jsonschema           1498 marshmallow
1499 msgpack        1500 orjson         1501 boto3                1502 botocore
1503 s3transfer     ... (28 more, contiguous up to #1532)
```

Full list (49 numbers) generated locally via `gh issue list --label project:mamba --state open --search "-label:type:epic -label:type:tracking -label:flagged:needs-human" --limit 200 | jq '.[] | select(.title | contains("mamba/3p")) | .number'`.

## Why none are 1-tick autopilot scope

Per the #1483 handoff (canonical analysis — read that first), every per-lib issue is blocked on **four shared infrastructure pieces that do not yet exist**:

1. **`infra(mamba/3p): pytest-under-mamba runner for vendor test suites`**
   R1 of every per-lib spec says "vendor test suite green under mamba". Today there is no `mamba pytest <path>` invocation that exits non-zero on test failure with appropriate Python-runtime semantics. Building it once unlocks R1 for all 49 libs.

2. **`infra(mamba/3p): cross-runtime bench harness (CPython vs mamba)`**
   R2 of every per-lib spec says "bench ≥ 1.0× CPython 3.12". Today there is no `cargo bench -p mamba-bench-3p <lib>` invocation; `projects/mamba/benches/3p/` does not exist; no comparison logic exists. One harness, 49 fixture targets.

3. **`infra(mamba/3p): typeshed surface coverage tool`**
   R3 of every per-lib spec says "typeshed surface ≥ 95% covered". Today there is no surface-extraction tool for typeshed (`#1397` covers this for the umbrella; a CLI subcommand `mamba-surface-report --package <lib>` needs to land).

4. **`infra(mamba/3p): real-world hello-world fixture convention`**
   R4 of every per-lib spec says "at least one downstream PyPI consumer runs unchanged". Today there is no `tests/fixtures/conformance/3p/<lib>/real_world/` pattern + runner; per-lib issues each invent their own. Document the convention once.

After these four infra issues land, every per-lib issue (the 49 in this cluster + the ~50 stdlib counterparts #14xx) becomes a real 1-tick autopilot candidate: write the shim, write the fixtures pointing at the existing harness, commit. **This is the single highest-throughput unblock for the mamba backlog.**

## Operator actions

### 1. File the 4 infra issues

Each as `priority:p1, project:mamba, type:enhancement`, with full CRRR sections (Problem / Requirements / Scope / Reference Context / DoD). The DoD acceptance for each should be a runnable command, not a feature description.

### 2. Bulk-flag the 49 per-lib issues so autopilot stops bailing on them

Patrol attempted this in-tick but was correctly blocked by the auto-mode classifier (49-issue mass-edit is beyond a single-tick patrol's authorized scope). Operator can run:

```bash
NUMS="1482 1483 1484 1486 1487 1488 1489 1490 1491 1492 1493 1494 1495 1496 1497 1498 1499 1500 1501 1502 1503"
NUMS="$NUMS 1504 1505 1506 1507 1508 1509 1510 1511 1512 1513 1514 1515 1516 1517 1518 1519 1520 1521 1522 1523 1524 1525 1526 1527 1528 1529 1530 1531 1532"
# (verify list matches `gh issue list --label project:mamba --state open --search "..." | jq '.[].number'`)
for n in $NUMS; do
  gh issue edit "$n" --repo chrischeng-c4/cclab \
    --add-label "flagged:needs-human" \
    --comment "Blocked by 3p infra cluster — see .score/handoffs/1482-patrol-handoff.md. Will unflag once pytest-under-mamba / bench-harness / surface-tool / real-world-fixture-pattern infra issues land."
done
```

(Add a `--remove-label "flagged:needs-human"` sweep once the infra lands so the per-lib cluster reactivates.)

### 3. Same pattern almost certainly applies to the stdlib cluster

Inspecting the `gh issue list` output shows ~50 sibling `conformance(mamba/stdlib): <module>` issues (#14xx range, e.g. #1429 struct, #1430 io, #1431 os) using the same R1–R4 / DoD template. They block on the same infra except R1 → "CPython Lib/test passes" (i.e. `pytest-under-mamba` + CPython's own `Lib/test/test_<mod>.py` files). Same 4 infra issues (with a stdlib-aware wrapper in #1) unblock both clusters.

Operator may want to file 5 infra issues (4 shared + 1 stdlib-specific Lib/test loader) and bulk-flag ~100 per-lib issues in one operator session.

### 4. Format fixes the per-lib bodies need

A separate finding from the #1483 tick: the existing per-lib bodies use `**R1 (Behavior)** — ...` which is rejected by `score wi validate` (validator requires `- R1: ...`). The Spec Plan table also has `(fill)` placeholders. Bulk-rewriting these via a script is cheaper than 49 per-issue manual edits — operator may want to land a script that:

- Replaces `**R(\d+) \([^)]+\)** —` with `- R\1:`
- Replaces `(fill via score td create) | (fill) | (fill)` with `mamba-3p-<lib>-conformance | create | projects/mamba/src/runtime/stdlib/3p/<lib>_mod.rs`
- Adjusts Scope paths from `stdlib/<lib>_mod.rs` to `stdlib/3p/<lib>_mod.rs` for the 3p subset

## What patrol did this tick

1. Pre-flight clean; no in-flight mamba branches.
2. State-C scan: p0/p1 fully `flagged:needs-human` from prior ticks; p2 head-of-queue is the 49-issue 3p cluster (anchor #1482).
3. Materialized #1482 as the anchor for this cluster handoff.
4. Attempted bulk-flag of 49 issues → auto-mode classifier (correctly) blocked external mass-edit.
5. Pivoted to single-issue handoff that hands the bulk-flag back to the operator with a copy-paste shell snippet.
6. Will flag #1482 only (single issue, within patrol's authorized scope) so the next tick skips at least the anchor.
7. Will FF the materialize + handoff commits into `project-mamba`.

## Tick timing

Started ~15:17, bailed ~15:22 — ~5 min. Cluster discovery + verification dominated; handoff doc write was the rest.

## Predicted next-tick behavior

Without operator intervention, the next state-C scan (15:47) picks #1484 (charset-normalizer — next-oldest after #1482 + #1483 are skipped). It will also bail with reference to this handoff. Net throughput will be ~30 per-lib bails over the next 24h (one per tick), each ~2–5 min — wasted patrol time. The operator's one-shot bulk-flag-and-file-infra session converts that into actual forward progress.

## Repeated observation across 7 ticks now

| Tick | Slug | Phase | Outcome | Class |
|------|------|-------|---------|-------|
| 1 | #2023 | merged | completion ✓ | mamba-bug |
| 2 | #1696 | created | bail | bug-narrative spec |
| 3 | #1260 | created | bail | grab-bag perf |
| 4 | #1312/1313/1546 | td_reviewed | bail × 3 | empty Adopted stubs |
| 5 | #1749 | none | bail | umbrella (5-fire) |
| 6 | #1885 | none | bail | umbrella (4-child) |
| 7 | #1483 | reviewed | progress + bail | 3p infra blocked |
| 8 | #1482 | created | cluster handoff | 3p cluster anchor |

Bail rate 7/8 (88%). Patrol is doing its job — surfacing structural backlog problems — but the backlog has no clean p0/p1/p2 single-fire work. Until operator decomposes/files-infra, autopilot ticks are pure triage.
