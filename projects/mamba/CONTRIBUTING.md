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
| Binary lands at **workspace root** `target/release/mamba`, not `projects/mamba/target/` | wrong-path runs test a stale binary |
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

## Verification rules (before closing any issue)

| Rule | Trap it prevents |
|------|------------------|
| A/B against the oracle from a **clean cwd** | stray `.py` files shadow stdlib for CPython (a scratch `inspect.py` broke `bdb` imports) |
| **Matrix-verify**: all operators, both operand positions, both class kinds (plain + builtin-subclass), single- and multi-arg print | agents verify one case; families hide (`+` worked, `-` raised, `*` returned None) |
| Probe with **runtime-computed values**, never literals | constant folding masks codegen bugs entirely |
| `2>/dev/null` stdout-only compare while other agents build | debug stderr pollutes diffs; the grading harness is stdout-only anyway |
| Memory claims: `/usr/bin/time -l` at **two scales** | superlinear/leak classes look flat at small N |
| Perf claims: run the pins yourself | ratios drift under box contention |
| Confirm the issue number after `aw wi create` | the too-large phrase check (`whole`/`entire`/`everything`/`from scratch` **substrings** — "wholesale" trips it) fails silently in batches |

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
