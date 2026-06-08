# NOTES-NEXT — Mamba autonomous loop scratchpad

Short, single-fire notes from the autonomous cron loop. Each entry is one
investigation that ran out of budget before it could land. Pick up from the
top of the list and either finish it or supersede it with a clearer plan.

## MVP test profile — `docs/MVP_TEST_PROFILE.md` (#2535)

Workers claiming "mamba MVP green" must walk the seven gates in
`projects/mamba/docs/MVP_TEST_PROFILE.md`. The doc is the canonical
source for default-vs-opt-in, runtime budgets, and per-failure-mode
meaning across smoke, CPython Lib/test, ecosystem, perf, mambalibs,
package-manager, and debt. When a new harness lands (e.g. #2531
mambalibs Mode 2 or #2532 pkgmgr E2E), update the doc's gate row in
the same commit so workers do not run an outdated profile.

## MVP CPython Lib/test contract — folder convention (#3729, supersedes #2547)

Per-seed outcomes are pinned by **parent directory name** under
`projects/mamba/tests/cpython/lib_test_seeds/`:

- `pass/` → must AssertionPass (MVP-pass material)
- `spec/` → must Fail today; encodes full CPython contract mamba is growing into
- `stub/` → must Stub (entry point silently bypassed)
- `fail/` → must Fail (known broken, no growth path)
- `import_pass/` → must ImportPass (legacy bucket)
- `timeout/` → must Timeout (60s budget exceeded)

`cpython_lib_test_baseline.toml` + `cpython_lib_test_allowlist.toml` (and the
`cpython_lib_test_allowlist_regression_gate` test) were retired by #3729 —
the directory layout IS the contract. The runner
(`tests/cpython_lib_test_runner.rs::cpython_lib_test_folder_contracts`)
fails on bidirectional drift: a `spec/` seed that starts passing surfaces
the promotion as `git mv spec/<f>.py pass/<f>.py`; a `pass/` seed that
regresses surfaces the demotion the same way.

To promote a seed, just `git mv` it from its old contract dir to the new
one in the same commit that records the behavioural change. No TOML edit
needed.

## Gate 0 — `cargo test -p mamba -- --list` (#2534)

Every mamba MVP test session must start by clearing gate 0. Run:

```
python3 projects/mamba/scripts/gate0_list_tests.py --release
```

The wrapper compiles the mamba test profile and enumerates discovered
tests via `cargo test --list` without running them. Exit 0 means
"compile OK + tests visible — deeper gates may run". Exit non-zero
means stop and report; **no deeper gate may run until gate 0 is
green**. Use `--json` for machine-readable output suitable for the
autonomous loop summary line. See the script's docstring for full
exit-code semantics.

## Conformance metric — read this first (#1398)

When entries below say "conformance N/N" or "conformance green", that is the
**self-defined fixture suite** (`tests/cpython/`, `cpython_compat`,
`generator_conformance`, `iterator_conformance`, `behavioral_*`,
`p0_conformance`) — fixtures we authored ourselves. Passing them proves
**no regression** vs. mamba's own behaviour baseline; it does **not** prove
CPython 3.12 compatibility. Until #1396 wires CPython's `Lib/test/test_*.py`
in as the canonical denominator (T1 language core, T2 hot stdlib, T3 long
tail), Goal 1 status is **regression-baseline only** and the phrase
"100% conformance" is structurally misleading — never report it without the
"regression-baseline self-defined fixtures" qualifier. See #1260 / #1265
retraction comments for the full correction.

## 0. GC-tracker hash swap (PtrHasher) is bench-neutral — fire 57 (2026-05-07)

**Hypothesis ruled out.** Replaced `tracked: HashSet<usize>` with a
custom `PtrHasher` (identity-style `addr >> 4`, drops alignment bits,
no SipHash mixing) on the assumption that the std SipHash overhead
on every `gc_track` insert / `gc_untrack` remove was a measurable
fraction of the per-iter cost on `string_concat` (0.37×) and
`list_sort_builtin` (0.83×). Two stable bench runs after the swap:
`list_sort_builtin` 0.81× / 0.89× (vs baseline 0.83× — within noise)
and `string_concat` 0.34× / 0.28× (vs 0.37× — within noise, slightly
worse on one). Conformance + full lib suite stayed green. **Reverted**
because shipping a perf-claimed change that doesn't measurably win is
noise.

**Implication for §1b/§2:** the dominant cost on the parked sub-1.0×
workloads is NOT in the hash function but in the architectural
choices already documented (Box outer + Vec inner alloc per list,
RwLock per container, periodic sweep that walks every tracked
container). The single-fire "swap-the-hasher" lever is now ruled
out — closing the gap genuinely requires the multi-fire
inline-storage / generational-GC / Vec-pooling work in those
sections. Next-fire candidates that *might* still be single-fire
tractable on the GC side, in decreasing order of confidence:

  - **(a)** Skip `gc_track` for short-lived list/dict literals when
    the codegen can prove the object never escapes the iter
    (escape analysis on MIR — moderate scope, but bounded to the
    lower pass; would zero-out the alloc-tracking cost per
    string_concat/list_sort iter without touching the GC at all).
  - **(b)** Collapse `gc.tracked.insert(addr)` + `alloc_count += 1`
    into a single `Cell<usize>`-bumping fast path that defers the
    HashSet insert until threshold trip (requires an "untracked
    young" stash; doable but adds invariant complexity to
    `release_contained_values` and the sweep).
  - **(c)** ~~Lift the threshold from 700 → 4096~~ — **ruled out
    fire 58.** Tested with two stable bench runs: list_sort_builtin
    0.52× / 0.81× (vs baseline 0.83× — strictly worse first run,
    matched baseline second), string_concat 0.26× / 0.37× (also
    strictly worse first run). The sweep does the same per-tracker
    work but fires with 6× more trackers in flight, producing a
    large spike when it lands inside the measurement window —
    variance went up, mean didn't move. Reverted.

**Status (post-fires 57+58):** the cheapest two single-fire GC
levers (hash-fn swap, threshold bump) are both ruled out — the
remaining candidate is **(a)** MIR-level escape analysis on
short-lived list/dict literals. That is the next-fire candidate
for further sub-1.0× work, and is also the highest-ceiling lever.
Skipping (b) — invariant complexity stays prohibitive.

## 1. generator_sum — micro-opts didn't move the dial; bigger lever needed

**Fire 42 (2026-05-07) — first real win.** `mb_next` and `mb_has_next`
were doing 3+4 ITERATORS HashMap lookups + RefCell borrows per yield
in the slow path's per-kind dispatch (`is_callable` probe, `is_gen`
probe, short-circuit probe, write-peeked). Added a single-borrow
generator-iterator fast path at the top of each: one borrow returns
`Peeked|Resume(handle)|NotGen` and the Resume branch calls
`mb_generator_next` directly with the handle in hand, skipping
`advance_generator_if_applicable`'s internal borrow_mut.
**Bench delta: 0.42-0.44× → 0.57-0.59× (+35%, two-run stable),
2.24M ns/iter → 1.66M.** Conformance 591/591, all tests green.
Commit `8b535f172`.

