# #1610 — literal escape-alias tracking must be flow-sensitive

Status: landed `3cb1db0c3` (2026-07-14). Backfill TD.

## Mechanism

`mir/escape_analysis.rs` (from d1c94b49b "elide GC tracking for local
literals", #1435) precomputed a whole-function Copy-alias fixed point BEFORE
classifying uses. MIR VRegs are reused as the "current value" slot on variable
reassignment (not SSA), so one Copy-destination VReg legitimately aliases
DIFFERENT literal roots at different program points — the precompute kept only
the LAST root, erasing earlier attributions. A truly-escaping earlier literal
(e.g. via StoreGlobal) got classified NonEscaping → built with the
`_untracked` FFI constructor (skips `gc_track`) → its real lifetime outran GC
bookkeeping → runtime state corruption. Minimal repro: `x=[1]; x=[2]` hangs.

## Invariant

Escape classification of a literal use must consult the alias map AS OF that
use's program point. Any per-VReg map over non-SSA MIR must be maintained in
program order, never precomputed globally.

## Fix pattern

Single forward pass: update the alias map at each `Copy` in program order and
classify each use against the live alias; `propagate_copy_aliases` deleted.

## Verification contract

`x=[1]; x=[2]` / dict analog probes; `_regression/builtin-libs/list_methods/reentrancy.py`
(was hang, 0.07s green); `escape_analysis` lib tests 19/19; A/B sweep of
list_methods+contextvars+copy (136 fixtures) — only intended flips.
