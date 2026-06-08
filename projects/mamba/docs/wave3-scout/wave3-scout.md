# Wave-3 typeshed scout — colorsys / fractions / uuid / functools

**Date:** 2026-05-15
**Author:** runner-stdlib (Team B, brute-force-then-standardize)
**Scope:** 4 unshipped stdlib libs surveyed from typeshed for wave-3 conformance
ships. Pure research — no source edits.

## TL;DR

| Lib       | True surface (consts+def+class) | Hot path                | Predicted regime           | Tier             | Blockers                                   |
|-----------|---------------------------------|-------------------------|----------------------------|------------------|--------------------------------------------|
| colorsys  | 3 + 6 + 0 = **9**               | scalar tuple-return     | balanced (compute-leaning) | compute          | tuple-return ABI (existing pattern OK)     |
| fractions | 0 + 0 + 1 = **1 class (~36 methods)** | OOP integer-handle, GCD-heavy | balanced (allocation + compute) | compute    | int-handle table; Decimal/Real coerce; carve `__pow__(complex)` |
| uuid      | 8 + 5 + 2 = **15**              | bulk-id generation, hex/int conversion | balanced (allocation-bound on bytes) | compute | NAMESPACE_* must be UUID instances at import; `getnode()` MAC lookup; subset A on `.bytes` (#2096) |
| functools | 2 + 3 + 0 = **5** (forward-only); 4 class deferred | reduce(fn, iter) — callback-bound | mostly startup-dominated, reduce blocked by #2100 | compute (carved) | reduce is #2100 callback-bound; lru_cache/partial/cached_property/cmp_to_key defer to wave-4 |

**Total estimated LOC:** ~720 across 4 libs (colorsys ~50, fractions ~350, uuid
~220, functools ~100 forward-only).
**Ship order recommendation:** colorsys → uuid → fractions → functools(carved).

## colorsys — pure-scalar tuple-return, trivial ship

**Source:** `projects/mamba/vendor/typeshed/stdlib/colorsys.pyi` (15 lines).

**Surface (9 entries):**
- **Constants (3):** `ONE_SIXTH`, `ONE_THIRD`, `TWO_THIRD` (all `Final[float]`).
- **Def (6):** `rgb_to_yiq`, `yiq_to_rgb`, `rgb_to_hls`, `hls_to_rgb`,
  `rgb_to_hsv`, `hsv_to_rgb`. All `(float, float, float) -> tuple[float, float, float]`.
- **Class (0):** none.

**Hot-path classification:** scalar 3-in-3-out. Pure math (no allocation beyond
the 3-tuple return). Hot-loop friendly — identical shape to math.scalar_sqrt PASS
template plus tuple-return.

**Predicted regime:** balanced.
- 100k iter of `rgb_to_hls(0.5, 0.25, 0.75)`: ~30-50 ms inner work in CPython
  (Python-level branches in `rgb_to_hls`).
- Mamba should compute-win because the branches lower to native ifs; tuple
  alloc is the only friction. Expect wall 8-15×, internal 2-5×.
- Memory: 3-tuple-per-call. Tuple is a hot-path concern for #2111-style
  iteration-retention if the user holds onto N tuples — but the bench harness
  drops them per-iter, so OK for bulk fixtures.

**Tier:** **compute** (pure float math, deterministic).

**Blockers:**
- **Tuple-return ABI.** `MbValue::from_ptr(MbObject::new_tuple3(r, g, b))` —
  pattern already used by `divmod` builtin and statistics.linear_regression
  (Task #41, ships fine). No new infra needed.
- **Constants surface.** Walker counts 3 `pub const ONE_SIXTH: f64 = ...;`-style
  module-level `attrs.insert("ONE_SIXTH".to_string(), MbValue::from_ptr(MbObject::new_float(...)));`
  single-line entries. Trivial — covers 3 of 9 surface points.
- **No OOP, no callback, no I/O, no Final-Const-walker drama.** Cleanest ship in
  the wave.

**Suggested fixture shape:**

```python
# benches/3p/colorsys/rgb_to_hls_bulk.py — tier: compute
import colorsys, sys, time
_rgb = colorsys.rgb_to_hls  # hoist for #2097
ITERS = 100_000
acc_h = acc_l = acc_s = 0.0
_t0 = time.perf_counter()
for i in range(ITERS):
    h, l, s = _rgb(0.25 + (i & 7) * 0.1, 0.5, 0.75 - (i & 3) * 0.05)
    acc_h += h
    acc_l += l
    acc_s += s
_t1 = time.perf_counter()
print("colorsys_hls:", int((acc_h + acc_l + acc_s) * 1000))
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

Second fixture: `hsv_round_trip.py` — round-trip `rgb→hsv→rgb` × 100k for
floating-point stability assert (within 1e-9). Third (informational):
`yiq_compute.py` for the simpler linear-coeff path.

## fractions — single OOP class, GCD-heavy compute

**Source:** `projects/mamba/vendor/typeshed/stdlib/fractions.pyi` (167 lines).

**Surface (1 class — `Fraction`, ~36 callable members):**
- **Constants (0):** none top-level (Fraction is the only export per `__all__`).
- **Def (0):** none top-level.
- **Class (1):** `Fraction(Rational)` — the workhorse, with constructors
  (`__new__`, `from_float`, `from_decimal`, 3.14+ `from_number`), conversions
  (`limit_denominator`, `as_integer_ratio`, 3.12+ `is_integer`), arith dunders
  (`__add__/__radd__/__sub__/__rsub__/__mul__/__rmul__/__truediv__/__rtruediv__/__floordiv__/__rfloordiv__/__mod__/__rmod__/__divmod__/__rdivmod__/__pow__/__rpow__`,
  16 total binop pairs), unary (`__pos__/__neg__/__abs__/__trunc__/__floor__/__ceil__/__round__`,
  7), comparison (`__eq__/__lt__/__gt__/__le__/__ge__/__bool__/__hash__`, 7),
  copy/serialize (`__copy__/__deepcopy__/__int__`, 3), and props
  (`numerator/denominator/real/imag/conjugate`, 5). Total ~36 distinct methods.

**Hot-path classification:** OOP-state with GCD-heavy compute kernel.
- **Constructor + arith** (the main win): `Fraction(3, 7) + Fraction(11, 13)`
  → reduce-by-gcd → return new Fraction. 100k iter of arithmetic.
  Integer-handle pattern fits: handle wraps `(num: i128, den: i128)`.
- **Comparison/hash:** delegate to numerator/denominator i128 ops. Cheap.
- **Property access** (`f.numerator`, `f.denominator`): one int read. Cheap.
- **`limit_denominator(n)`:** continued-fraction expansion. Compute-bound, no
  alloc beyond the return Fraction.
- **`from_float(f)`:** unpack IEEE-754 mantissa+exponent, build exact Fraction.
  Compute-bound.

**Predicted regime:** balanced (allocation + compute).
- Per-call: handle alloc + GCD reduction. GCD on i128 is ~5-30 ns native; the
  alloc dominates for small numerators.
- 100k iter of `(a + b) * c - d` Fraction arithmetic: ~200-400 ms inner CPython
  (4 ops × N reductions). Mamba native-Rust GCD beats CPython's Python-level
  Euclidean in `_gcd` — expect wall 10-20×, internal 3-8×.
- Memory: integer-handle table grows monotonically (per `[[project_mamba_integer_handle_pattern]]`
  Con #1). For 100k-iter bench: table will hold 100k+ handles. Acceptable
  per-iter dropped, but document the memory-leak note.

**Tier:** **compute** (deterministic, no I/O, no callbacks).

**Blockers:**
- **Integer-handle pattern + class.rs predicate branch** ([[project_mamba_integer_handle_pattern]]).
  ~36 methods × dispatch_method routing inside `fractions_mod.rs::dispatch_method`.
  Largest dispatch table in the wave — but mechanical.
- **i128 numerator/denominator:** CPython uses unbounded ints. Mamba currently
  has the 48-bit overflow at 2^47 ([[project_mamba_runtime_correctness_gaps_2026_05_13]]).
  Wave-3 ship can clamp to i128 with overflow-on-mul detection → fall back to a
  separate big-int path. Realistic: ship i128-only first, document the overflow
  carve-out for very-large-fraction workloads.
- **`from_decimal(d)` requires decimal interop** — decimal module ships
  (Task #15), but the cross-mod boundary needs a "Decimal → (mantissa, exponent)"
  read. Either expose a decimal mod helper or carve `from_decimal` out of v1
  (small surface cost).
- **`__pow__(complex)`** — complex returns require cmath shim (shipped Task #42
  according to handoff queue). Verify cmath_mod exposes a complex constructor;
  if not, carve `__pow__(complex)` out.
- **`Rational` ABC inheritance:** typeshed lists `class Fraction(Rational)`.
  Mamba doesn't have a numbers.Rational ABC. Skip the inheritance — Fraction
  stands alone as a class. May affect duck-typing tests; document.
- **Hash equality with int/float:** `Fraction(1, 2) == 0.5` must be True. Hash
  must match `hash(0.5)`. This is a known Python compat gotcha; CPython
  implements `Fraction.__hash__` to match `Decimal.__hash__` and `float.__hash__`
  via the modular-inverse trick. ~30 LOC of cleverness; document if mamba's
  test suite checks this cross-type hash compat.

**Suggested fixture shape:**

```python
# benches/3p/fractions/arith_bulk.py — tier: compute
import fractions, sys, time
F = fractions.Fraction  # hoist for #2097
a = F(355, 113)         # pi-ish
b = F(7, 22)            # 1/pi-ish
ITERS = 100_000
acc = F(0)
_t0 = time.perf_counter()
for i in range(ITERS):
    acc = (a + b * F(i & 31, (i & 7) + 1)) - F(1, 3)
_t1 = time.perf_counter()
print("frac_arith:", acc.numerator, "/", acc.denominator)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

Second fixture: `gcd_reduce.py` — 100k iters of `Fraction(2**31 - 1, 2**31 - 3)`
constructor to stress GCD on coprime large-ish ints. Third (informational):
`limit_denom.py` for `Fraction(math.pi).limit_denominator(1000)` × 10k to
exercise the continued-fraction kernel.

## uuid — module-level bulk-id, integer-handle class

**Source:** `projects/mamba/vendor/typeshed/stdlib/uuid.pyi` (106 lines).

**Surface (15 entries):**
- **Constants (8):** `NAMESPACE_DNS`, `NAMESPACE_URL`, `NAMESPACE_OID`,
  `NAMESPACE_X500` (all `Final[UUID]` — bound UUID instances at import time),
  `RESERVED_NCS`, `RFC_4122`, `RESERVED_MICROSOFT`, `RESERVED_FUTURE` (all
  `Final[LiteralString]`). Plus 3.14+ `NIL`, `MAX` (deferred, count = 8 today).
- **Def (5):** `getnode`, `uuid1`, `uuid3`, `uuid4`, `uuid5` (plus 3.12+ `main`,
  3.14+ `uuid6`/`uuid7`/`uuid8` — defer).
- **Class (2):** `UUID` (16-byte int + props), `SafeUUID` (Enum: `safe/unsafe/unknown`).

**Hot-path classification:** module-level bulk-id generator + OOP-state UUID
class with int-handle dispatch.
- **`uuid4()` × N:** `getrandom(16)` → 128-bit int → UUID handle. Per-call
  cost dominated by syscall (~1-5 µs each in CPython). 100k calls: ~100-500 ms
  CPython.
- **`UUID(int=x).hex` / `.bytes`:** integer-to-hex-string formatting (32-char
  lowercase hex with dashes) and integer-to-bytes16 packing. Bulk-bytes on
  `.bytes` → subset A (#2096) for 16-byte output.
- **`uuid5(NAMESPACE_DNS, "example.com")`:** SHA-1 of namespace+name (already
  shipped via hashlib int-handle — direct reuse).
- **Property reads** (`.int`, `.hex`, `.fields`): cheap once UUID is built.

**Predicted regime:** balanced (allocation-bound for `.bytes`/`.hex`).
- `uuid4()` × 100k: wall 3-8× (syscall-dominated; mamba can't beat the syscall
  itself, but can avoid CPython's wrapper overhead). Internal 1-3×.
- `UUID(int=x).hex` × 100k: ~30 ms CPython; mamba native fmt should be 5-15×
  internal. Wall 10-20×.
- `uuid5(ns, name)` × 100k: SHA-1 dominated. Same 8.56× hashlib win
  ([[project_mamba_integer_handle_pattern]] hashlib pilot) — reuses the
  hash dispatch path.

**Tier:** **compute** (`uuid4` has OS-entropy I/O but treat as compute-tier
because the syscall is the work; we're not benching wall-time RNG quality).

**Blockers:**
- **NAMESPACE_* constants must be UUID *instances* at import.** Walker counts
  them as `attrs.insert("NAMESPACE_DNS", ...)` single-line entries, but the
  value side must be `MbValue::from_ptr(MbObject::new_int(handle))` where the
  handle is pre-allocated in the UUIDS table at register() time. Pattern:
  module-level pre-init in `register()`, similar to how decimal_mod sets up
  context defaults. ~20 LOC.
- **`getnode()` MAC address lookup** is a Linux-specific syscall + `/sys/class/net/*`
  parse. Carve as a stub returning a stable pseudo-MAC for non-Linux/test
  contexts; document. Wave-3 bench doesn't hit `getnode` directly.
- **`uuid1()` requires `getnode()` + time-based v1 spec** — version+variant
  bit-packing. Mechanical but careful. Default arg `node=None` triggers the
  carve.
- **`SafeUUID` enum** — mamba enum support exists (per Task #15 / decimal). Wire
  as 3-constant class. Cheap.
- **`UUID(...)` constructor with 6 mutually-exclusive args** (`hex`, `bytes`,
  `bytes_le`, `fields`, `int`, `urn`-via-`hex`): CPython does runtime arg
  validation. Mamba shim needs the same exclusivity rule (raise `TypeError`
  if more than one is non-None). ~30 LOC of arg validation.
- **`__hash__`** must match `hash(uuid.int)` for set/dict membership. Trivial
  — delegate to the 128-bit int.
- **Subset A on `.bytes`** (#2096): 16-byte alloc per access. Acceptable;
  document the ~0.5-0.7× memory ratio.

**Suggested fixture shape:**

```python
# benches/3p/uuid/uuid4_bulk.py — tier: compute
import uuid, sys, time
_u4 = uuid.uuid4  # hoist for #2097
ITERS = 100_000
acc_xor = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc_xor ^= _u4().int & 0xFFFFFFFF
_t1 = time.perf_counter()
print("uuid4_xor:", acc_xor)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

Second fixture: `uuid5_ns.py` — `uuid5(NAMESPACE_DNS, f"name-{i}")` × 100k
(stresses SHA-1 reuse path). Third (informational): `uuid_hex_fmt.py` —
construct UUID from fixed int, read `.hex` × 100k for integer-to-hex bench.

## functools — reduce-only forward ship; lru_cache/partial deferred

**Source:** `projects/mamba/vendor/typeshed/stdlib/functools.pyi` (275 lines).

**Surface (proposed forward-only ship: 5 entries; FULL surface ~13 entries with
4 callable classes deferred):**
- **Constants (2):** `WRAPPER_ASSIGNMENTS` (Final[tuple[Literal, ...]]),
  `WRAPPER_UPDATES` (Final[tuple[Literal["__dict__"]]]). Trivial tuple constants.
- **Def (3 forward-only):** `reduce`, `total_ordering`, `cmp_to_key`. (Also
  `update_wrapper`, `wraps` — version-gated tuple defaults make these slightly
  awkward; can include if we hardcode 3.12 form.)
- **Class (0 in v1):** All 4 deferred — `_lru_cache_wrapper` (incl. `lru_cache`
  and `cache` factories), `partial`, `partialmethod`, `singledispatch` /
  `singledispatchmethod`, `cached_property`. All are callback-bound and/or
  state-heavy.

**Hot-path classification:** mostly callback-bound — this is the wave's
risk-pick.
- **`reduce(fn, iterable[, init])`:** callback-per-element. #2100 territory
  (every call invokes user-supplied `fn(acc, x)` from inside the reducer).
  Worst case for current dispatch overhead.
- **`total_ordering(cls)`:** class decorator. Computes 3-4 missing comparison
  methods from any 1 supplied. One-shot at class definition; not hot-path.
  Trivial ship.
- **`cmp_to_key(cmp)`:** returns a callable wrapper class. Each comparison
  during `sorted(..., key=cmp_to_key(my_cmp))` invokes the wrapper's
  `__lt__`, which invokes `my_cmp(a, b)`. Double-callback per compare —
  worse than itemgetter. Document carve.

**Predicted regime:** reduce is **mostly startup-dominated and likely SLOWER
under mamba** until #2100 lands. total_ordering / cmp_to_key are one-shot or
sort-callback-bound (#2100 again). This is a "ship the surface, fail the perf
gate cleanly, document #2100 carve" lib.

- `reduce(operator.add, range(100_000), 0)` 1 call: ~5 ms CPython. Mamba
  callback overhead → expect 2-5× **slower** wall. Internal 0.5-0.8× (mamba
  loses).
- `total_ordering`-decorated class with 100k compares: depends on whether the
  user-supplied comparator goes through mamba dispatch. Same shape as
  operator.itemgetter — defer per [[project_mamba_phase2_crosscutting_blockers]] #2100.

**Tier:** **compute** for `total_ordering`/`cmp_to_key` (no I/O); **carved**
for `reduce` until #2100 lands.

**Blockers:**
- **#2100 callback-bound — `reduce` is the canonical example.** Same blocker
  as `bisect.insort(key=)`, `heapq.nsmallest(key=)`, `sorted(key=itemgetter())`.
  Until #2100 (FFI-side cheap callback for runtime-supplied callables) lands,
  reduce's perf gate is **expected FAIL by design** — ship the surface to
  unblock import-compat, document the carve.
- **lru_cache / cache:** these need a dict-keyed cache that survives across
  calls (closure state). The wrapper class needs to be a callable mamba
  Instance. Wire as an integer-handle wrapping `HashMap<Vec<HashableArg>, MbValue>`.
  ~250 LOC + #2100 for the actual `fn(...)` invocation hop. **Defer to wave-4.**
- **partial:** state class wrapping `(func, args, kwargs)`. Integer-handle
  pattern. ~80 LOC + #2100 for the wrapped call. **Defer to wave-4.**
- **cached_property:** descriptor (`__get__` / `__set__`) — needs mamba
  descriptor protocol support. Unsure if shipped; **defer to wave-4** with a
  prerequisite-check task.
- **singledispatch:** type-dispatch table. ~200 LOC + type-introspection
  helpers. **Defer to wave-4.**
- **WRAPPER_ASSIGNMENTS / WRAPPER_UPDATES tuple constants:** version-gated
  in typeshed. Hardcode 3.12 form (6-element tuple); single-line walker entry.
  ~5 LOC each.

**Suggested fixture shape:**

```python
# benches/3p/functools/reduce_add.py — tier: compute (expected FAIL pending #2100)
import functools, operator, sys, time
_reduce = functools.reduce  # hoist for #2097
_add = operator.add
DATA = list(range(100_000))
ITERS = 10
acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = _reduce(_add, DATA, 0)
_t1 = time.perf_counter()
print("reduce_add:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

Second fixture: `total_ordering_compare.py` — 100k `<` compares on a
`@total_ordering`-decorated class; documents #2100 carve at the comparator
level. Third (informational): `cmp_to_key_sort.py` — sort 10k-element list
via `cmp_to_key(lambda a, b: a - b)` × 100 iters (callback-bound, expected
FAIL).

## Cross-lib wave-3 sequencing

Recommended ship order (lowest risk → highest):

1. **colorsys** (~50 LOC shim, 6 fns + 3 floats, tuple-return ABI only).
   Cleanest ship in the wave; expected clean PASS on all 3 gates. Reuses
   math/cmath float infrastructure.
2. **uuid** (~220 LOC, module-level + UUID int-handle, reuses hashlib SHA-1).
   Carve `getnode()` and 3.14+ versions. Expected ~13/15 = 86.7% Gate 3.
3. **fractions** (~350 LOC, single-class integer-handle, ~36 methods,
   GCD-heavy). Largest scope but most mechanical given the established
   hashlib/hmac/decimal handle template. Carve `from_decimal`, `__pow__(complex)`,
   and `Rational` ABC inheritance. Expected ~32/36 = 88.9% Gate 3 of the class
   members.
4. **functools** (~100 LOC, forward-only ship of `reduce` + `total_ordering` +
   `cmp_to_key` + 2 WRAPPER_* constants). reduce expected to **FAIL Gate 2
   perf** by #2100 design — document and ship. 5/13 surface coverage (38.5%);
   accept undershoot until #2100 + wave-4 callable classes.

**Total wave-3 estimate:** ~720 LOC across 4 libs, ~4-5 ticks at brute-force
cadence. fractions is the dominant cost.

## Pre-flight note re #2112 walker

Per team-lead dispatch: #2112 walker undercount fix runs in parallel (Team A).
All surface counts in this scout are **manually counted from typeshed `.pyi`**
(option (b) per dispatch), not from a `mamba surface-report --package` run.
Walker fix may reconcile to slightly different numbers post-ship — that's
fine. The numbers above are the conformance target; ship tasks should
re-verify with `surface-report` post-#2112.

## Cross-cutting findings

- **#2100 is still the big wall-blocker for callback-bound libs.** wave-2
  surfaced operator's 3 callable classes; wave-3 surfaces functools' 4 callable
  classes plus `reduce`/`cmp_to_key`. The wave-3 ship-now / wave-4 ship-after-#2100
  split for functools is the same pattern as operator forward-vs-callback.
- **Integer-handle pattern coverage is broadening:** fractions, uuid (UUID),
  functools (4 deferred classes), and (deferred to later waves) datetime /
  calendar / Calendar / TextCalendar all want the same template. Worth
  building the codegen target ([[project_mamba_rustlib_native_goal]] Phase 3
  `mamba-stdlib-module` section_type) sooner rather than later — current
  estimate is 5+ unshipped OOP libs all wanting the same boilerplate.
- **Decimal cross-mod coercion need:** fractions.`from_decimal` and (future)
  fractions⇄Decimal arith are the first wave to hit cross-stdlib coercion.
  May want a small "numeric-cross-mod-helper" infrastructure if 2-3 more libs
  bring this up (statistics-when-NormalDist-ships, math-when-Fraction-input-ships).
- **Wave-3 perf-tier story is mixed:** colorsys/fractions/uuid expected to
  hit Gate 2 PASS cleanly (compute-tier wins). functools.reduce is a known
  loss until #2100 — frame externally as "import-compat surface ship, perf
  pending callback FFI fix".

## Related

- `[[project_mamba_cross_runtime_startup_dominated]]` — four/five-regime table
- `[[feedback_mamba_perf_is_the_product]]` — perf-tier framing for carve decisions
- `[[project_mamba_phase2_crosscutting_blockers]]` — #2100 (callback FFI), #2096
  (bytes-materialization subset A), #2112 (walker undercount)
- `[[project_mamba_integer_handle_pattern]]` — fractions.Fraction / uuid.UUID
  / (wave-4) functools.partial all want this template
- `[[project_mamba_dispatch_prefix_convention]]` — `dispatch_` prefix for all
  wave-3 shims (codecs Task #34 worked example)
- `[[project_mamba_rustlib_native_goal]]` — Phase 3 codegen target for the
  growing OOP-state lib cluster
- `[[project_mamba_runtime_correctness_gaps_2026_05_13]]` — 48-bit int overflow
  at 2^47 is relevant to fractions i128-or-bigger numerator path