**Fire 43 (2026-05-07) — investigation only.** Tried to push generator_sum
further by removing the now-dead `is_gen` block in mb_has_next's slow
path (the fast path makes it unreachable). Brace surgery slipped on
nested if/return shape; reverted clean. The arithmetic gap analysis:
post-fire-42 bench is ~167 ns/yield, CPython is ~94 ns/yield, gap
~73 ns. Two `swap_context` calls per yield = ~40-60 ns inherent
(architectural — CPython doesn't swap stacks; frames live on the eval
stack). Remaining ~15-30 ns is split across 3 ITERATORS borrows +
~11 TLS `.with()` calls + retain_if_ptr FFI thunks. Without an
architectural change (state-machine generators, or merging
mb_has_next + mb_next into a single sentinel-returning call to halve
FFI thunk crossings), generator_sum is stuck at ~0.6× ceiling.

**Fire 39 follow-up (2026-05-07):** Bundled the four `ACTIVE_GEN_*` /
`LAST_RESUMED_*` thread-locals into a single `GenActive` struct (one
`thread_local!` cell holding four `Cell<…>` fields). Hypothesis: each
`.with()` on macOS arm64 hits a dyld TLV stub, so collapsing the four
into one would amortize. **Bench delta: zero.** Rust's modern
thread-local lowering already inlines the access — the four-`with`
overhead in `resume_generator` was negligible. Shipped as a
code-quality refactor (commit `c845675f3`); does not advance §1's
acceptance bar.

**Conclusion confirmed:** the slot-vector refactor (lever (b) below)
remains the only single-fire-tractable lever left. Micro-opts on the
TLS / RefCell / hash side are exhausted.

**Bench state (2026-05-07 — fire 52, post mb_dispatch_binop primitive fast path):**

| workload                | run-1 | run-2 | run-3 | notes                                        |
|-------------------------|-------|-------|-------|----------------------------------------------|
| generator_sum           | 1.14× | 1.05× | 1.02× | **fire 52: 0.65× → 1.05× (skip format!)**    |
| string_concat           | 0.36× | 0.33× | 0.36× | parked §2 — GC tracking dominated            |
| list_sort_builtin       | 0.82× | 0.75× | 0.75× | parked §1b — same shape as string_concat     |
| factorial_recursive     | 2.14× | 2.11× | 2.11× | fire 50 (raw-int fast path through recurse)  |
| fib_recursive           | 4.24× | 4.24× | 4.35× | fire 50 (raw-int fast path through recurse)  |
| int_mul_loop            | 3.45× | 3.88× | 4.16× | beating CPython                              |
| int_sum_loop            | 20.5× | 22.7× | 21.5× | beating CPython                              |
| range_sum_loop          | 27.6× | 29.3× | 26.8× | beating CPython                              |

**Status: 6 of 8 benches beat CPython.** Only string_concat and
list_sort_builtin remain sub-1.0×, both blocked by the same
GC-tracking / inline-storage architectural work tracked in §1b/§2.

