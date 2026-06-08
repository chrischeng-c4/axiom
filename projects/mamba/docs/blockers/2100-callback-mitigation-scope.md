# #2100 callback-bound — profile + mitigation scope

**Status**: scoping (Task #49, 2026-05-15, pilot-fixture-runner)
**Trigger**: functools.reduce(operator.add, list(range(10_000)), 0) shipped at 660× slower than CPython; perf carve-out documented in `functools_mod.rs`. Re-bench of colorsys after the #2128 conditional-tuple-track fix still ran at ~0.01× internal — same family, same diagnosis (below).
**DoD for this doc**: identify the dominant per-call cost centers, propose ranked mitigations with surface area + estimated speedup + LOC. **Do NOT implement here** — pick a target after team-lead review.

## Cross-fixture confirmation (added 2026-05-15)

Profiled **both** fixtures to isolate stdlib-shim dispatch from user-callback overhead:

- `reduce_add` (user-callback + alloc-heavy reduce hot loop) — `2100-reduce-profile.json`
- `rgb_to_hls_bulk` (pure stdlib-shim dispatch — no Python callback, `dispatch_ternary!` macro path only) — `2100-colorsys-profile.json`

| Bucket | colorsys (12 377 samples) | reduce (58 757 samples) |
|---|---|---|
| `SipHasher::write` (GC HashSet/HashMap) | **55.09%** | 20.6% |
| `gc::collect` | 24.76% | 30.2% |
| `BuildHasher::hash_one` (GC) | 6.29% | 8.6% |
| `HashMap::insert` (GC's `gc_refs`) | 5.41% | 6.9% |
| `gc::mark_object` | 2.68% | 27.6% |
| `RawTable::reserve_rehash` (GC) | 2.24% | 2.5% |
| **GC + hashing subtotal** | **~96.5%** | **~96.3%** |
| `dispatch_ternary!` / `mb_call_spread` | not in top-40 | 0.005% |
| `mb_add` / `mb_dispatch_binop` / `mb_box_float` | < 0.2% combined | < 0.02% |

**Same dominant cost in both fixtures, ~same %.** The `dispatch_ternary!` macro and the `mb_call_spread` path are *not* the bottleneck — neither in user-callback nor in stdlib-shim-only mode. The dominant cost is **GC bookkeeping triggered by per-iter tuple/list allocation**. The shim macros are functionally free; they merely expose the GC by allocating return tuples.

The colorsys case is especially striking — `rgb_to_hls` returns a 3-FP-element tuple containing only NaN-boxed floats. The conditional `value_is_cycle_capable` predicate correctly skips `gc_track` for that tuple (verified — `MbValue::from_float` returns None from `as_ptr()`). Yet GC still dominates at ~96.5%. Why? Because the tuple ALLOCATION itself bumps `gc.alloc_count`, which still triggers `collect()` every 700 allocs (`gc_track` gates tracking, not allocation counting). Each `collect()` then walks the entire `tracked: HashSet<usize>` and SipHashes every live container.

**Refined diagnosis**: the GC's `collect()` trigger is keyed on raw allocation count, not on cycle-capable-allocation count. Even though we now skip tracking for atomic-FP tuples, we still pay the full collect-sweep cost for them. Mitigation 1A (raise threshold) and 2A (FxHash) both fix this without touching `gc_track` semantics. A new Mitigation 1C is added below.

## TL;DR

The bottleneck is **NOT** in the FFI/dispatch ABI as #2100 originally framed it. It is in **per-iteration GC bookkeeping**:

| Bucket | Samples | % of self-time |
|---|---|---|
| `mamba::runtime::gc::collect` | 17 733 | **30.2%** |
| `mamba::runtime::gc::mark_object` | 16 215 | **27.6%** |
| `core::hash::sip::Hasher::write` (called from GC's `HashSet<usize>`) | 12 122 | **20.6%** |
| `core::hash::BuildHasher::hash_one` (GC) | 5 031 | **8.6%** |
| `hashbrown::HashMap::insert` (GC) | 4 025 | **6.9%** |
| `hashbrown::raw::RawTable::reserve_rehash` (GC) | 1 478 | **2.5%** |
| **GC + hashing subtotal** | **56 604** | **~96.3%** |
| `mb_call_spread` (the supposed dispatch hot path) | 3 | 0.005% |
| `dispatch_reduce` | 5 | 0.009% |
| `mb_add` (the actual reducer body) | 2 | 0.003% |

The reduce loop allocates `MbObject::new_list(vec![acc, item])` per call, which is **cycle-capable** and therefore must enter `gc::gc_track`. With the default threshold of 700 allocations, a 10 000-iter reduce triggers ~14 collect cycles per outer iter (×50 outers = ~700 GC passes), each one walking every tracked container and SipHashing every `usize` key.

The good news: **the dispatch ABI itself is essentially free** — `mb_call_spread` + `dispatch_reduce` together are 8 samples out of 58 757. We do not need to redesign the callback ABI; we need to fix the GC's per-call overhead.

## Methodology

- Build: `CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only cargo build --release -p mamba`
- Profile: `samply record --save-only --unstable-presymbolicate ./target/release/mamba run /tmp/reduce_profile.py`
- Fixture: `functools.reduce(operator.add, list(range(10_000)), 0)` × 50 outer iters
- Wall time: 62.097s (mamba) — implies ~111 µs per reduce step
- Samples: 58 757 across one thread
- Symbolication: `atos -o target/release/mamba -l 0x100000000` with batch input
- Profile artifact: `projects/mamba/docs/blockers/2100-reduce-profile.json` (1.1 MB)

## Top-3 cost centers (ranked by leverage)

### Center 1 — GC threshold too aggressive for callback hot loops (30% + 28% = 58% of total)

**Where**: `projects/mamba/src/runtime/gc.rs:39` — `threshold: 700`. Combined with the implicit GC trigger in `gc_track` (gc.rs:91 `gc.alloc_count >= gc.threshold`), every ~700th container allocation runs a full mark-and-sweep over all tracked containers.

**Why it hurts**: The reduce loop creates a tiny 2-element `List` per call. With 500 000 such lists, GC fires hundreds of times, and each pass walks an `O(tracked.len())` set. The `mark_object` recursion (gc.rs:347-351) and `collect` body (gc.rs:212) together dominate.

**Mitigation 1A — raise the threshold**

- Surface: gc.rs:39, change `700` → e.g. `10_000`. Optionally also dynamic: scale with live container count (`threshold = max(700, 2 * tracked.len())`) so the amortized work stays linear.
- Speedup estimate: **~5-10× on reduce** (fewer collect passes; the per-pass work shrinks proportionally because we let small short-lived lists die via refcount before GC sees them).
- LOC: ~3-5 lines. Risk: long-lived cyclic graphs sit longer before reclamation — but they are reclaimed at process exit anyway, and Python's own threshold default is `(700, 10, 10)` for generational GC.

**Mitigation 1B — generational GC**

- Surface: gc.rs ~750 LOC of new state + young/old set management.
- Speedup estimate: 10-30× on reduce, sustained across all callback-bound patterns. Aligned with CPython 3.12.
- LOC: ~300-500. Risk: significant refactor; touches the `tracked` field, `gc_track`/`gc_untrack`, `collect`, and probably `mb_release_value` (objects need a generation tag, requiring 1-2 bits in `MbObjectHeader`).

### Center 2 — SipHash on `usize` GC keys (~30% of total: 20.6% + 8.6% in hash itself)

**Where**: `gc.rs:19` — `tracked: HashSet<usize>` and `gc.rs:194` — `gc_refs: HashMap<usize, i64>`. Both use the standard library's `RandomState`, which is SipHash-1-3. SipHash is a cryptographic hash chosen for HashDoS resistance — irrelevant for a process-internal pointer set.

**Why it hurts**: Every `gc_track` insert and every `mark_object` visit hashes a 64-bit pointer through full SipHash. For a sweep over 5 000 tracked objects, this is 5 000 × ~40 ns = 200 µs of pure hashing, repeated per GC pass.

**Mitigation 2A — switch to FxHash for the GC sets**

- Surface: gc.rs:11 add `use rustc_hash::FxHashSet;` (or `IdentityHasher` if we want zero-cost since pointers are already random in their lower bits). Change `tracked: HashSet<usize>` → `tracked: FxHashSet<usize>` and `gc_refs: HashMap<usize, i64>` → `gc_refs: FxHashMap<usize, i64>`.
- Speedup estimate: **2-3× on the hashing-heavy ~30%** of total, i.e. ~10-15% off total wall.
- LOC: ~10. Risk: minimal — `rustc-hash` is already a transitive dependency through hashbrown/petgraph in the workspace (verify with `cargo tree -p mamba | grep fx`).

**Mitigation 2B — use a sorted `Vec<usize>` + binary search**

- Surface: replace `HashSet` with `Vec` + sort on collect. Avoids hashing entirely.
- Speedup estimate: 3-5× over SipHash; comparable to FxHash but with worse insert characteristics (sorted Vec inserts are O(n)).
- LOC: ~40. Not recommended — FxHash is simpler and just as fast for this access pattern.

### Center 3 — list-of-2 boxing in the reduce hot path (caller-side, not runtime)

**Where**: `projects/mamba/src/runtime/stdlib/functools_mod.rs:329-331`:

```rust
let pair = MbValue::from_ptr(MbObject::new_list(vec![acc, *item]));
acc = super::super::builtins::mb_call_spread(func, pair);
```

**Why it hurts**: `new_list` is cycle-capable (List can hold pointers to anything), so the conditional-track optimization from #2128 doesn't help. Every call pays: heap alloc + Vec build + gc_track insert + later release + gc_untrack.

**Mitigation 3A — direct binary-call path bypassing `mb_call_spread`**

- Surface: in `mb_functools_reduce`, when `func` is a native dispatcher (resolved via `resolve_callable_pub` and `is_native_func`), call it directly with a stack-allocated `[MbValue; 2]` instead of building a List. Skip the spread.
  ```rust
  if let Some(raw_addr) = super::super::builtins::resolve_callable_pub(func) {
      if super::super::module::is_native_func(raw_addr as u64) {
          let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue = unsafe { std::mem::transmute(raw_addr) };
          for item in &items[start..] {
              let args = [acc, *item];
              acc = unsafe { f(args.as_ptr(), 2) };
          }
          return acc;
      }
  }
  // fall through to the existing spread path for closures
  ```
- Speedup estimate: removes ~500 000 List allocs from the bench → eliminates most of the GC pressure that Center 1 / Center 2 expose. **Estimated 50-200× on reduce specifically** when func is a native fn (operator.add, operator.mul, math.* unaries, etc.).
- LOC: ~25. Risk: low — it's a fast-path; the slow path stays as-is for closure/lambda reducers.
- Caveat: this is reduce-specific, not a general FFI fix. But because reduce is the canonical callback-bound benchmark and many real-world reduce uses use operator.add/mul/max/min (all natives), this single fixture moves disproportionately.

**Mitigation 3B — generalize: stack-args fast path in `mb_call_spread` for small arities**

- Surface: in `mb_call_spread`, when the args_list is small (≤4) and func is a native, skip the list build entirely. Caller-side change: callers pass `args_ptr, nargs` instead of materializing a List.
- Speedup estimate: same magnitude as 3A but broader (helps map, filter, sorted-with-key, etc.).
- LOC: ~50-100 across `builtins.rs` and the caller sites. Risk: medium — touches the ABI between caller code and `mb_call_spread`.

## Ranked recommendation

| Rank | Mitigation | Est. speedup on reduce | LOC | Risk | Notes |
|---|---|---|---|---|---|
| **1** | **3A** (direct binary-call for native reducers) | **50-200×** | 25 | low | Single-file change; preserves closure fallback. Fixes the canonical bench without touching GC. |
| 2 | 2A (FxHash for GC sets) | 1.1-1.2× (broadly applicable) | 10 | minimal | Cross-cuts every workload, not just reduce. Cheap win. |
| 3 | 1A (raise GC threshold) | 1.5-3× on alloc-heavy loops | 5 | low | Complements 3A; without 3A this is the main lever on closure-reduce. |
| 4 | 1B (generational GC) | 10-30× sustained | 300-500 | medium | Phase 3 work; not for this scope. |
| 5 | 3B (general stack-args ABI fast path) | similar to 3A but broader | 50-100 | medium | If we want a one-fix-many-libs solution. |

**Recommended target**: **3A + 2A bundled** (~35 LOC total). Lands reduce close to or above CPython, gives every workload a 10-15% GC speedup, and leaves the harder generational-GC question for Phase 3. Threshold tuning (1A) is also cheap and can ride along.

**Cross-fixture revision**: The colorsys profile shifts the per-fixture lever rankings. For reduce, 3A is the single-biggest win (kills the per-iter List alloc). For colorsys, 3A doesn't apply (it returns a tuple, no per-iter caller-side alloc) — but 1A (raise threshold) and 2A (FxHash) both move the needle directly because they cut the GC sweep cost without touching the allocation pattern. So:

- **For #2100 reduce specifically**: 3A + 2A
- **For #2128 colorsys / general stdlib-shim perf**: 1A + 2A (3A doesn't help)
- **Bundle that helps both**: **1A + 2A** (~15 LOC, no source-level surgery, cross-cuts every workload that allocates *any* cycle-capable container in a hot loop)

Either bundle ships; pick based on which fixture you want to optimize first. **My pick: 1A + 2A first (helps both fixtures + every other workload), then 3A as a follow-up reduce-only ship.**

## Profile artifacts

Two samply profiles saved alongside this doc:

- `2100-reduce-profile.json` (1.1 MB) — functools.reduce(operator.add, list(range(10_000)), 0) × 50 outer
- `2100-colorsys-profile.json` (272 KB) — colorsys.rgb_to_hls × 100 000

Reproduce:

```bash
CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only \
  cargo build --release -p mamba --bin mamba
samply record --save-only --unstable-presymbolicate --no-open \
  --output projects/mamba/docs/blockers/2100-reduce-profile.json \
  ./target/release/mamba run projects/mamba/tests/cpython/fixtures/std-libs/functools/bench/reduce_add.py
samply record --save-only --unstable-presymbolicate --no-open \
  --output projects/mamba/docs/blockers/2100-colorsys-profile.json \
  ./target/release/mamba run projects/mamba/tests/cpython/fixtures/std-libs/colorsys/bench/rgb_to_hls_bulk.py
```

To symbolicate hex-address leaf frames (Firefox Profiler doesn't auto-symbolicate from Mach-O):

```bash
# 1. Extract leaf-frame addresses from the JSON via python3
# 2. Add 0x100000000 base offset
# 3. atos -o target/release/mamba -l 0x100000000 -f addrs_offset.txt
```

## Open questions for team-lead

1. **Pick a target**: 3A only? 3A+2A bundled? Or open all five as separate tasks?
2. **Threshold tuning** (1A) — comfortable raising to 10 000 unconditionally, or gate behind a `--gc-threshold` flag with autodetect?
3. The framing in #2100 was "callback ABI overhead." This profile shows that's wrong — it's GC overhead, exposed by the reduce *allocation pattern*. Reopen #2100 with the refined diagnosis vs. file a sibling issue?
