# Object lifetime — escape analysis, GC tracking, and protocol refcounts

The rules that keep NaN-boxed heap values alive exactly as long as needed.
Getting any of them wrong shows up as a hang or SIGTRAP anywhere in the
corpus — this domain owns those symptoms wherever they appear.

## Escape analysis licenses GC-tracking elision

`mir/escape_analysis.rs` decides which literal allocations can use the
`_untracked` FFI constructor (skipping `gc_track`). The analysis MUST be
flow-sensitive: MIR VRegs are reused as a variable's "current value" slot on
reassignment (not SSA), so one Copy-destination VReg legitimately aliases
DIFFERENT literal roots at different program points. A whole-function
precompute keeps only the LAST root and misclassifies an earlier
truly-escaping literal (e.g. via StoreGlobal) as NonEscaping → its lifetime
outruns GC bookkeeping → corruption. Invariant: classify each use against the
alias map AS OF that use's program point; maintain any per-VReg map over
non-SSA MIR in program order, never globally. (Minimal witness of a violation:
`x=[1]; x=[2]` hangs.)

## The cycle collector is real

`gc.rs::collect` is an enabled thread-local 4-phase trial-deletion collector
(threshold ~10k). It is not a stub. `gc_clear_all_state` must never collect.
The `__main__` epilogue release sweep is intentionally OFF (suspected BigInt
inner-Vec double-free, tracked: #1663) — do not "fix" it by re-enabling.

## With-protocol refcount contract

The with-exit lowering double-releases the context value (explicit release +
`Copy`'s auto release-before-overwrite, #1129 R2); every `mb_context_enter`
branch compensates with a retain. Invariant: any `__enter__` — native or
synthesized — must leave both the receiver AND the returned value's refcount
satisfied. The trap: an `__enter__` returning a NON-self value (e.g.
`TemporaryDirectory.__enter__` returns `name`) needs `retain_if_ptr(recv)`
too; missing it is a use-after-free that SIGTRAPs intermittently — audit every
non-self-returning `__enter__`.

## Attribution rule for this domain

Intermittent crashes defeat single-sample bisects (a UAF can present green on
one run). A/B with repeated sampling (and MallocScribble in debug) before
blaming a commit.

## Known carve-outs

Per-iteration rebind leak carve-out (#2111, per-loop-back-edge release sweep
for fresh VRegs — documented only in a jit.rs comment). Generator ObjKind=14
GC coverage is undefined until the ObjData variant lands (#2182).

## EC surface

`behavior|surface|type/std-libs/gc`, `_regression/core/stability` soaks;
corpus-wide: any hang (alarm-wrap runs) or SIGTRAP; mir escape-analysis lib
tests.