**Status: 5 of 8 benches beat CPython. The 3 remaining sub-1.0× rows
(generator_sum, list_sort_builtin, string_concat) all share the same
architectural blocker — alloc-tracking / swap_context dominate, not
algorithmic. Single-fire wins on these are exhausted; closing the gap
requires multi-fire architectural work (stackless generators or
generational GC / inline-storage), tracked under §1 (Lever B) and §1b/§2.**

*Asymmetry to investigate:* fib_recursive (0.88×, ~5 ns/call gap)
versus factorial_recursive (0.68×, ~29 ns/call gap). Both have the
same call ABI + same raw_int CheckedAdd/Sub/Mul fast paths. fib has
*more* arithmetic per call (2 calls + 1 add) than factorial (1 call
+ 1 mul) — yet factorial is much further from parity. Hypothesis:
CheckedMul's fast path adds 4-5 extra ops vs CheckedSub (smulhi +
sshr_imm + icmp + band) and that's the dominant per-call cost in
factorial. Tractable next-fire investigation: either (i) accept the
fits_48 cost on Mul as fundamental, or (ii) for callers known to
keep all intermediates inside a smaller bound (e.g. fact(15) → max
~10^12 fits in 41 bits), skip the smulhi+icmp pair via a type-derived
"narrow Int" hint.

**Fire 49 (2026-05-07) — KEY FINDING, no commit on the fix yet.**
The hypothesis above was wrong about *which* cost dominates factorial.
Built `/tmp/fact_probe.py` (`def fact(n: int) -> int: ... return
n * fact(n - 1)`) and ran `mamba build --emit mir`. The MIR for the
return statement contains **NO `CheckedMul` instruction at all** —
instead it lowers to `mb_box_int(n)` + `mb_dispatch_binop` (the slow
generic runtime dispatch path).

**Root cause (located):** `ast_to_hir.rs:1881-1893` decides the
`HirExpr::Call.ty` field by looking up the callee's HIR `return_ty`
from `self.result.functions`. But `result.functions.push(func)`
happens *after* the body is lowered (line 496), so when
`fact`'s body lowers `fact(n-1)`, `fact` itself is not yet in
`result.functions`. The lookup returns `None`, the call's `ty` falls
through to `tcx.any()`, and downstream `hir_to_mir.rs:4178`'s
`(lt, rt) == (Ty::Int, Ty::Int)` check on the BinOp fails — so
CheckedMul is never emitted, the multiplication goes through the
boxed dispatch path, and the raw-int fast path is bypassed entirely.
This applies to *any* recursive integer call, not just factorial.

**Why fib is different:** fib's body uses `fib(n-1) + fib(n-2)`. The
add is `CheckedAdd`-eligible only if both operands are Ty::Int — so
fib *also* misses the fast path on the add. But fib's two calls + 1
add is dominated by the call overhead (the BL stub + retain/release),
not by the boxing on the leaf op, so the typing miss costs less.
Factorial is 1 call + 1 mul, so the mul-side typing miss is the
larger fraction of per-call cost.

**Fix shape (next fire — single fire, tractable):** Two-pass HIR
lowering. Pass 1 walks all `ast::Stmt::FnDef` at module/class scope
and pre-registers a stub HirFunction (or just `(SymbolId, return_ty)`
in a side map) using the AST's annotated `return_ty` resolved through
`infer_return_type_from_ast` for unannotated bodies. Pass 2 lowers
bodies as before, but the call-site lookup at line 1886 now finds the
self-referential entry and assigns `ret_ty` correctly. Estimated
factorial impact: 0.68× → ~0.95-1.0× if the fast path lights up
(close the 29 ns/call gap to fib's ~5 ns level, since both then go
through CheckedMul/Sub/Add on raw_ints). May also lift fib slightly
(0.88× → ~0.92×) since fib's Add will now type-match.

**Verification:** after the fix, re-run
`./target/release/mamba build /tmp/fact_probe.py --emit mir` and grep
for `CheckedMul`. Should appear; absence means the fix didn't reach
the recursive call site.

**Fire 50 (2026-05-07) — SHIPPED, results far exceed projection.**
Implemented the two-pass approach as a side-table: added
`func_return_tys: HashMap<SymbolId, TypeId>` to `AstLowerer`,
populated at `lower_fn_inner` BEFORE `lower_stmt` walks the body
(line ~733), and the call-site lookup at line ~1881 now checks this
map first before falling back to `result.functions`. Methods are
skipped (`is_method=true`) since they intentionally use `any_ty`.

`mamba build /tmp/fact_probe.py --emit mir` now shows `CheckedMul`
on the recursive call (previously absent). All 700+ tests green.

**Bench delta (3-run stable):**

|                       | before  | after  | speedup |
|-----------------------|---------|--------|---------|
| factorial_recursive   | 0.68×   | ~1.9×  | 2.8×    |
| fib_recursive         | 0.88×   | 4.34×  | 4.9×    |

