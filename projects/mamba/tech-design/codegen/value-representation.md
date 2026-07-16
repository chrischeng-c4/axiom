# Value representation — the boxing contract at lowering and the JIT frame boundary

Codegen's contract for whether a VReg or return value is raw or already
boxed (`MbValue`), and where that's cached, synced, or re-derived.
NaN-box layout/tags are `memory/ARCHITECTURE.md`'s `MbValue(u64)`; this
doc owns the raw-vs-boxed decision at HIR→MIR lowering and the JIT-frame
dispatch boundary.

## The core convention: `Any` means already-boxed

`box_operand(vreg, ty_id)` (`hir_to_mir.rs:13396-13416`) boxes `Ty::Int`/
`Float`/`Bool` via `mb_box_int`/`mb_box_float`/`mb_box_bool`; everything
else — `Any` included — returns `vreg` unchanged ("already NaN-boxed
(str, list, etc.)", per its own comment). 148 callsites feed values into
Any-typed positions (attrs, container elements, `StoreGlobal`, capture
cells, dynamic-call args) through this function — every one trusts an
Any-typed value is already boxed.

## Hazard: `sym_to_vreg` has no raw/boxed tag (tracked: #1794)

`sym_to_vreg: HashMap<SymbolId, VReg>` (`:1573`) caches one VReg per
symbol — params, loop vars, comprehension/pattern/exception bindings,
plain assignments (~40 write sites) — with no raw/boxed record. A
module-scope accumulator written concretely (`total = 0`, `Assign`
first-assignment arm, `:3906-3921`) caches the RAW pre-box VReg; a later
occurrence typed `Any` by the checker (`total += <Any-typed>`) then reads
via plain `HirExpr::Var` (`:8877`, cache hit `:8900`) and hands that same
raw VReg to a callsite expecting boxed — `box_operand` treats `Any` as
already-boxed, so NaN-box decoding misreads raw `0i64` as float `0.0`.

Fix: `global_synced_syms: HashSet<u32>` (`:1580`) — SymbolId.0 of
module-scope Vars whose every write mirrors the local `Copy` with a boxed
`StoreGlobal`. Populated ONLY at the two plain-`Assign` write paths
gated on `self.in_module_scope`: the re-assignment arm (`:3898-3904`) and
first-assignment arm (`:3913-3919`). `HirExpr::Var` now reloads via
`LoadGlobal` instead of trusting the cache when `*ty == self.tcx.any() &&
global_synced_syms.contains(&sym.0)` (`:8918`). Loop/comprehension
bindings share `sym_to_vreg` but never join this set — reloading would
read uninitialized global storage (the regression the first fix hit,
with nested comprehensions); unrelated to the pre-existing
`module_reload_global_syms` (`:1746`, unconditional nested-function
`global` reload, not gated on type).

## Hazard: boxed-return registry at the JIT frame boundary (tracked: #1794)

A function whose return is already boxed must be flagged, or a
dynamic-dispatch caller re-boxes it. Chain: (1) at lowering,
`register_boxed_return_bodies` (`hir_to_mir.rs:222-303`) pre-scans each
body's `Return` terminators and, when every returned VReg is provably
not raw `Int`/`Bool`, calls `register_boxed_return_symbol`
(`module.rs:1213`); (2) at JIT finalize, the flagged SymbolId's pointer
is promoted via `register_boxed_return_func` (`module.rs:1225`); (3)
dynamic-call boundaries with no static ABI (spread/kwargs, dunder
dispatch, thread dispatch) consult `is_boxed_return_func(addr)`
(`module.rs:1234`) before re-boxing. `dispatch_thread_jit_frame`
(`asyncio_mod.rs:803-881`) is one such boundary: when `is_boxed_ret` is
false it runs `mb_box_int(raw_result.to_bits() as i64)` — if the callee
actually returned boxed, this misreads it as raw, the same corruption
family as the `sym_to_vreg` hazard above. A hand-written Rust stand-in
for a "JIT'd" function skips step (1), so it must call
`register_boxed_return_func` itself: `asyncio_mod.rs:2041`'s
`test_pseudo_jit_add` (returns already-boxed `MbValue::from_int`).

## Cross-references

The checker-side widening exemption limiting how often `Any` is reached
(the fix above fires only on residual cases) is
`preregister_loop_reassign_counts` (`types/check.rs:776`) — see
`type-system/walls-and-widening.md`.

## EC surface

No dedicated fixture: a raw/boxed mismatch surfaces only as a byte-diff
in the conformance gate (`cargo test -p mamba --release --test
conformance`) once a corrupted value reaches `print`/repr.
