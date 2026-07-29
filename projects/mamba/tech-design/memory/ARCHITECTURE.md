# memory — architecture (as-is, 2026-07-15)

Scope: value representation, refcount contracts, cycle GC, escape analysis, and the
JIT-emitted refcount protocol. Fix-family hazards in this directory
(`object-lifetime.md` §Escape analysis licenses GC-tracking elision,
§With-protocol refcount contract) are cross-referenced, not restated.
Ownership-violation evidence and its four-part detector / control / balance /
audit model live in `ownership-violation-evidence.md`.

## Responsibilities

- NaN-boxed 64-bit value model — `src/runtime/value.rs:MbValue` (float/int48/bool/None/ptr48/func + NotImplemented/StopIter/Ellipsis sentinels).
- Atomic refcounting of heap objects + the NEW/BORROWED/VOID ownership audit for every `mb_*` FFI symbol — `src/runtime/rc.rs` (#1129 header doc is the audit ledger).
- Cycle collection: thread-local, CPython-style 4-phase trial-deletion collector — `src/runtime/gc.rs:collect` (REAL, `enabled: true`, threshold 10_000).
- JIT refcount insertion: pre-write release, Copy-retain, return-epilogue sweep, borrowed-param rules — `src/codegen/cranelift/jit.rs:emit_inst`, `mod.rs:emit_terminator`.
- Literal escape analysis that licenses `gc_track` elision + typed-list layout eligibility — `src/mir/escape_analysis.rs`.
- Non-pointer lifetimes: integer-handle refcounts (`src/runtime/integer_handle_registry.rs`, #2111) and closure handles (`closure.rs:{retain,release}_closure_handle_if_live`).

## Key structures & invariants

| Structure | Where | Invariant |
|---|---|---|
| `MbValue(u64)` | `value.rs` | NaN prefix `0xFFF8…`, 3-bit tag (PTR=0 INT=1 BOOL=2 NONE=3 FUNC=4 NOTIMPL=5 STOP_ITER=6 ELLIPSIS=7), 48-bit payload. Exactly two tagged-prefix bit patterns are genuine floats: `f64::NAN` and `NEG_CANON_NAN` (bit-identical to `from_ptr(null)` — null is never boxed). Ints >48-bit go to heap `BigInt`. |
| `MbObjectHeader{rc: AtomicU32, kind: ObjKind}` + `MbObject` | `rc.rs:466` | `#[repr(C)]`; `IMMORTAL_REFCOUNT = u32::MAX` makes retain/release no-ops AND is reused as a mid-dealloc "being freed" mark (rc is not a valid signal during teardown). |
| `ObjData` | `rc.rs:487` | Str/List/Dict/Tuple/Instance/Set/Bytes/ByteArray/FrozenSet/BigInt/Complex/CodeObject. `Generator = 14` is a HANDWRITE carve-out (#2182) — stdlib "iterators" currently materialize as eager `List`. |
| `MbList = SmallVec<[MbValue;8]>` | `rc.rs:99` | Inline literal storage ≤8 elems (#2517); `MbRwLock` = parking_lot, non-poisoning (#2518); dict = `IndexMap` with SipHash (attacker-controlled keys), instance fields = FxHash. |
| Ownership contract | `rc.rs:17-73` | Every registered symbol returning `MbValue` is NEW (rc=1, caller owns), BORROWED (must `return_owned` before returning), or VOID. Helpers `store_owned/return_owned/store_and_return_owned/release_owned` carry debug refcount-delta asserts. |
| Emitter rules as pure fns | `rc.rs:should_release_local_slot`, `should_retain_borrowed_return`, `should_preseed_loop_owner_slot` | Single authoritative predicate per JIT release/retain decision — change here, not in jit.rs. |
| `GcState` | `gc.rs:16`, `thread_local!` | tracked `FxHashSet<usize>`, threshold 10_000 (#2100 1A), `roots: Vec<MbValue>`. Safepoint API (`gc_safepoint` etc.) = no-op stubs; per-thread GC has no stop-the-world. |
| GC control state | `gc.rs:gc_enable`, `gc_disable`, `gc_is_enabled` | `enabled` is configuration local to the calling thread. A test that changes it snapshots the prior value and restores it with a drop guard so panic/early return cannot leak state. Bookkeeping reset and configuration reset are separate operations; parallel proof uses independent threads, never a suite-wide mutex. |
| gc_track coverage | `rc.rs:new_*` ctors | Lists/dicts/sets always tracked; tuples/frozensets only if any element `value_is_cycle_capable` (rc.rs:402, #2128); str/bytes/bigint never. Untracked-ctor variants (`new_list_untracked` …) exist ONLY for escape-proven literals. |
| Escape classification | `escape_analysis.rs:analyze_literal_escapes` | Flow-sensitive: alias map updated per-`Copy` in program order, classification uses the alias live AT the use point. Never precompute over non-SSA VRegs — see `1610-…md`. |
| With-protocol refcount contract | `class/mod.rs:mb_context_enter` (15458), `hir_to_mir.rs:5265` exit block | Exit lowering double-releases a temporary ctx (explicit `mb_release_value` + `Copy` pre-write release, #1129 R2) ⇒ every `mb_context_enter` branch retains the receiver once; `__enter__` returning non-self must ALSO retain the returned value — see `1627-…md`. |

## Control flow

1. **JIT compile** — `jit.rs:439` runs `analyze_literal_escapes(body)`; `allow_untracked_literals = !is_entry_body` (module scope never elides). `emit_make_list`/`emit_make_dict` (jit.rs:2139/2244) pick `mb_*_untracked` ctors via `literal_can_skip_gc_track` (jit.rs:2100); fixed-arity `mb_list_new_1..10[_untracked]` shims for small literals.
2. **Per-inst refcounting** — `emit_inst` (jit.rs:649): pre-write release of any dest VReg (#1129 R2; skips F64, first writes, raw_ints, params). `Copy` emits `mb_retain_value(source)` (jit.rs:1097) — aliasing means two owners. Loop-carried VRegs are pre-seeded with the None sentinel (jit.rs:516, #1013/#2111) so the pre-write release fires every dynamic iteration.
3. **Return epilogue** — `mod.rs:emit_terminator:1742`: releases definitely-assigned locals (`compute_must_assign` filter) excluding params/raw_ints/retval; a borrowed param returned by value gets one retain (`return self` chains). Entry body (`__main__`, `body.name.0 == u32::MAX`) skips the whole sweep — `release_func_ref` gated `!is_entry_body` (jit.rs:488, #1663 T4c5).
4. **Alloc** — `MbObject::new_*` → rc=1 → conditional `gc_track` → `alloc_count ≥ threshold` triggers `collect()` (gc.rs:86).
5. **Release** — `mb_release` (rc.rs:961): dec → 0 ⇒ weakref notify → Bytes fast-free (never tracked, no children) → store IMMORTAL (defuses re-entrant release from cycles) → `gc_untrack` → `release_contained_values` → drop Box.
6. **Collect** — `gc.rs:collect` (174), 4 phases: ① `gc_refs` = rc snapshot (skip immortal); ② subtract internal refs via `visit_contained`; ③ mark from `gc_refs > 0` + explicit roots; ④ sweep — mark ALL garbage IMMORTAL first, then three passes (weakref notify → release contained → dealloc) so cascading releases between peers are no-ops. Re-entrancy guard: `collecting` flag.
7. **With-statement** — `hir_to_mir.rs:5130`: lower ctx → `mb_context_enter` → exception check → bind alias → body → shared exit block (5265): `mb_context_exit` per ctx reversed; temporary ctx gets `mb_release_value(ctx)` + `Copy(none)`; re-check exception → route to enclosing with / try handler / return-None.
8. **`mb_context_enter` branch table** — `class/mod.rs:15458`, in order: @contextmanager generator → `cm_gen_enter`; file handle → retain+self; Lock/RLock/Condition instance → acquire+retain+self; TarFile dict-stub; tempfile (MUST precede generic dunder — its registered methods are dir()-stubs); `__enter__` dunder (missing `__exit__` on user class ⇒ TypeError; retain obj; extra retain iff result is self); fallback → retain+self.

## Known hazards

- **Precomputed alias maps over MIR VRegs** — VRegs are reused per variable, not SSA; last-root-wins steals escape marks. WHY: mis-elided `gc_track` = state corruption/hang far from cause (`x=[1]; x=[2]` hung). See `object-lifetime.md` §Escape analysis licenses GC-tracking elision.
- **`__enter__` returning non-self without compensating retain** — the with-exit double-release contract silently under-counts. WHY: intermittent SIGTRAP UAF, maskable for weeks by an unrelated type wall. See `object-lifetime.md` §With-protocol refcount contract + comment at `tempfile_mod.rs:1043`.
- **Re-enabling the `__main__` epilogue release sweep** — deliberately off (jit.rs:488). WHY: double-free, oscillating conformance gate (#1663 T4c5; suspect BigInt inner-Vec drop path; rationale block at mod.rs:1751).
- **#2111 carve-out: fresh per-iter VRegs bypass rebind release** (jit.rs:653 comment). WHY: module-scope hot-loop allocations leak monotonically with iteration count; fix surface = per-back-edge release sweep.
- **`gc_clear_all_state` must never run `collect()`** (gc.rs:139 doc; called from `runtime/mod.rs:66` + JIT teardown). WHY: `module_to_value` dicts hold borrowed rc=1 copies — a sweep double-frees them.
- **Test reset must not invent the asserted GC mode**: clearing tracked objects,
  roots, counters, or the re-entrancy bit is bookkeeping cleanup; forcing
  `enabled` at the same time makes the next test depend on helper ordering.
  Tests of `enable()`/`disable()` explicitly establish their starting mode,
  restore the caller's prior mode through RAII, and use a two-thread canary to
  prove thread-local isolation. Global test serialization would hide the
  contract rather than prove it.
- **Integer-handle id collision** — handle bases must be ≥ `HANDLE_MIN_ID = 1<<40` (`integer_handle_registry.rs`). WHY: `MbValue::from_int(1)` is bit-identical to handle id 1; primitive-int releases would corrupt handle tables.
- **`NEG_CANON_NAN == from_ptr(null)`** (value.rs:47-55). WHY: boxing a null pointer would create a value indistinguishable from a float NaN.
- **Wrong `NonEscaping` is a delayed-symptom bug class** — untracked ctors don't crash at alloc; corruption surfaces as hang/SIGTRAP in unrelated fixtures. Bisect with repeated sampling (intermittency defeats single-sample bisects — 1627 lesson).
- **rc is not a valid signal mid-dealloc** — both `mb_release` and GC sweep stamp IMMORTAL before cascading; code reading rc during teardown sees u32::MAX.
- **A green crash reproducer is not ownership evidence by itself** — an extra
  retain can convert a double-release into a leak. Consumer fixes use the
  detector, positive control, leak balance, and site-count reconciliation in
  `ownership-violation-evidence.md`.

## Extension points

- **New container kind**: add `ObjKind` + `ObjData` variant, then extend ALL of `gc.rs:visit_contained`, `rc.rs:release_contained_values`, `rc.rs:value_is_cycle_capable`, and the `debug_validate_obj` kind-range checks (`> 13` in rc.rs:931/1061/1281). The Generator carve-out (#2182, rc.rs:445) is the worked template.
- **New MIR instruction**: must take a position in `escape_analysis.rs:classify_inst_uses` — escape its operands or justify the pass-through arm; and in `jit.rs:emit_inst`'s dest_vreg match for pre-write release.
- **New native context manager**: add a branch in `class/mod.rs:mb_context_enter` AND mirror it in `value_supports_context_manager` (15560); obey the retain contract (retain recv; separately retain the returned value if ≠ recv).
- **New handle-pattern stdlib module**: register `IntegerHandleHooks{retain,release}` and start ids ≥ `HANDLE_MIN_ID`.
- **New typed-list element kind**: `escape_analysis.rs:scalar_typed_list_element_kind`; eligibility stays gated on `NonEscaping`.
- **GC tuning / gc-module surface**: `gc.rs:mb_gc_*` wrappers; threshold via `gc_set_threshold`.
- **GC state tests**: use a scoped configuration guard around mutations of
  `enabled`; keep `reset_gc_for_test` limited to collector bookkeeping. Prove
  opposing enable/disable states can coexist across a synchronization barrier
  on two threads.

## EC surface

Per `projects/mamba/external-contracts/README.md`: memory = `gc/`, `stability/` soaks, **plus absence of hang/SIGTRAP anywhere in the 46k corpus** — symptom jurisdiction, not directory jurisdiction.

- Fixture dirs: `tests/cpython/behavior/std-libs/gc/` (~50, incl. `self_referential_cycle_soak.py`), `surface/std-libs/gc/`, `type/std-libs/gc/`, `_regression/core/stability/heap_churn_soak.py`.
- Regression anchors from the fix TDs: `_regression/builtin-libs/list_methods/reentrancy.py` (1610), `behavior/std-libs/tempfile/temporary_directory_cleanup_on_exit.py` + `real_world/std-libs/errno/translate_oserror_errno_to_name.py` (1627, run repeatedly — intermittency).
- Rust-side proof: `src/mir/tests/{escape_analysis,escape_analysis_gate,typed_list_layout}.rs`; unit tests in `rc.rs`/`gc.rs` (cycle shapes, tracked-tuple contract #2128, re-entrancy guard).
- C2 perf pins are sensitive to this domain (threshold/hasher/SmallVec/parking_lot mitigations #2096/#2100/#2517/#2518) — memory changes need a perf-pin sweep, not just conformance.