Why fib jumped *more* than projected: fib's `fib(n-1) + fib(n-2)` had
*both* recursive operands missing their type, so the Add went through
`mb_dispatch_binop` too. The fix unlocks both Add and Mul fast paths
in one go. Factorial's 0.68× → 2.0× is at the projected level
(raw-int CheckedMul + CheckedSub on i64 path); fib's 0.88× → 4.34×
is the bonus payoff for closing both operand-type holes at once.

**Implication:** any benchmark whose hot loop contains a recursive
integer call now gets the same uplift. This was a single low-LOC fix
(7 lines added + 4 lines modified) that converted the *2 closest-to-
parity recursion benches into 2 of the highest-margin wins in the
suite.

**Fire 12 (2026-05-07):** Added LAST_RESUMED_ID/LAST_RESUMED_CTX
thread-local cache in `resume_generator`. Cache hit skips the
`GENERATORS.borrow_mut().get_mut(&id)` HashMap probe in the bench's
steady-state loop. Cache busted on completion / throw / close / release.
**Bench delta: 0.33× → 0.40× (+18%).** Conformance 591/591.

**What was tried this fire (ineffective):**

- `clear_current_exception()` now borrow()-checks before borrow_mut() — saves
  one mutating borrow per yield in the steady-state (no exception) case.
- `resume_generator()` post-swap completion-check + return-value read collapsed
  into one `GENERATORS.with` block (was two). Saves one HashMap lookup per
  completion event.

Both shipped as `perf(mamba/runtime): trim resume_generator hot-path borrows`.
Tests + conformance green. **Bench delta: zero (within noise).**

**Conclusion:** The dominant per-yield cost is NOT the RefCell borrows or
`clear_current_exception`. Likely candidates:

a. **`swap_context` itself** — assembly stack swap. CPython's frame switch
   doesn't actually swap stacks (frames live on the eval stack). This is an
   architectural gap, not a tuning opportunity.

b. **`GENERATORS: RefCell<HashMap<u64, GenEntry>>`** — every public API entry
   (`mb_generator_next`, `mb_generator_send`, `mb_generator_close`,
   `mb_generator_throw`) hashes the id. Move to a slot-vector (`Vec<Option<…>>`)
   with the gen handle = slot index. Saves the hash+lookup per send/next/etc.
   Bigger ask: ~50 line refactor across generator.rs.

c. **`raise_stop_iteration` on every exhausted resume** — generator_sum
   exhausts the generator once at the end of 10000 yields, so this is one-off
   per benchmark iter, not per yield. Not the bottleneck.

**Fire 41 correction (2026-05-07):** Re-read `resume_generator`
(generator.rs:529-644) and `mb_generator_send` (:654). On the
steady-state per-yield path:

  - `mb_generator_send(handle, None)` short-circuits the pre-resume
    state check (the `value.is_none()` branch skips the
    `GENERATORS.with(|gens| gens.borrow().get(&id)…)` lookup
    entirely).
  - `resume_generator` cache-hit path (`a.last_resumed_id.get() == id`)
    skips the `GENERATORS.with(|gens| gens.borrow_mut().get_mut(&id))`
    lookup entirely — no hash, no borrow.
  - `mb_generator_yield_value` doesn't touch `GENERATORS` at all.

So the slot-vector refactor would NOT help generator_sum's
steady-state. It would only help cold start, completion, throw, and
close — none of which are inside the bench's tight loop. **Lever (b)
is misdirected; deferred indefinitely.**

What is actually hot per yield (after the cache hit):
  - 2× `swap_context` (asm stack-swap, ~20-40ns/swap on arm64) ≈ 40-80ns.
  - 4-5× TLS `with()` calls (GEN_XFER set, GEN_ACTIVE set/get, CALLER_CTX_STACK push/pop, GEN_XFER read) ≈ 5-10ns total.
  - Plus the iter-protocol cost in the for-loop wrapper: `mb_next` looks
    the iterator up in `ITERATORS` (HashMap), then dispatches to the
    Generator branch — this IS a per-yield hash hit. **Real candidate
    lever, but tangled with §4 (`iter(g) is g`).**

