# Wave-2 Typeshed Surface Scout — statistics / random / cmath / operator

**Task #37** (2026-05-15) — pure research, no code. Scouts 4 libs newly unblocked by the
#2100 scope refinement (module-scalar fns NOT in callback territory; established by
math.scalar_sqrt 3.21× internal PASS in Task #36).

## TL;DR — per-lib summary

| Lib | Surface (constants + def + class) | Hot path | Predicted regime | Tier | Blockers |
|---|---|---|---|---|---|
| **cmath** | 7 + 24 + 0 = **31** | scalar-loop | startup-dominated (10-20× wall, 1-3× internal) | compute | none — clean mirror of math on complex |
| **statistics** | 0 + ~14 + 2 = **~16** | bulk-iterable + scalar | startup-dominated (5-15× wall) / compute-dominated for stdev/variance on 1M-element | compute | NormalDist OOP-state → integer-handle pattern needed for Gate 3 full surface |
| **random** | 0 + ~22 module-level + 2 = **~24** | scalar-loop (most), bulk for sample/shuffle/choices | startup-dominated (10-20× wall) for scalar; balanced for bulk | compute | Random / SystemRandom OOP-state (Mersenne Twister) → integer-handle pattern; `_inst` module-level singleton needs special init |
| **operator** | 0 + ~55 forward + 3 = **~58** | scalar forward wrappers (fast) | startup-dominated (10-15× wall) for forward; **#2100 BLOCKED** for itemgetter/attrgetter/methodcaller as `sorted(key=…)` callbacks | compute (partial) | itemgetter/attrgetter/methodcaller = callback-bound (PyO3 INTO mamba runtime); ship forward-only first wave, defer callable-classes |

## cmath — clean math-mirror on complex

**Source:** `projects/mamba/vendor/typeshed/stdlib/cmath.pyi` (37 lines).

**Surface (31 entries):**
- **Constants (7):** `e`, `pi`, `tau`, `inf`, `infj`, `nan`, `nanj`
- **Def (24):** `acos`, `acosh`, `asin`, `asinh`, `atan`, `atanh`, `cos`, `cosh`, `exp`, `isclose`, `isfinite`, `isinf`, `isnan`, `log`, `log10`, `phase`, `polar`, `rect`, `sin`, `sinh`, `sqrt`, `tan`, `tanh`, (one more in 3.13+: `isqrt` not present, so 24 total)
- **Class (0):** all functions are module-level.

**Hot-path classification:** scalar-loop — `cmath.sqrt(complex(i, i))` per iter. Mirrors
math.scalar_sqrt shape. No bulk-iterable surface (no `fsum`-equivalent for complex).

**Predicted regime:** **startup-dominated**. Per-iter cost is tiny (one complex sqrt
≈ ~50 ns). 100k iters × 50 ns = 5 ms inner work, dominated by 200 ms CPython startup.
Expect wall 10-20×, internal 1.0-3.0× (mamba should match or beat — same module-scalar
shape that math.sqrt cleared at 3.21×).

**Tier:** **compute** (numeric kernel, deterministic, no I/O).

