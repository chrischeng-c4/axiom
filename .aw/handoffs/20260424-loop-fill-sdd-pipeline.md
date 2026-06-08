---
topic: loop-fill-sdd-pipeline
date: '20260424'
project: main
branch: main
---

## Status

Loop-fill refactor for `score issues create` + `score td create` landed on main in two slices. Both CLIs + agent prompts updated; retry cap (cap=2 + mainthread CLI takeover + arbitrate@3) is in place. NOT YET end-to-end tested with a real subagent dispatch — the full loop hasn't been exercised by an actual `Agent()` call this session.

## Findings

- Hooks are **permanently unreliable** for async `Agent` dispatch (Anthropic #27755 closed "not planned"). Mainthread orchestration is the design, not a workaround. Documented in memory `project_async_task_skips_hooks.md` + `project_crr_mainthread_plus_cron.md` (both updated).
- Existing `revise` flow is already "whole-body input + per-section merge" hybrid (via `--section <csv>` accepting CSV like `requirements,scope`). No revise refactor needed; the merge layer protects unflagged sections from regression.
- `score issues` uses fixed 3-section contract (requirements/scope/reference_context) → envelope carries literal `sections: [...]` list.
- `score td` sections are author-chosen via `fill_sections` frontmatter → envelope just carries `mode: "loop"` signal and subagent picks sections itself.
- sdd + score TD specs: **format-wise all green** (0 findings) after the Mermaid Plus `entry:`/`initial:` fix session. All 399 code-side findings are `Uncovered` — reflects "hand-written code, never dogfooded codegen", NOT spec bugs.
- R3f (codegen-ready) fires only on the FIRST matching block of each section type per file — known quirk of the current implementation. Fixing the first `logic`/`state-machine` block per file is enough to clear the validate report.

## Done

- **Slice 1 (commit `e06b4114`)** — issues create loop-fill + retry cap
  - `projects/score/cli/src/issues.rs`: envelope emits `sections: [requirements, scope, reference_context]`; new `run_fill_loop` + `try_advance_fill_section` replace `handle_fill_milestone`; retry cap messages encode `retry=N [takeover|arbitrate]` qualifier. *Tested via smoke: fresh issue → validate → 3 Fill-* commits + reviewer dispatch confirmed in git log.*
  - `crates/sdd/src/issues/types.rs` + all 3 backends: new `fill_retry_count: Option<u8>` field. *Tested via 38/38 sdd issues:: tests.*
  - `.claude/agents/score-issue-author.md`: rewritten for loop contract; turns 12→30. *Not tested end-to-end with real subagent dispatch.*
  - `.claude/skills/score-issue/SKILL.md`: mainthread loop mermaid + retry-cap rules updated. *Untested.*

- **Slice 2 (commit `82f4d59b`)** — td create loop-fill + per-section merge + retry cap
  - `projects/score/cli/src/td.rs`: `CreateArgs.section: Option<String>`; new `merge_spec_section` helper matches by `<!-- type: X -->` annotation; payload at `.score/payloads/<slug>/<section>.md` deleted after merge; `td init` envelope carries `mode: "loop"`; `handle_create_milestone` gains retry-cap parity with issues side. *Tested via 3/3 new unit tests (replaces/appends/preserves frontmatter) and 1746/1746 sdd tests.*
  - `.claude/agents/score-td-author.md`: rewritten for loop contract; turns 30→40. *Not tested end-to-end.*

- **Previous session commits** still on branch (from earlier in same session):
  - `e69d87d1` — Mermaid Plus frontmatter fix (9 sdd + 5 score findings → 0).
  - `81b61332` — fillback fn body → Logic section + logic generator async/params.
  - `67ca6db9` — logic generator 20%→80%.
  - `e3f59be0` — fillback struct fields + enum variants → Schema section.
  - `bf447639` — fillback TD-format output.

## Next

1. **End-to-end smoke test of loop-fill**. Do a real `score issues create` → dispatch author via `Agent()` → verify subagent actually loops all 3 sections → mainthread validate advances phases → reviewer dispatched. Same for TD flow. If the subagent prompt has issues, update it.
   ```bash
   score issues create --title "smoke: loop-fill verify" --type enhancement --label "crate:sdd"
   # Follow the envelope — dispatch Agent(subagent_type=score-issue-author, prompt=...)
   # After agent returns: score issues validate <slug>
   # Expect: 3 Fill-* commits + reviewer dispatch
   ```

2. **Force a retry path** to exercise the cap-2 logic:
   - Corrupt a section mid-flow so validate rejects it
   - Verify envelope message contains `[retry=1]`, then `[retry=2 takeover]`, then `[retry=3 arbitrate]` across runs

3. **Legacy cleanup backlog** — `score workflow`, `score artifact`, `score run-change`, `score revise-artifact`, `score status`, `score handoff` (some), `sdd::workflow/*`, `sdd::state/*`, `sdd::models::state::StatePhase`, `sdd::tools/workflow_*`. See memory `project_sdd_legacy_cleanup_backlog.md` for the full list. ~3000-5000 lines, 44 files directly touched. Needs its own issue + careful regression test.

4. **Jet crate format fixes** — 5 R3f findings still in `.score/tech_design/crates/cclab-jet/` + `crates/jet/testing/*`. Same shape as the sdd/score fixes already done; just not this session's scope.

5. **TD specs dogfood candidates** — the user's north-star is flipping hand-written code to codegen. With the refactor done, natural pilot is a small pure-data type with a mature schema spec (`projects/mamba/mambalibs/httpkit/src/health.rs::HealthStatus` was flagged as already cleanly codegen'd — it's the reference case). Pick a small surviving type from the "keep" list in the legacy backlog memory.

## Criteria

- [ ] `cargo test -p sdd --lib` passes (expected: 1746/1746 green, 3 ignored)
- [ ] `cargo test -p score-cli --lib td::tests::` passes (expected: 3/3 green — merge_spec_section tests)
- [ ] `cargo build -p score-cli` clean
- [ ] `score issues create --title "smoke" --type enhancement --label "crate:sdd"` envelope contains `"sections": ["requirements", "scope", "reference_context"]`
- [ ] `score td create --help` lists `--section <SECTION>` flag
- [ ] `grep "fill_retry_count" crates/sdd/src/issues/types.rs` matches
- [ ] `git log --oneline main | head -3` contains `82f4d59b` and `e06b4114`
- [ ] no regressions (manual): run an existing `score issues create` flow and verify the fixed surface still works