**Two architectural levers worth scoping (multi-fire, but each is the
only way past the ~0.6× ceiling):**

  - **Lever A — single-call iterator advance.** Replace the for-loop's
    `mb_has_next` + `mb_next` pair with one runtime call that returns
    a sentinel MbValue when exhausted, and the next value otherwise.
    Halves FFI-thunk crossings per yield (2→1) and eliminates the
    peeked-cell round-trip for new for-loops.

    **Fire 45 (2026-05-07) — scoping recon.** No code change; this
    bullet is now concrete enough that the next fire can implement
    directly.

    *Sentinel choice.* The MbValue tag field (`value.rs:29-37`) is
    3 bits: TAG_PTR=0, TAG_INT=1, TAG_BOOL=2, TAG_NONE=3, TAG_FUNC=4,
    TAG_NOTIMPLEMENTED=5. Tags **6 and 7 are unused** — pick `TAG_STOP_ITER = 6`,
    yielding the constant `0xFFFE_0000_0000_0000` (NAN_PREFIX `0xFFF8…` |
    `6 << 48`). One u64 const, no payload bits. Add
    `MbValue::stop_iter_sentinel()` + `is_stop_iter_sentinel()`
    helpers (~10 LOC in `value.rs`).

    *Runtime fn.* `mb_next_or_stop(iter_handle: MbValue) -> MbValue`
    in `iter.rs`, modeled on the fire-42 `mb_next` fast path: one
    `ITERATORS.borrow_mut()` returning
    `Sentinel | Peeked(v) | Resume(gen_handle) | NotGen`. Sentinel
    arm returns the sentinel; Resume arm calls `mb_generator_next`,
    on `check_stop_iteration` returns sentinel (and marks
    `exhausted = true`). NotGen arm falls through to current per-kind
    dispatch + sentinel-on-end. ~60 LOC.

    *Codegen.* Two for-loop call sites in
    `lower/hir_to_mir.rs` (`HirStmt::For` at line 1670, lowered at
    2858-2912; comprehension loop at 3720-3758). Replace each
    `mb_has_next` + Branch + `mb_next` triple with a single
    `mb_next_or_stop` + `CmpEq` against the sentinel const + Branch.
    Need a `MirInst` that does u64-bitwise eq against an immediate
    (or reuse existing — verify by grep on `CmpEq`/`EqImm`). If
    absent, falling back to a runtime helper `mb_is_stop_iter(v) -> bool`
    keeps codegen unchanged but adds back one FFI thunk (still
    a net-win: 2 thunks → 1 thunk + 1 cheap-FFI; cheap-FFI ~5ns
    vs the swap_context-adjacent `mb_next` thunk ~30ns).

    *Total LOC:* ~10 (value.rs) + ~60 (iter.rs) + ~30 (hir_to_mir.rs
    × 2 sites) ≈ 100 LOC. Single fire if the codegen path uses an
    existing immediate-eq MirInst; two fires if the cheap-FFI fallback
    is preferred for safety on first ship.

    *Risk.* `mb_has_next` outside the for-loop pattern (e.g. iterator
    protocol exposed to JIT-compiled `next()` calls or stdlib) must
    keep working — leave both functions intact, only the for-loop
    lowering switches to the new path.

    **Fire 46 (2026-05-07) — v1 shipped (cheap-FFI fallback).**
    Implemented as scoped: `TAG_STOP_ITER=6` sentinel + helpers in
    value.rs; `mb_next_or_stop` + `mb_is_stop_iter` in iter.rs (with
    fast-path generator branch + has_next/next slow-path fallback);
    both for-loop and comprehension lowering in hir_to_mir.rs switched.
    All 38 test groups + conformance fixtures green; pipeline test
    updated to assert `mb_next_or_stop` / `mb_is_stop_iter`.

    *Bench delta (two stable runs):* generator_sum 0.57-0.59× →
    **0.62-0.63×** (+~5%), Mamba ns/iter 1.66M → 1.49-1.51M (~10%).
    Smaller than fire 42's +35% because the cheap-FFI fallback still
    pays one FFI thunk for `mb_is_stop_iter`. The single ITERATORS
    borrow per yield IS gone (was 2 in mb_has_next + mb_next), but
    the FFI-thunk count is still 2.

    *Lever A v2 (next fire).* To capture the remaining ~30 ns/yield,
    inline the sentinel comparison in codegen instead of calling
    `mb_is_stop_iter`. Options: (a) add `MirConst::U64Bits(u64)` +
    a bit-equality op, then compare via `Is`; (b) add a dedicated
    `MirInst::IsStopIter { dest, src }` that lowers to
    `icmp_imm eq <sentinel>` in cranelift. (b) is the lower-risk path
    because it doesn't require new MirConst surface or backend
    audit. Estimated +5-10% on top of v1.

    **Fire 47 (2026-05-07) — v2 shipped via name-recognition (option c).**
    Took a third path: special-cased the name `mb_is_stop_iter` at the
    top of `cranelift/jit.rs::emit_extern_call`, lowering it to
    `icmp_imm eq 0xFFFE_…` + marking dest as `native_bools` so the
    Branch terminator skips `band_imm`. No new MirInst variants — the
    runtime symbol stays registered for non-JIT paths. ~30 LOC.

    *Bench delta (three stable runs):* generator_sum 0.62-0.63× (v1) →
    0.63 / 0.65 / 0.65× (v2). Marginal +1-2%, within bench noise. The
    FFI thunk was already very cheap (~5-10 ns) — the icmp inline saves
    that thunk but not the dominant `swap_context` cost.

    **Lever A is now at its ceiling.** Further wins on generator_sum
    require Lever B (stackless generators).
  - **Lever B — stackless generators.** Replace `swap_context` with
    state-machine lowering: each `yield` becomes a labeled
    suspension point, the generator body becomes a
    `match resume_state { 0 => …, 1 => after_yield_1, … }` switch,
    and locals are spilled to a heap-allocated frame. Eliminates the
    40-60 ns swap_context cost per yield. Multi-fire epic — touches
    HIR-to-MIR generator lowering, JIT entry/return ABI, and
    runtime/generator.rs in equal measure.