**Blockers:** none. cmath surface is entirely module-scalar functions — fully unblocked
by #2100 scope refinement. RustCrypto-equivalent for complex math is `num-complex` or
hand-rolled (libstd's `f64::sqrt` + a thin Complex struct). No OOP state. No callback
paths.

**Suggested fixture shape:**

```python
# benches/3p/cmath/scalar_sqrt_complex.py — tier: compute
import cmath, sys, time
sqrt = cmath.sqrt
ITERS = 100_000
acc_re = 0.0
_t0 = time.perf_counter()
for i in range(1, ITERS + 1):
    acc_re += sqrt(complex(i, i)).real
_t1 = time.perf_counter()
print("cmath_scalar_sqrt:", int(acc_re))
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

## statistics — bulk-iterable + scalar mix, NormalDist OOP

**Source:** `projects/mamba/vendor/typeshed/stdlib/statistics.pyi` (162 lines).

**Surface (~16 entries top-level):**
- **Constants (0):** none.
- **Def (~14 module-level):** `fmean`, `geometric_mean`, `mean`, `harmonic_mean`,
  `median`, `median_low`, `median_high`, `median_grouped`, `mode`, `multimode`,
  `pstdev`, `pvariance`, `stdev`, `variance`, `quantiles`, `correlation`, `covariance`,
  `linear_regression` (~18 if counting all; 3.13 adds `kde`/`kde_random` — skip).
- **Class (2):** `StatisticsError` (trivial — ValueError subclass), `NormalDist`
  (heavy OOP with ~15 methods + properties + 8 dunder ops + classmethod from_samples),
  `LinearRegression` (NamedTuple — easy, just a 2-tuple).

**Hot-path classification:** mixed.
- **Bulk-iterable** (the win): `fmean`, `mean`, `stdev`, `variance`, `pstdev`,
  `pvariance`, `geometric_mean`, `harmonic_mean` — all take `Iterable[float]` and
  consume in one pass. 1M-element input is the natural hot loop. Same shape as
  math.fsum (which cleared compute-dominated at 1.02× internal).
- **Scalar** (`NormalDist.pdf(x)`, `NormalDist.cdf(x)`, etc.) — per-call float math
  on a small object, OOP-state.
- **Sort-based** (`median`, `median_low`, `median_high`, `quantiles`) — require
  `sorted()` of input. Mamba's sorted() perf is the bottleneck, not the wrapper.
- **Hashable-mode** (`mode`, `multimode`) — Counter-style; allocation-bound risk.

**Predicted regime:** mixed.
- `fmean` 1M elements: ~80-200 ms inner work (Welford or Kahan summation). Likely
  **balanced** wall 2-5×, internal 0.6-1.0×.
- `stdev` 1M elements: ~150-300 ms (two-pass Welford). Closer to **balanced** or
  edge of **compute-dominated**.
- `mean` (Fraction-exact path) 1M ints: ~500ms+ (CPython's Fraction kernel is slow).
  Likely **compute-dominated** at 1.0-1.3× — wrapper exonerated.
- `NormalDist.cdf` per-call: tiny scalar. **Startup-dominated**.

**Tier:** **compute** (numeric kernels, no I/O, deterministic).

**Blockers:**
- **NormalDist OOP-state** — needs integer-handle pattern per
  `[[project_mamba_integer_handle_pattern]]`. 8 dunder ops + samples (returns list)
  + classmethod constructor. Estimate ~250 LOC. Defer to wave-3 — surface counts
  NormalDist as 1 class but full Gate 3 needs all methods covered.
- **mean() preserves type** (returns Fraction for Fraction input, Decimal for
  Decimal). Pure-float fast path is easy; full type-preserving fmean is a runtime
  concern (Fraction support exists via `mb_fraction_*` but Decimal is partial).
- **mode() requires hashable handling** — uses Counter. Allocation-bound risk
  (each unique key → MbObject). For float input, hashable but few duplicates → fine.
- **3-tier ship plan:** wave-2 first ship = `fmean`, `mean` (float fast path),
  `variance`, `stdev`, `pvariance`, `pstdev`, `median`, `median_low`, `median_high`,
  `mode` → 10/16 = 62.5% Gate 3. Wave-3 picks up NormalDist + Fraction-exact paths.
  Note: 10/16 < 80% Gate 3 floor; either widen first ship to include `geometric_mean`,
  `harmonic_mean`, `quantiles`, `multimode`, `covariance`, `correlation`,
  `linear_regression` (numeric-only) → 17/18 = 94.4% wave-2-only ship, OR
  declare NormalDist a "section" carve-out.

**Suggested fixture shape:**

```python
# benches/3p/statistics/fmean_bulk.py — tier: compute
import statistics, sys, time
fmean = statistics.fmean
DATA = [i * 0.5 + i * 0.0001 for i in range(1_000_000)]  # JIT bit-op bug avoidance
ITERS = 50
acc = 0.0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc += fmean(DATA)
_t1 = time.perf_counter()
print("stat_fmean_bulk:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

Second fixture for `stdev` on the same DATA shape — expected closer to compute-dominated.

## random — Mersenne Twister + scalar, OOP-state heavy

**Source:** `projects/mamba/vendor/typeshed/stdlib/random.pyi` (135 lines).

**Surface (~24 entries top-level):**
- **Constants (0):** none top-level (`_inst` is private).
- **Def (module-level, via _inst delegation, ~22):** `seed`, `random`, `uniform`,
  `triangular`, `randint`, `choice`, `randrange`, `sample`, `shuffle`, `choices`,
  `normalvariate`, `lognormvariate`, `expovariate`, `vonmisesvariate`,
  `gammavariate`, `gauss`, `betavariate`, `paretovariate`, `weibullvariate`,
  `getstate`, `setstate`, `getrandbits`, `randbytes` (+ `binomialvariate` 3.12+).
- **Class (2):** `Random` (the workhorse, ~20 methods + Mersenne Twister state),
  `SystemRandom` (OS-rng subclass, ~5 methods, no state — uses /dev/urandom).

**Hot-path classification:** mixed.
- **Scalar-loop** (the main win): `random()`, `random()`, `randint(a, b)`,
  `uniform(a, b)`, `gauss()`, `expovariate()` — per-call returns single float/int.
  100k-iter hot loop. Same shape as math.scalar_sqrt PASS pattern.
- **Bulk allocation** (memory-pressure risk): `sample(pop, k)`, `choices(pop, k=N)`,
  `shuffle(list)`. Returns/mutates a list. Falls into #2096 subset A territory if
  k is large.
- **State-read-write** (rare hot path): `getstate()` / `setstate()` — returns
  624-int tuple (MT state). Per-iter is anti-pattern; pre-/post-loop is fine.

**Predicted regime:** mostly startup-dominated.
- `random()` 100k iter: ~10 ms inner work. wall 15-20×, internal 1-2×.
- `randint(0, 100)` 100k: ~20 ms inner. wall 10-15×, internal 1-2×.
- `gauss(0, 1)` 100k: ~80 ms inner (Box-Muller). wall 5-10×, internal 0.6-1.0×.
- `sample(range(1000), 100)` 1000 iter: per-call ~50 µs (Fisher-Yates partial),
  list-alloc of 100 ints. Predicted **balanced** with allocation pressure
  (subset A territory for very large k).

**Tier:** **compute** (deterministic given seed, no I/O for default rng).

**Blockers:**
- **Random class = Mersenne Twister 624-int state** — integer-handle pattern fits
  perfectly. Pilot reuses `rand` crate (`rand_mt::Mt19937GenRng32`) for parity with
  CPython's MT19937. Estimate ~300 LOC for full Random class (constructor + seed +
  all the variate methods).
- **`_inst` module-level singleton** — Python does `_inst = Random()` at import,
  then re-exports `_inst.random` as `random.random`. Mamba shim must bind a
  module-scoped Random instance handle at import time, then route the ~22 module-
  level functions to call `_inst.<method>(args...)`. Two clean approaches:
  (a) thread_local default handle, looked up on every call; (b) lazy-init on first
  call. (a) is simpler; (b) avoids unnecessary state if user only uses SystemRandom.
- **SystemRandom requires OS RNG access** — `getrandom()` syscall via `rand::OsRng`.
  Easy in Rust, but blocking concern: cross_runtime_3p harness runs in stable test
  context; OS entropy must be deterministic for repeatable bench. Defer SystemRandom
  to wave-3.
- **`bytes` return for randbytes** — subset A bulk-bytes-materialization for large n.
  Predicted memory ~0.5-0.7× per #2096; acceptable for ship.
- **`shuffle(list)` mutates in place** — needs to dispatch through mamba's list
  mutation path. Verify mamba list is mutable from FFI side.

**Suggested fixture shape:**

```python
# benches/3p/random/scalar_random.py — tier: compute
import random, sys, time
random.seed(42)
r = random.random
ITERS = 100_000
acc = 0.0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc += r()
_t1 = time.perf_counter()
print("rand_scalar:", int(acc * 1000))
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

Second fixture: `randint_bulk` on 100k iters; third (informational): `sample_bulk`
on (range(1000), k=100) × 1000 iters for allocation-bound check.

## operator — forward wrappers + 3 callback classes (split-ship recommended)

**Source:** `projects/mamba/vendor/typeshed/stdlib/operator.pyi` (218 lines).

**Surface (~58 entries top-level):**
- **Constants (0):** none.
- **Def (~55 forward wrappers, re-exported from `_operator` C module):** `abs`, `add`,
  `and_`, `concat`, `contains`, `countOf`, `delitem`, `eq`, `floordiv`, `ge`,
  `getitem`, `gt`, `iadd`, `iand`, `iconcat`, `ifloordiv`, `ilshift`, `imatmul`,
  `imod`, `imul`, `index`, `indexOf`, `inv`, `invert`, `ior`, `ipow`, `irshift`,
  `is_`, `is_not`, `isub`, `itruediv`, `ixor`, `le`, `length_hint`, `lshift`, `lt`,
  `matmul`, `mod`, `mul`, `ne`, `neg`, `not_`, `or_`, `pos`, `pow`, `rshift`,
  `setitem`, `sub`, `truediv`, `truth`, `xor` (+ `call` 3.11+, +
  `is_none`/`is_not_none` 3.14+, +24 dunder aliases like `__lt__`, `__abs__` — count
  these or not depending on Gate 3 walker policy).
- **Class (3):** `attrgetter`, `itemgetter`, `methodcaller` — callable classes used
  as `key=` arguments to `sorted()`, `min()`, `max()`, `groupby()`, etc.

**Hot-path classification:** binary split.
- **Forward-direction (50/58 unblocked):** `add(a, b)`, `eq(a, b)`, `lt(a, b)`, etc.
  Single dispatch INTO Rust → out. Module-scalar shape per #2100 refinement.
  Cleared by math.scalar_sqrt PASS template. Hot-loop friendly.
- **Callback-bound (3 classes, +potentially `call`):**
  `sorted(seq, key=itemgetter(0))` — sort calls `itemgetter.__call__(obj)` N times
  PER LOOP. PyO3 callback INTO mamba runtime is the #2100 blocked path. Same
  category as `bisect.insort(key=lambda x: x.k)` and `heapq.nsmallest(key=...)`.

**Predicted regime:**
- **Forward (`eq`, `add`, `lt` × 100k iters):** ~5-10 ms inner. wall 15-25×, internal
  1-3× (mamba should win — pure tight numeric, less Python-eval overhead than CPython
  bytecode interp).
- **itemgetter/attrgetter via sorted():** WORST case under #2100 — every key invocation
  is a cross-runtime dispatch. Expected **FAIL** until #2100 lands; defer to wave-3.

**Tier:** **compute** (forward fns; itemgetter/attrgetter pin to dynamic until #2100).

**Blockers:**
- **itemgetter, attrgetter, methodcaller = #2100 callback-bound.** These three classes
  are the canonical `sorted(..., key=...)` use case — every comparison call invokes
  `instance.__call__(obj)` from inside the sort algorithm. Currently FAIL territory.
  Document and ship without these three in wave-2; revisit when #2100 lands.
- **3.11+ `call` fn (one-shot dispatcher) and 3.14+ `is_none`/`is_not_none`:** version-
  gate; include if mamba targets 3.11+ (per the py312 conversion handoff, yes).
- **Forward-direction surface coverage:** 50 forward fns is straightforward
  `dispatch_binary!(dispatch_add, mb_op_add)` mapping to existing mamba binop runtime.
  Most can dispatch to the same functions class.rs already uses for `a + b` codegen,
  no new arithmetic needed. Estimated ~150 LOC for all 50 forward fns.
- **Dunder aliases (`__lt__ = lt`, etc.):** these are module-level re-exports. Gate 3
  walker counts them if it sees `__lt__ =` at column 0. Trivial — wave-2 ship covers
  these by re-export.

**Suggested fixture shape:**

```python
# benches/3p/operator/forward_eq.py — tier: compute
import operator, sys, time
eq = operator.eq
ITERS = 100_000
acc = 0
_t0 = time.perf_counter()
for i in range(ITERS):
    if eq(i & 7, 0):
        acc += 1
_t1 = time.perf_counter()
print("op_forward_eq:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

Second fixture (informational, expected FAIL): `sorted_itemgetter.py` — sorts a 10k-
element list of tuples by `itemgetter(1)` × 100 iters. Documents the #2100 callback
carve-out.

## Cross-lib wave-2 sequencing

Recommended ship order (lowest risk → highest):

1. **cmath** (~30 LOC shim, all module-scalar, no OOP, no blockers). Expected
   clean PASS like math.scalar_sqrt. Reuses math infrastructure (libstd f64 + a thin
   Complex struct).
2. **operator (forward-only)** (~150 LOC, 50 forward fns, defer 3 callable classes).
   Achieves ~50/58 = 86.2% Gate 3 surface with the carve-out documented.
3. **statistics (numeric-only)** (~400 LOC, drop NormalDist; first ship covers
   fmean/mean/median/stdev/variance/etc.). Expected ~17/18 = 94.4% Gate 3 sans
   NormalDist methods (count NormalDist as 1 class undeliverable carve-out).
4. **random (default-rng-only)** (~500 LOC, integer-handle Random + module-level
   delegation, defer SystemRandom). Largest scope but most reusable pattern —
   reuses the integer-handle template from hashlib/hmac.

Total wave-2 estimate: ~1080 LOC across 4 libs, ~3-4 ticks at brute-force cadence.

## Cross-cutting findings

- **#2100 scope refinement (locked in Task #36) is the wave-2 enabler.** Without it,
  scalar hot loops in cmath/random/operator would have been blocked. With it, all
  four libs have a clean perf-tier story for their scalar/forward surface.
- **OOP-state libs cluster:** statistics.NormalDist + random.Random + (future)
  threading.Lock / queue.Queue all need integer-handle pattern. Worth standardizing
  per-method codegen for OOP shims (already targeted in
  `[[project_mamba_rustlib_native_goal]]` Phase 3, section_type `mamba-stdlib-module`).
- **operator callback-bound classes are the next #2100 surface area:** confirms #2100
  is broader than just `bisect.insort(key=)` — itemgetter/attrgetter add 3 high-traffic
  callback patterns. Worth flagging when #2100 lands that operator's 3 callable
  classes should be re-tested first.
- **Subset A bulk-bytes risk:** random.randbytes(n) for large n falls into the
  bytes-materialization subset A. Pre-classify random ship to document the memory
  ratio rather than chase it.

## Related

- `[[project_mamba_cross_runtime_startup_dominated]]` — four-regime table
- `[[feedback_mamba_perf_is_the_product]]` — perf-tier framing
- `[[project_mamba_phase2_crosscutting_blockers]]` — #2096 / #2100 carve-outs
- `[[project_mamba_integer_handle_pattern]]` — NormalDist / Random shim shape
- `[[project_mamba_dispatch_prefix_convention]]` — `dispatch_` prefix for all shims
- `[[project_mamba_rustlib_native_goal]]` — Phase 3 codegen target for OOP-state libs
