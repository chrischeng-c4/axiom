# 1414-patrol-handoff (anchor for #1414–#1481 stdlib-conformance cluster, 67 issues)

**Issue:** #1414 — conformance(mamba/stdlib): ssl — CPython Lib/test pass + perf ≥1.0× + 95% surface
**Branch:** `issue-1414` (materialization only)
**Stop reason:** Sibling of the 3p cluster — same 4 shared infra blockers, with one stdlib-specific variant. See #1482 handoff for canonical analysis.

## The cluster

67 open `conformance(mamba/stdlib): <module>` issues span #1414–#1481, all `priority:p2 / phase:created`, all use the same body template as the 3p cluster (Problem / R1–R4 / Scope / RefCtx / DoD-3-gates) — except R1 says **"CPython `Lib/test/test_<mod>.py` runs green under mamba"** instead of **"vendor test suite green"**.

Sample (first / last):

```
1414 ssl                1415 (next)             1416 (next)
1417 (next)             1418 (next)             ...
1429 struct             1430 io                 1431 os
...
1478 (next)             1479 (next)             1480 (next)             1481 xml.etree.ElementTree
```

Full numeric span: #1414, #1415, #1416, #1417, #1418, #1419, #1420, #1421, #1422, #1423, #1424, #1425, #1426, #1427, #1428, #1429, #1430, #1431, #1432, #1433, #1434, #1435, #1436, #1437, #1438, #1439, #1440, #1441, #1442, #1443, #1444, #1445, #1446, #1447, #1448, #1449, #1450, #1451, #1452, #1453, #1454, #1455, #1456, #1457, #1458, #1459, #1460, #1461, #1462, #1463, #1464, #1465, #1466, #1467, #1468, #1469, #1470, #1471, #1472, #1473, #1474, #1475, #1476, #1477, #1478, #1479, #1480, #1481 (67 total).

Combined with the 48-issue 3p cluster (#1482–#1532, handed off as `1482-patrol-handoff.md`), the operator faces a **115-issue p2 backlog** that all share the same infra blockers.

## Blockers — see #1482 handoff for full analysis

Per `.score/handoffs/1482-patrol-handoff.md`, four shared infra issues unblock the 3p cluster:

1. `infra(mamba/3p): pytest-under-mamba runner for vendor test suites`
2. `infra(mamba/3p): cross-runtime bench harness (CPython vs mamba)`
3. `infra(mamba/3p): typeshed surface coverage tool`
4. `infra(mamba/3p): real-world hello-world fixture convention`

For the stdlib cluster, three of those four are direct dependencies. The fourth (vendor test runner) is replaced by:

5. **`infra(mamba/stdlib): CPython Lib/test/test_<mod>.py loader + runner`**
   - R1 of every stdlib per-lib spec says "CPython Lib/test passes". Today there is no integration with CPython's own `Lib/test/test_<mod>.py` files; each per-lib issue would have to reinvent the loader. Building one loader once unlocks R1 for all 67 stdlib libs.
   - Likely shares the bench/surface/real-world fixture infra with the 3p cluster — same harness, different test entry point.

So the actual cluster-unlock is **5 infra issues** that unblock **115 per-lib issues**.

## Operator actions

### 1. File 5 infra issues (one operator session)

Each as `priority:p1, project:mamba, type:enhancement` with full CRRR sections.

```bash
score wi create --title "infra(mamba/stdlib): CPython Lib/test loader + runner" --type enhancement --project mamba --priority p1
score wi create --title "infra(mamba/3p): pytest-under-mamba runner for vendor test suites" --type enhancement --project mamba --priority p1
score wi create --title "infra(mamba): cross-runtime bench harness (CPython vs mamba)" --type enhancement --project mamba --priority p1
score wi create --title "infra(mamba): typeshed surface coverage tool" --type enhancement --project mamba --priority p1
score wi create --title "infra(mamba): real-world hello-world fixture convention" --type enhancement --project mamba --priority p1
```

### 2. Bulk-flag 115 per-lib issues so autopilot stops bailing

Patrol cannot do this mass edit (auto-mode classifier correctly blocks). Operator can run:

```bash
# Build the number list from the live GitHub state, then flag.
gh issue list --repo chrischeng-c4/cclab --label "project:mamba" --state open \
  --search "-label:type:epic -label:type:tracking -label:flagged:needs-human" \
  --json number,title --limit 200 \
  | jq -r '.[] | select(.title | test("conformance\\(mamba/(stdlib|3p)\\)")) | .number' \
  | while read n; do
      gh issue edit "$n" --repo chrischeng-c4/cclab \
        --add-label "flagged:needs-human" \
        --comment "Blocked by mamba conformance infra cluster — see .score/handoffs/1414-patrol-handoff.md and .score/handoffs/1482-patrol-handoff.md. Will unflag once the 5 infra issues land."
    done
```

(After infra lands, sweep `--remove-label "flagged:needs-human"` to reactivate the cluster.)

### 3. Format fixes per #1483 handoff

Same R1–R4 / Spec Plan format issues apply to all 115 bodies. The bulk-rewrite script described in `1482-patrol-handoff.md` step 4 should run once with regexes adjusted for both `(mamba/stdlib)` and `(mamba/3p)` patterns.

## What patrol did this tick

1. Pre-flight clean; no in-flight mamba branches (the 6 in-flight branches are all project:sdd or project:pg — verified at this tick again).
2. State-C scan with `--limit 200`: discovered #1414 is the actual oldest unflagged p2 (prior tick used `--limit 50` and missed it).
3. Confirmed #1414 is a sibling of the 3p cluster — same template, same R1–R4 layout, same DoD shape. Differs only in R1 (CPython Lib/test vs vendor tests) and the in-scope path (no `3p/` subdir).
4. Materialized #1414 + wrote this handoff. Did NOT attempt bulk-flag (correctly out of scope after the prior auto-mode classifier denial).
5. Will flag #1414 only (single issue, in scope) + FF into project-mamba.

## Tick timing

Started ~15:47, bailed ~15:52 — ~5 min. Same shape as the #1482 tick.

## Updated session table

| Tick | Slug | Phase | Outcome | Class |
|------|------|-------|---------|-------|
| 1 | #2023 | merged | completion ✓ | mamba-bug |
| 2 | #1696 | created | bail | bug-narrative spec |
| 3 | #1260 | created | bail | grab-bag perf |
| 4 | #1312/1313/1546 | td_reviewed | bail × 3 | empty Adopted stubs |
| 5 | #1749 | none | bail | umbrella (5-fire) |
| 6 | #1885 | none | bail | umbrella (4-child) |
| 7 | #1483 | reviewed | partial+bail | 3p infra blocked |
| 8 | #1482 | created | cluster handoff | 3p anchor (49) |
| 9 | #1414 | created | cluster handoff | stdlib anchor (67) |

After this tick, the unflagged-p2 backlog effectively goes to zero (all 115 per-lib conformance issues are anchored under one of the two cluster handoffs). Next tick (16:17) will fall to p3, then `priority:none`, OR — if operator runs the bulk-flag script — to whatever non-conformance work survives the filter.

## Suggestion: pause autopilot until operator session

Patrol is in a known-unproductive state. The 30-min cron will keep ticking and burning ~5 min/tick on cluster discovery + handoff writes until the operator either (a) files the 5 infra issues or (b) creates a `.score/cron-patrol.halt` sentinel. Option (b) saves cron tokens while waiting.