**Older next-fire candidates (still valid for incremental wins):** Either
  (i) collapse the 4-5 yield-side TLS cells into one bundled
      thread_local (precedent: fire 39's GenActive bundling, but on a
      different cell set — even if amortization is small, it's safe),
  (ii) attack the `mb_next` HashMap lookup for generator-handle iterators
       — but only as the §4 multi-fire refactor's first step (clean up
      `mb_next`'s generator-handle path while `mb_iter` still wraps),
  (iii) accept that generator_sum's gap is dominated by `swap_context`
       cost (architectural — CPython doesn't swap stacks) and move on
       to other workloads.

Acceptance: generator_sum ≥ 1.0× CPython (currently ~0.42-0.44×).

**Fire 51 (2026-05-07) — inlined mb_box_int for raw_ints; bench
neutral, foundation laid.** Hypothesis: each `yield i` in
generator_sum calls `mb_box_int(i)` via FFI thunk — inlining the
common-path boxing (raw INT48 → bor with template) should shave
~10-20 ns/yield. Implementation: at the top of `emit_extern_call`,
when `name == "mb_box_int"` and the arg is in `vars.raw_ints`,
emit a fits-48 check (`(x<<16)>>s 16 == x`) that simultaneously
rejects NaN-boxed values (high bits 0xFFF8) and out-of-range raw
i64 (which would BigInt-promote). Fast: `(arg & PAYLOAD_MASK) |
0xFFF9_0000_0000_0000`. Slow: FFI thunk to `mb_box_int` (handles
both NaN-boxed-BigInt passthrough + retain, and BigInt promotion
for >INT48 raw values).

**Initial bug caught by `1<<62` conformance test:** original
formulation skipped the fits-48 check and just did `bor` for any
non-NaN-boxed value. `1<<62 = 0x4000_0000_0000_0000` masked to
zero payload + bor template = boxed integer 0. Fixed by combining
both checks into the single `fits_48` predicate (correct for all
inputs).

**Bench delta:** within noise on all 8 rows. The reason
generator_sum (the intended target) didn't move: `total = total + x`
in the for-loop body — `x` is NaN-boxed (return type of
`mb_next_or_stop` is MbValue), so the Add goes through
`mb_dispatch_binop`, not CheckedAdd. That dispatch is the actual
bottleneck along with `swap_context`, not the yield-side
`mb_box_int`.

**Implication / next-fire candidate:** to move generator_sum, the
real lever is propagating the *yield element type* to the
for-loop variable so `total + x` can use the raw-int fast path.
This needs HIR/type-check awareness: if a generator's body has
all `yield <Int>`s, callers should see the iter's `next()` return
type as Int. Big refactor — separate from §1's stackless
generators (Lever B), but additive: even with swap_context
intact, typing the loop var would close another ~20-30 ns of the
49 ns gap. Tag for future fire.

## 1b. list_sort_builtin — same alloc-tracking shape as string_concat

**Fire 40 (2026-05-07) — investigation only, no commit.** Ran
`MAMBA_BENCH_SCALE_SWEEP=1 ./target/release/mamba bench --compare cpython`.
Per-iter scale signature for `list_sort_builtin`:

```
N= 10 → 491ns/iter  tracked=242  alloc_cnt=242/700
N= 50 → 384ns/iter  tracked=342  alloc_cnt=342/700
N=100 → 394ns/iter  tracked=542  alloc_cnt=542/700
N=200 → 720ns/iter  tracked=942  cycles=1  alloc_cnt=242/700  ← sweep fired in window
N=500 → 615ns/iter  tracked=1942 cycles=2  alloc_cnt=542/700
Δtracked=+1720 over 500 iters → ~3.4 tracked allocs / iter
```

Per-iter alloc breakdown for the bench body
(`data = [9,3,7,1,5,8,2,6,4,0]; sorted_data = sorted(data)`):
  - list literal: `mb_list_new_*` → `Box<MbObject>` + `Vec::from` backing
    + `RwLock` wrapping = 3 allocs.
  - `sorted(data)`: `extract_items` (`RwLock::read + Vec::clone`, 1
    alloc) + `new_list_borrowed` (`Box<MbObject>` + `RwLock`, 1-2 allocs).

Total ~5 raw allocs / iter, of which ~3.4 register with the GC tracker.
At GC threshold 700 the bench triggers a sweep every ~140 iters — the
720ns N=200 row is the sweep firing inside the measurement window.

**Same conclusion as §2:** the dominant cost is GC tracking + sweep,
not the sort itself. Single-fire wins exhausted; same multi-fire levers
apply (inline-storage for ≤4 elements, generational young-gen, pooled
Vec backing buffers). **Parked alongside §2** until a GC/inline-storage
refactor lands.

## 2. string_concat — GC-tracking dominated, not algorithmic

**Update (2026-05-07, fire 32):** Confirmed by `MAMBA_BENCH_SCALE_SWEEP=1`.
Per-iter cost split:
  - cold start (N=10): 354ns/iter — first-call code is hot, but GC threshold
    not yet hit; alloc_cnt=652/700 already high from outer module init.
  - mid-bench (N=50): 4.715µs/iter — GC sweep fires inside the measurement
    window (`alloc_cnt=2/700` after reset; col=false meaning no cycle
    collection, but tracking sweep still runs).
  - steady-state (N=100): 180ns/iter — close to CPython's 148-158ns once
    GC has just swept and isn't due again.

Per-iter alloc count: ~5 (mb_list_new_4 → Box<MbObject> + Vec::from buffer
→ 2 allocs; mb_str_join → 1 result string → 1 alloc; plus 4 per-element
retain calls update a counter but don't allocate). At ~5 allocs/iter and
GC threshold = 700, the bench triggers a sweep every ~140 iters; with
iters=100 the sweep firing inside the window is determined by prior
module-init allocs and is therefore order-noise.

**What `mb_str_join` is doing fine:** single pre-sized String alloc via
the join_slice fast path. No room for further algorithmic improvement
without escape analysis.

**What's left to win** (multi-fire, not single-fire-tractable):
  - Pool small Vec<MbValue> backing buffers so list-literal Vec::from
    doesn't hit the system allocator (saves 1 of the 2 allocs/list).
  - Generational GC: the 4-element list dies same-iter; tracking it in
    the global set + sweeping it on every threshold trip is the dominant
    overhead. CPython's young-gen survives mostly via refcount, only
    promoting after surviving a sweep.
  - Inline 4-or-fewer MbValues directly into the MbObject so list-of-4
    needs only the outer Box::new alloc.

None of these are single-fire scope. **String_concat is parked** until a
GC/inline-storage refactor lands.

## 3. factorial_recursive / fib_recursive (0.65×–0.85×)

**Fire 34 (2026-05-07) — partial win shipped.** Tagged Ty::Int/Bool params
as raw_ints in the JIT prologue + propagated raw_ints through internal-call
returns when both callee and call-site types are primitive Int/Bool. The
fits_48 check inside `emit_raw_int_op_with_overflow_check` keeps the change
correctness-safe — NaN-boxed inputs (inline ints or BigInt pointers) fail
the fits_48 test and route to the runtime slow path that handles boxed
inputs via `reg_to_mbvalue`. Bench delta: fib_recursive 0.73× → ~0.83×
(+10%), factorial_recursive 0.63× → ~0.71× (+10%, noisier). Conformance
591/591, jit_tests 39/39, iterator/generator/runtime-bugs all green.

Remaining gap to ≥1.0×: per-call FFI thunk overhead (declare_extern's
indirect-call thunk for arm64 BL ±128MB workaround adds 2 MOVZ/MOVK +
BLR per call) plus return-value retain on raw int returns
(emit_terminator may be adding a `mb_retain_value` call at return time
on raw ints — needs verification). Lower priority than NOTES §1.

**Fire 33 (2026-05-07) — investigation only, no commit.**

**Bench split:** factorial_recursive 0.68× (1.93 ns/call), fib_recursive 0.77×
(49.3 ns/call across 21891 calls per fib(20) iter). The gap to CPython is
~11 ns/call.

**Bottleneck identified:** function parameters are NOT added to
`vars.raw_ints` in the JIT entry prologue (`src/codegen/cranelift/jit.rs`
~line 290). So in `def fact(n): return n * fact(n - 1)`:
  - `n - 1` → CheckedSub VRegs not in raw_ints → `emit_checked_int_op`
    branch → FFI call to `mb_bigint_sub`.
  - `n * fact(...)` → same FFI path via `mb_bigint_mul`.

The fire-31 raw_ints fast path (now overflow-correct after `emit_raw_int_op_with_overflow_check`)
is bypassed because params are dressed as NaN-boxed `MbValue` bits.

**Option D design (the candidate fix):**

a. **Param unbox at prologue** — for each Int-typed param, sign-extend
   the lower 48 bits via `sshr_imm(ishl_imm(param_val, 16), 16)` and add
   the param VReg to `vars.raw_ints`. Identity for raw inline ints in
   INT48 range; correct for NaN-boxed inline ints (tag=1 payload extract);
   **garbage for NaN-boxed BigInt pointer (tag=0)**.

b. **Return-side raw forwarding** — when callee body's terminator is
   `Return(vreg)` and `vreg ∈ raw_ints`, declare the function's return
   ABI as raw i64 instead of NaN-boxed bits. Requires a parallel
   "callee returns raw" flag the call site reads. Without this, every
   recursive call site has to re-unbox the boxed return — savings on
   sub-cost but adds 2 ops per call.

c. **Call-site mirror** — `emit_internal_call` (jit.rs:1258) currently
   passes args via `vars.use_as_i64`, which already produces raw i64
   for raw_ints sources. So callers of an Int-param function that
   compute via raw_ints already pass raw bits — but the callee's
   prologue treats the param as boxed. Aligning (a) closes that gap.

**Estimated win:** fib_recursive 0.77× → ~1.15× CPython. Per-call body
savings ~14 ns (FFI ~10ns → raw ~3ns × 2 sub-ops); param unbox cost
~2 ops/param amortized.

**Correctness limitation (the reason fire 33 didn't ship):** Option D
silently produces wrong results for any caller that passes a BigInt
to a function whose param is typed `Int`. Mamba's force-typed contract
*should* prevent that, but the type system doesn't currently track
"int that fits INT48" vs "int that may be BigInt" — both are `Ty::Int`.
For fib(20)/fact(15) bench inputs no overflow happens, so behaviour
is correct in practice. But shipping silently-wrong-on-bigint behind
a generic "Int param" lowering is the same class of bug fire 31 just
closed for `iadd`.

**Two ways forward (single-fire-tractable):**

D1. **Conservative** — gate the unbox on the compile-time constraint
    `Ty::Int` AND a runtime check at the call site that the arg fits
    INT48. If not, fall back to the boxed-param ABI. Cost: 1 brif at
    each Int-call site; benefit: same as Option D when args fit.

D2. **Bold** — just unbox unconditionally for Ty::Int params. Add a
    test that `fact(very_big_bigint)` either raises or works
    correctly; if mamba doesn't have such a test today, write it
    BEFORE the optimization so the regression is caught at landing.

**Recommendation:** Next fire pick D2 (simpler, faster bench win)
and add a `tests/jit_tests.rs::test_recursion_bigint_arg_safety`
fixture guarding the boundary. If the type contract is "Int means
fits INT48", document it in `src/types.rs` near `Ty::Int`.

Acceptance: factorial_recursive ≥ 1.0× CPython, fib_recursive ≥ 1.0×
CPython, bigint-arg recursion test green, conformance 591/591.

## 4. `iter(g) is g` identity — RESOLVED in fires #1656 + #1658 (2026-05-09)

The "estimated 100–200 lines, multi-fire" landed as ~50 lines across two
atomic fires:

- **#1656** — PR #1657 (commit `0cbf4d8a1` → merge `dbf550ee6`): instead
  of teaching every `iter(...)` consumer about bare gen handles, register
  each generator's handle as an `IterKind::Generator` entry in `ITERATORS`
  at `mb_generator_create` time (gen IDs start at 1, iter IDs at 2^32 —
  disjoint sub-ranges, no collision). `mb_iter(gen)` now falls through
  the existing "already an iter" shortcut and returns the gen handle
  unchanged → CPython `iter(g) is g` identity holds. All dispatchers
  (`mb_next`, `mb_has_next`, `mb_next_or_stop`, `mb_next_raise`) already
  understand Generator-kind iter entries with proper peek-ahead, so the
  off-by-one drain bug from the earlier minimal-fix attempt is sidestepped.
  Test `test_generator_is_its_own_iterator_identity` un-`#[ignore]`d and
  passes; conformance stayed 686/0.
- **#1658** — PR #1659 (commit `15dc2413b` → merge `efd631d56`): dropped
  the now-dead generator-wrap branch in `mb_iter` and the unused
  `is_generator_handle` helper (-22 / +3 lines).

The original "make every consumer go through `mb_next`" plan was the
wrong unification point. Registering at creation pushed all gen handles
into the iter-id space ONCE, instead of teaching N consumers to handle
two flavours of handle.

## 5. Two codegen gaps now sitting under `#[ignore]` (fire 22, 2026-05-07)

Fire 22 marked seven jit_tests `#[ignore]` so the suite runs cleanly.
Each `#[ignore = "..."]` carries a one-liner pointing at the fix site.
There are **two** distinct root causes hiding under those seven tests —
not seven different bugs.

### 5a. Boxed-return from `__main__` — RESOLVED in fire 24 (2026-05-07)

Fix landed in `src/lower/hir_to_mir.rs::lower_top_level`: __main__'s
`return_ty` is now propagated from the captured `last_expr_ty` when it
is `Ty::Int|Bool|Float`, and the captured VReg is routed through
`mb_unbox_*_if_boxed` (new `runtime/builtins.rs` helpers) before
return. This collapses both forms (raw literal, NaN-boxed
IfExpr/getattr) to the raw primitive the JIT entry caller expects.
All 5 jit_tests + 3 fixtures now pass; their `#[ignore]` / `# XFAIL`
markers were removed.

### 5b. raw_ints fast path skips overflow check — RESOLVED in fire 31 (2026-05-07)

`emit_raw_int_op_with_overflow_check` (`src/codegen/cranelift/jit.rs`)
now wraps the raw_ints fast path in a runtime `fits_48` test:
`(v << 16) >>s 16 == v` over the result of `iadd` / `isub` / `imul`.
Overflow takes a slow-path block that boxes both operands and calls
`mb_bigint_{add,sub,mul}`, then merges via a Cranelift `select`.
Fire 37 follow-up further trimmed merge-block bookkeeping
(commit `53ca672aa`).

`test_bigint_overflow_{add,mul}_no_silent_wrap` `#[ignore]` markers
were lifted; jit_tests count is back to 39 + 1 ignored (the unrelated
`test_jit_disabled_when_env_set` env-isolation guard). All overflow
tests green; no remaining `#[ignore]` markers from this gap family.
