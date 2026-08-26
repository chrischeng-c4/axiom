# CONTRIBUTING — mamba engineering doctrine

Session-independent operating knowledge for anyone (human or agent) working on
mamba. The orchestration loop lives in tracker issue **#1134** (bootstrap
prompt pinned there); this file carries the repo-native rules and doctrines.
Where this file and the tracker disagree, **the tracker is truth**.

## Build discipline

```bash
CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" \
cargo build --release -p mamba
```

| Rule | Why |
|------|-----|
| Foreground builds only; never background a build then stop | stalled-agent class failure; re-run the identical command if the harness backgrounds it |
| Binary lands at **workspace root** `target/release/mamba`, not `apps/mamba/target/` | wrong-path runs test a stale binary |
| Ship profile (no env overrides) only for perf-pin **gating**; fast-verify for everything else | pins README documents the split |
| Concurrent sessions share `target/` — copy the binary to a scratch path before long verification runs | mid-verify rebuilds swap the binary under you |
| Debug-assertions variant for memory bugs: `CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true` + separate `--target-dir` | `rc.rs debug_validate_obj` catches double-frees deterministically (150/150 bar, see #1018/#1027) |

## Working-tree discipline (the braid)

- **NEVER `git add -A`. Never commit unless the user explicitly asks.** The
  tree deliberately carries held, uncommitted WIP; verified fixes stay
  in-tree and land together (procedure: issue **#1102**).
- Preserve every hunk you did not author **byte-for-byte**.
- Surgical commits (when asked): filter hunks via `git diff` → `git apply
  --cached`, then **build the committed state in a fresh worktree** before
  pushing — a committed file referencing an uncommitted symbol does not
  compile (this has happened; twice).

## Test harness

- Conformance: `tests/harness/cpython/tools/sweep.py` with
  `MAMBA_BIN=<workspace-root>/target/release/mamba`, absolute paths, never
  `--store`/`--all`. `.cache/sweep/failures.txt` goes stale fast — treat as a
  hint, confirm pre-existing failures by revert-and-rebuild isolation.
- Oracle: `python3.12`. 3rd-party fixtures need the dedicated venv
  `tests/cpython/.cache/oracle-env/bin/python3` (bare python3 lacks the libs).
- Perf: `perf_pin.rs` pins (run as `run_pin::<name>`), baselines via
  `tools/perf_baseline.py` (+ JSONL export, host stamping); Go-competitiveness
  suite at `tests/harness/go_suite/` (`tools/suite_bench.py`).
- Fixture authoring: `tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`.

## Test architecture (four planes)

| Plane | Form | Verifies | A red means | Speed |
|-------|------|----------|-------------|-------|
| A. Runtime primitive | Rust calls `mb_*` APIs directly | internal invariants Python cannot see: refcount pairing, GC, dict resize vs iteration, exception-chain assembly, iterator state machines | a runtime-function bug | µs, parallel |
| B. Compilation semantics | minimal Python snippet → `jit_capture` + `assert_output` | one construct per test through the full pipeline (typecheck → HIR → MIR → codegen → run) | a pipeline bug | ms, serialized by `JIT_LOCK` |
| C. Type-checker surface | fixture source + directive-aware helper | `# mamba-strict-type:` walls: expected static reject → assert type error; expected runtime TypeError → assert exception type | a checker coverage gap | ms |
| D. E2E gate | fixture files + external harness vs **real CPython 3.12 oracle** | semantic divergence; the only plane with an oracle | product divergence | slow; mandatory pre-release |

Rules that keep the planes honest:

- **Verification power is mandatory.** Any in-process harness must check the
  pending Python exception after execution and panic with the traceback
  (driver's `run` path is the reference pattern). Tests that discard harness
  output (`let _ = jit_capture(...)`), swallow panics (`catch_unwind` in a
  harness), or run with no assertion verify nothing and count toward no
  coverage statistic.
- **Snapshot lock, not `include_str!`.** Planes B/C embed fixture source as a
  generation-time copy (the lock). Never `include_str!` from `tests/` — a lock
  that silently inherits upstream edits is not a lock, and it inverts the
  dependency direction. Drift is caught by a linter comparing embedded source
  against machine-readable provenance; re-sync is an explicit, reviewed act.
- **Regression rule.** Every bug fix lands with one minimal test in the plane
  where the bug lived. Unit coverage grows through regressions, not upfront
  enumeration.
- **Environment-dependent fixtures stay in D.** surface/real_world/security/
  3rd-party fixtures are meaningless in-process.
- **Plane D is authoritative.** No unit-plane result substitutes for the
  oracle gate; release judgment reads D only. Weakening a harness to turn
  tests green (swallowed panics, discarded output, loosened assertions) is a
  violation — audit harness files by diff, every time.

## Verification rules (before closing any issue)

| Rule | Trap it prevents |
|------|------------------|
| A/B against the oracle from a **clean cwd** | stray `.py` files shadow stdlib for CPython (a scratch `inspect.py` broke `bdb` imports) |
| **Matrix-verify**: all operators, both operand positions, both class kinds (plain + builtin-subclass), single- and multi-arg print | agents verify one case; families hide (`+` worked, `-` raised, `*` returned None) |
| Probe with **runtime-computed values**, never literals | constant folding masks codegen bugs entirely |
| `2>/dev/null` stdout-only compare while other agents build | debug stderr pollutes diffs; the grading harness is stdout-only anyway |
| Memory claims: `/usr/bin/time -l` at **two scales** | superlinear/leak classes look flat at small N |
| Perf claims: run the pins yourself | ratios drift under box contention |
| Open changes through `/aw-grill-me-to-change`, never by hand | `change.py create` refuses a body that fails the GHAN schema and renames the staged file to the number it got back; a hand-filed issue is one nothing validated |

## Doctrines (check these FIRST when diagnosing)

| Doctrine | Canonical issues |
|----------|------------------|
| All refcount emission/accounting sites must share **one ownership rule set**; every independent site has produced a p0 | #1018 #959 #1027 #1091 #1013 #1132 → centralization: #1033 |
| Binop/unary fallbacks must be **NaN-box TAG-complete**; enumerate accepted tags before suspecting lowering | #1025 #961 #1090 |
| `raw_ints` static tags **lie** for call-results/params; fast paths must self-verify at runtime (overflow check or tag test) or go extern | #1090 #1131 |
| Address-keyed registries fail **both directions**: identical bodies fold (ICF), one body splits across CGUs | #954 #962 #1040 (fold) #1065 (split) |
| **Truth-table the oracle first** — before trusting the issue text, the dispatch prompt, or your own framing | #1050 #1025 #1027 (three same-day framing corrections) |
| Inference-default gaps read as IEEE-754 bit garbage (raw int slot through boxed path); check detectors and symbol allocation, not per-shape patches | #953 #977 #1015 #1064 |
| Inline int = 48-bit signed NaN-box payload `[-(2^47), 2^47-1]`; BigInt beyond. `line!()/column!()` inside `macro_rules!` repetition is **constant** — fingerprint with `stringify!($name)` | #1090 #962 |

## Dispatch pattern (agents)

One `mamba-dev` agent per **file lane** (`jit.rs`+`codegen/mod.rs` / `class.rs`
/ `builtins.rs` / `dict_ops.rs` / `types/` / `lower/`). Prompt = issue# + the
ticket's recipe + FOREGROUND/no-commit/preserve-braid boilerplate. Agents
silent >30 min with no live processes: `SendMessage`-resume with a
foreground-blocking instruction. STOP conditions: blast radius >5 classes →
revert and inventory (#1037/#1069); architecturally deep → design note +
dependency, never a private gate (#1043).
