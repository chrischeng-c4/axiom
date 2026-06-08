---
topic: sdd-dogfood-62pct-plateau
date: '20260427'
project: cclab
branch: main
---

## Status

223/359 files dogfooded (62.12%) — sdd 212/333 (63.66%), score 11/26 (42.31%). 1778 sdd lib tests green. **Plateau hit** — 6 remaining files all blocked by 3 generator gaps + test-fixture-only files. Cron deleted. Drive started at 110/359 (30.64%) on 2026-04-26 morning, gained ~113 files over ~30 fires before plateau.

## Findings

- **All "easy" generator gaps already fixed**: empty derive list, unit-struct replaces, narrow serde imports, `serde_untagged`, unit-struct shorthand `pub struct X;`, schema-yaml parse warnings, `serde_deny_unknown`, `is_other` variant, `x-clap-command`, variant-field `x-clap-arg`, subset-aware use-import dedupe, per-variant arbitrary `serde(rename = ...)`. All committed mainline. See memory `project_dogfood_progress_2026-04-25.md`.
- **YAML authoring gotcha**: bare `Null` parses as YAML null literal (not the string "Null"), silently drops the variant from codegen. Always quote ambiguous variant names like `"Null"`, `"True"`, `"Yes"`. Description strings with unquoted colons also break YAML parsing — always quote.
- **Lifetime generics work**: `x-rust-generics: ["'a"]` + reference field types via `x-rust-type: "&'a T"` verified end-to-end on `context_builder/mod.rs`.
- **Closure trait-objects work**: `Option<Arc<dyn Fn(...) + Send + Sync>>` passes through `x-rust-type` cleanly (verified on `agents/crr.rs`).
- **Custom `impl Serialize`/`Deserialize` coexistence pattern**: spec author omits `Serialize`/`Deserialize` from the derive list; manual impls outside the CODEGEN block handle it. Verified on `models/state.rs` StatePhase (60+ legacy backward-compat aliases).
- **Score CLI subcommand pattern works**: `IssuesArgs.command: IssuesCommand` with `x-clap-command: "subcommand"` + Subcommand enum with tuple variants wrapping per-subcommand Args structs. Verified on `score/cli/issues.rs` top-level (the 19 nested Args/Filter types still hand-written but unblocked when needed).
- **Cron pace**: aggressive 35/fire target was over-ambitious. Realistic with current spec-authoring overhead is 4-6 files/fire (simple) or 1-3 files/fire (multi-type with tricky shapes). Spec-template + sed-substitute trick saves time on sibling families (axum/express/deploy/react pattern).

## Done

- 6 generator-gap fix commits applied & rebuilt: `e514bc7b` (deny_unknown + is_other + x-clap-command + variant-field x-clap-arg), `7241c2e3` (subset-aware import dedupe + per-variant rename), plus earlier `e354dc8e`, `78fe13f1`, `3865d788`, `e54a546a`. All committed mainline, all tested.
- ~113 files dogfooded across the session — see memory `project_dogfood_progress_2026-04-25.md` for the current per-area breakdown.
- Helper script at `/tmp/dogfood_lifecycle.sh` (session-only, recreated each session) wraps full SDD CRRR + TD lifecycle.
- Cron `b7abd21a` (recurring 30-min fire) deleted on 2026-04-27 after 16 consecutive no-op fires.
- Memory updated:
  - `project_dogfood_progress_2026-04-25.md` — full state, all gap entries (resolved + remaining), final-ceiling notes
  - `project_codegen_ecosystem_roadmap.md` — cclab arsenal (4 layers) + TS+React + Python+FastAPI+DB priorities
  - `MEMORY.md` index points to current 223/359 (62.12%)

## Next

1. **Fix 3 remaining generator gaps** (~30 min total — all small additions to `crates/sdd/src/generate/gen/rust/schema.rs`):
   - Add `x-serde-alias: ["alt1", "alt2"]` per-field extension → emits `#[serde(alias = "alt1", alias = "alt2")]`. Unblocks `crates/sdd/src/models/frontmatter.rs` (TasksFrontmatter.id has alias for backward-compat `change_id`).
   - Add `flatten: true` per-field option in `x-rust-enum.variants[].fields` → emits `#[serde(flatten)]` on variant struct fields. Unblocks `crates/sdd/src/generate/spec_ir/types.rs` (SpecIR enum, 11 occurrences).
   - Add `x-serde-skip: true` field-level extension → emits `#[serde(skip)]`. Unblocks `crates/sdd/src/issues/types.rs` (Issue.slug + Issue.body) and `crates/sdd/src/models/change.rs`.
2. After gap fixes, run `cargo test -p sdd --lib` (must show 1778 passed) + rebuild score: `cargo build -p score-cli && cp target/debug/score ~/.cargo/bin/score && codesign -s - -f ~/.cargo/bin/score 2>/dev/null`.
3. Resume cron with `CronCreate` (same prompt as before) or run 2-3 manual fires to sweep the 4 unblocked files. ETA to 100%: ~2 hours after gap fixes.
4. Decide on `fillback.rs` + `analyze/mod.rs` (test-fixture-only files): either accept as "complete by definition" (mark coverage as 225/357 with these excluded) or add empty CODEGEN markers as a convention.
5. (Optional) Finish the 19 nested Args/Filter types in `projects/score/cli/src/issues.rs` for full per-type coverage — one focused session, all gap-free since the top-level migration verified the pattern.

## Criteria

- [ ] All 3 serde gap fixes shipped: search `git log` for commit messages mentioning `x-serde-alias`, `flatten`, `x-serde-skip`
- [ ] `cargo test -p sdd --lib` passes (1778 tests)
- [ ] Coverage advanced past 223/359: `find crates/sdd/src crates/score/cli/src projects/score/cli/src -name "*.rs" 2>/dev/null | xargs grep -l CODEGEN-BEGIN 2>/dev/null | wc -l` returns > 223
- [ ] Memory snapshot `project_dogfood_progress_2026-04-25.md` updated with new coverage line + resolved-gap entries for the 3 fixes
- [ ] `cargo build -p score-cli` succeeds and binary copied to `~/.cargo/bin/score`
- [ ] All 6 previously-blocked files have CODEGEN markers OR are explicitly marked "test-fixture-only" / "skip" in memory
