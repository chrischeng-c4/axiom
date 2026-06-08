# jet/ source-tree reorganization

Status: pass 1 done
Started: 2026-05-21
Completed: 2026-05-21
Branch: project-jet

## Result summary (pass 1)

- `src/` top-level `.rs` files: **44 → 10** (only `main`, `lib`, `cli`,
  `build_clean`, `build_target`, `ci_summary`, `report_package`,
  `rerun_manifest`, `result_envelope` remain).
- 4 new context modules: `src/e2e/` (21 files), `src/pm_report/` (8 files),
  `src/evidence/` (3 files), `src/agent/` (3 files including `mod.rs`).
- 28 file moves: 16 `e2e_*.rs` + 7 `pm_report_*.rs` + 3 `evidence*.rs` +
  2 `agent_*.rs` + `playwright_shim.rs`.
- `cargo build -p jet` green.
- `cargo test -p jet --lib`: 2390 passed, 0 failed.
- All pre-reorg `crate::<group>_<name>` paths still resolve via
  backward-compatibility `pub use ... as ...` aliases in `src/lib.rs`.

## Deferred (require broader branch scope)

P4 — `tests/` grouped into subdirs. Cargo auto-discovers `tests/*.rs` only
at the top level; nesting requires ~60 `[[test]]` entries. Skipped because
existing filename prefixes (`e2e_*`, `pm_report_*`, `fixture_*_tests`,
`*_debug`, `playwright_compat_*`, `jet_test_*`) already group visually.

See `## Deferred work` for crate-boundary reorg ambitions deferred to a
higher-scope branch.

## Goal (scoped)

Reorganize **`projects/jet/src/` and `projects/jet/tests/` internals** following
DDD layering and small-and-many-files conventions. **Public APIs and module
paths visible to external callers stay backward-compatible** via `pub use`
aliases.

## Scope decision

`.aw/config.toml` restricts the `project-jet` branch to `projects/jet/**`
and `.aw/tech-design/projects/jet/**`. Any change to the workspace root
`Cargo.toml` (member list) or to `crates/cclab-cli/Cargo.toml` is blocked
from this branch.

Therefore this pass is limited to:

- `projects/jet/src/**` — module tree reorganization
- `projects/jet/tests/**` — test file grouping
- `projects/jet/Cargo.toml` — only fields inside this manifest
- `projects/jet/docs/**` — this plan

The following ambitions are **deferred** until a higher-scope branch (or
relaxed path policy) is available:

- Collapsing sibling crates (`tauri-shell`, `multi-target`, `tui-renderer`,
  `parity-corpus`, `parity-gate`, `parity-oracle`, `conformance-cli`,
  `manifest-cli`) into the `jet` crate.
- Moving `wasm` / `wasm-renderer-poc` under a `crates/` subdir.
- Moving non-code dirs (`parity/`, `runtime/`) under a `data/` subdir.
- The "single `jet` binary" success criterion.

These are recorded in `## Deferred work` below and stay tracked as future
items.

## Non-goals (this pass)

- No runtime behavior changes.
- No public symbol renames; old paths keep working through re-exports.
- No edits to files outside `projects/jet/**`.

## Success criteria (this pass)

1. `cargo build -p jet` succeeds.
2. `cargo test -p jet` passes (or fails only on the same tests that fail
   pre-reorg — capture a baseline before starting).
3. `projects/jet/src/` top level has at most ~10 `.rs` files. The 28 grouped
   files (`e2e_*`, `pm_report_*`, `evidence_*`, `agent_*`) are folded into
   their respective context subdirectories.
4. New context directories exist with their own `mod.rs`:
   - `src/e2e/` — absorbs `e2e.rs` + 16 `e2e_*.rs`
   - `src/pm_report/` — absorbs 7 `pm_report_*.rs`
   - `src/agent/` — absorbs 2 `agent_*.rs`
   - `src/evidence/` — absorbs 3 `evidence_*.rs`
5. Every relocated module is reachable via both its new path and its
   pre-reorg path. E.g., `crate::e2e_actionability::*` still resolves
   (via `pub use crate::e2e::actionability as e2e_actionability` in
   `lib.rs`).
6. `projects/jet/tests/` is grouped by context: `tests/e2e/`, `tests/pm_report/`,
   `tests/parity/`, `tests/playwright_compat/`, `tests/debug/`,
   `tests/fixtures_runtime/`, with `[[test]]` entries in `projects/jet/Cargo.toml`
   updated to match. Tests still discovered and runnable.
7. The plan document is updated with completion status when finished.

## Approach: backward-compatible group moves

For each file `src/<group>_<name>.rs` (where `<group>` is one of `e2e`,
`pm_report`, `evidence`, `agent`):

1. Move file: `src/<group>_<name>.rs` → `src/<group>/<name>.rs`
2. In `src/<group>/mod.rs`, declare `pub mod <name>;`
3. In `src/lib.rs`, replace `pub mod <group>_<name>;` with
   `pub use crate::<group>::<name> as <group>_<name>;`

This preserves every existing `use crate::<group>_<name>::Foo;` in callers.

For deeper DDD layering inside existing module dirs (`bundler/`, `resolver/`,
etc.), only proceed where the layer split is already obvious in the code.
Aggressive layer introduction is **out of scope** for this pass — flagged as
future work.

## Deferred work (future branches with broader scope)

- D1 Workspace skeleton — move `wasm`, `wasm-renderer-poc` under
  `crates/`; move `parity/`, `runtime/` under `data/`. Requires editing
  `/Cargo.toml`.
- D2 Fold rlib siblings — `tauri-shell`, `multi-target`, `tui-renderer`
  into `src/`. Requires editing `/Cargo.toml`.
- D3 Collapse parity binaries — remove `parity-corpus/main.rs`,
  `parity-gate/main.rs`, `parity-oracle/bin/parity_oracle.rs`; expose as
  `jet parity {corpus|gate|oracle}` subcommands. Requires `/Cargo.toml`
  + sibling Cargo.toml edits.
- D4 Thin conformance/manifest bridges — move implementation into
  `src/conformance/` and `src/pkg/`; reduce bridge crates to linkme shims.
  Requires `crates/cclab-cli/Cargo.toml` and sibling Cargo.toml edits.
- D5 Aggressive DDD layering of every existing context. Requires careful
  inspection of every module; deferred to keep this pass mechanical.
- D6 Single-binary success criterion.

## Verification commands

```bash
cd /Users/chris.cheng/cclab/project-jet
cargo build -p jet
cargo test -p jet --no-run
ls projects/jet/src/*.rs | wc -l    # should drop from ~44 to ~10
find projects/jet/src/e2e projects/jet/src/pm_report projects/jet/src/evidence projects/jet/src/agent -name "*.rs" | wc -l   # ~28
```
