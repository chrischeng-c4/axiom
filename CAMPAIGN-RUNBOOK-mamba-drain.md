# mamba Phase-B single-lane drain campaign (runbook)

> PAUSED 2026-07-19 by user after the #1950 landing (121f45888). 14
> dispatches: 12 landed, #1985 parked (heisenbug, diagnosis on issue),
> #1943→#1981→#1982 opened the C2 axis. Ratchet at pause: cargo failed
> ≤1,006 nominal (last reading 1,007 incl. fixtures added by a
> CONCURRENT non-campaign lane — 6 of those fail and are NOT campaign
> regressions; verify attribution before chasing), real-PASS floor
> 19,637. Resume = SELECT from the open t1-t3 queue as usual; next in
> order: #1952 → #1953 → #1986/#1987/#1989/#1992 → #1976/#1977 →
> #1964/#1965/#1971/#1980 → C2 (#1960/#1513/#1636) → p3 tail.

DO NOT COMMIT THIS FILE. Re-read it at the start of EVERY turn — conversation
memory compacts; this file and the tracker are the only durable state.

You are the ONLY lane working this repo: do implementation directly
(Read/Edit/Bash), no subagent dispatches, no parallel cargo runs.

## Queue definition

- USER DIRECTIVE 2026-07-18: focus = tiers 1-2-3 全通 (strict-type,
  language-core, builtins). 全通 = correctness AND perf (C2) — user
  confirmed 2026-07-18「功能跟perf都要」. C2 chain: #1960 (t2 pin
  baselines) / #1513 (t3 pin baselines) / #1636 (profiling enabler, p2) /
  #1943 (gate-hygiene bug) are IN queue; #1512 is the t2 C2 ledger
  (parked, receives ratio tables, closes last). Correctness bugs still
  outrank C2 items at equal priority (bug > enhancement rule unchanged).
  QUEUE = open aw work-items, project mamba, type
  bug/enhancement/refactor, restricted to labels
  `mamba:tier-1-strict-type` / `mamba:tier-2-language-core` /
  `mamba:tier-3-builtins`. A new unlabeled runtime/parser/builtins bug
  belongs to t2/t3 — add the tier label on SELECT, then it queues.
- Tier 4-7 atoms (#868 #1100 #1101 #1104 #1105 #1120 #1223) and harness
  tooling (#1939) stay open but OUT of the selection order until t1-t3 is
  drained. Exception: #1515 (t7a cookiejar residual) stays out too.
- EPICS ARE OUT OF SCOPE — strategy items for human planning. Never fix,
  atomize, or "make progress" on an epic.
- PARK = `gh issue comment <n> --body "PARKED(drain-campaign): <reason + evidence>"`
  then `gh issue edit <n> --add-label blocked:hitl`. Parked items leave the queue.

## Turn skeleton (exactly ONE work unit per turn)

1. **SELECT** — `aw wi list --project mamba`; pick highest-priority open
   non-parked atomic: p0>p1>p2>p3; bug>enhancement>refactor at equal
   priority; oldest first. Print the pick.
2. **SCOPE-CHECK** — `aw wi show <n>`. Roadmap-sized, decision-blocked, or
   unbounded AC → PARK, end turn.
3. **REPRODUCE** — verify-first: reproduce the claim live (targeted
   `cargo test -p mamba` / fixture via release binary). Cannot reproduce →
   close with evidence comment, end turn.
4. **READ FIRST** — before editing, read the owning domain's
   `projects/mamba/tech-design/<domain>/ARCHITECTURE.md` + topic docs.
   Always check `stdlib/module-hazards.md` (DictKey/`dict_get_exact_str`,
   thread-local CLASS_REGISTRY, iterator-handle traps),
   `codegen/value-representation.md` (raw-vs-boxed), and
   `type-system/walls-and-widening.md` (walls) when in those areas.
   Do NOT edit tech-design/ or external-contracts/ — put knowledge deltas
   in the issue's evidence comment for the guardian.
5. **FIX** — minimal root-cause fix, matching surrounding style.
6. **VERIFY** — targeted test, then the owning module's tests. Full gates
   only at checkpoints (below).
7. **LAND** — stage ONLY files you touched (`git add <paths>`; NEVER
   `git add -A` — the tree carries deliberate uncommitted WIP). Commit
   `fix(mamba): <what> (Refs #<n>)`. Close the WI with an evidence comment
   (before/after readings). Newly discovered bugs → `aw wi create` a NEW
   item (never fold into the current one); it joins the queue.
8. **SCOREBOARD** — end EVERY turn with exactly this line (the goal
   evaluator reads it):

   `SCOREBOARD open_atomic=<N> parked=<M> | last_gate: release-lib <F> failed excl meta-gate (turn <T>) | this_turn: #<n> <closed|parked|cannot-reproduce-closed|created #<m>>`

   `open_atomic` = fresh count from `aw wi list --project mamba` of open
   bug/enhancement/refactor NOT labeled blocked:hitl, restricted to the
   tier-1/2/3 labels per the queue definition above.

## Checkpoints (every 5th turn, and ALWAYS before trusting open_atomic=0)

- `cargo test -p mamba --release --lib` — record failed-count EXCLUDING the
  `cpython_ported::*` meta-gate cluster (those self-heal; never touch them).
- `cargo test -p mamba --release --test conformance 2>&1 | tail -3` —
  failed must stay ≤ 1006 (2026-07-18 night ratchet @ 10aaec9ad:
  cargo 1,006 failed; verdict sidecar real-PASS 19,634 / FAIL 966 /
  DIVERGE 20 / XFAIL 25,595 — real-PASS must never DROP below its last
  checkpoint reading; batch noise ±10 documented in #1942 — borderline
  readings need a double run + fixture diff; known ±1-2 flickers:
  tabnanny pair + weakref shared-proxy (#1985, PARKED heisenbug — never
  chase it in a drain turn); do NOT use repo failures.txt, it is a
  stale 2026-07-14 artifact).
- Crash A/B discipline (#1985 second-round finding): on this box's
  fast-verify profile (codegen-units=16, non-LTO), build-to-build
  layout noise swamps single-rebuild crash-rate A/B. Any crash-fix
  causality claim needs ≥3 independent rebuilds per arm, or a static
  ownership-graph audit instead (MallocStackLogging suppresses this
  bug class entirely — observer effect, unusable).
- `cargo test -p mamba --release --test perf_gate_report -- --nocapture`
  — per-pin classes must not regress vs the 2026-07-18 night reading:
  pass 16 / ratio-fail 84 / no-baseline 26 / fixture-error 2 (the 2 =
  #1983-owned). A pin dropping from pass/ratio-fail INTO fixture-error
  is a correctness regression — fix THIS turn.
- Once #1960/#1513 land pins: the perf_pin gate joins this checkpoint —
  all registered pins green at their recorded floors; a floor regression
  is fixed THIS turn like any other gate regression.
- Any regression vs the previous checkpoint = fix it THIS turn before
  selecting anything new.

## Anti-thrash

2 failed fix attempts on the same item → PARK with your evidence so far and
move on. Never a third attempt in this campaign.

## Out of scope

- The 2 `cpython_ported` meta-gate lib failures (self-heal cluster).
- All epics. Anything outside projects/mamba. Editing knowledge docs.
- This runbook file itself and any other untracked/WIP files: never stage.
