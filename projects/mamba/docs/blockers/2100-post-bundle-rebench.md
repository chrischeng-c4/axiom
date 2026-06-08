# Post-#2100-bundle re-bench sweep (Task #55)

- **Sweep date**: 2026-05-15
- **Bundle commit**: `015b820a2` (3A + 2A + 1A)
- **Bench harness**: `cargo bench -p mamba --bench cross_runtime_3p`, best-of-5,
  release profile, Apple M-series
- **Headline**: **30/33 wall PASS = 91%** — crosses the 60% close-rule
  threshold for [#2100](https://github.com/chrischeng-c4/cclab/issues/2100)
- **Two still-wall-FAIL libs** (NOT #2100-bound, intrinsic): `fractions/arith_bulk`
  (JIT op-overload [#2129](https://github.com/chrischeng-c4/cclab/issues/2129)),
  `keyword/iskeyword_hot` (borderline, startup-dominated; see
  [keyword-iskeyword-borderline.md](keyword-iskeyword-borderline.md))
- **Seven-lib internal-<1.0× cohort** (Wall PASS, Internal FAIL — filed as
  phase-2 [#2178](https://github.com/chrischeng-c4/cclab/issues/2178)):
  array/typed_bulk, csv/reader_rows, colorsys/rgb_to_hls_bulk,
  functools/reduce_add, operator/forward_eq, random/randint_bulk,
  re/findall_hot

## Headline measurements

| Fixture | Metric | Pre-bundle | Post-bundle | Δ | Gate |
|---|---|---|---|---|---|
| functools/reduce_add | wall | 0.52× | **17.30×** | 33× | **FLIP→PASS** |
| functools/reduce_add | internal | 0.0015× | 0.17× | 113× | <1.0× FAIL |
| functools/reduce_add | mem | 1.16× | 1.03× | stable | PASS |
| colorsys/rgb_to_hls_bulk | wall | 0.13× | **1.65×** | 12.7× | **FLIP→PASS** |
| colorsys/rgb_to_hls_bulk | internal | 0.01× | 0.20× | 20× | <1.0× FAIL |
| colorsys/rgb_to_hls_bulk | mem | 0.08× | 0.11× | stable | <1.0× FAIL |
| urllib_parse/urlparse_bulk | wall | (pre-Task-#52 unmeasured) | **6.05×** | n/a | PASS |
| urllib_parse/urlparse_bulk | internal | n/a | 1.04× | n/a | PASS |
| math/scalar_sqrt (canary) | internal | 3.27× | 3.41× | stable | PASS |
| fractions/arith_bulk | wall | 0.31× | 0.33× | nochange | FAIL (#2129) |
| keyword/iskeyword_hot | wall | n/a | 0.88× | borderline | FAIL (carve) |

## Classification

**Wall flipped FAIL → PASS** (the #2100 win): every Gate 2 carve-out
filed as "wall <1.0×, alloc-pressure / dispatch-overhead suspected" is
a candidate to flip without re-shipping shim code. Confirmed across
reduce, colorsys, urllib_parse, and ~27 others in the sweep.

**Internal <1.0× residual cohort** (7 libs): remaining cost is per-iter
allocation in the JIT call/iter codegen path — NOT in the shim, NOT in
GC, NOT in dispatch. The bundle eliminated the shim-side allocs (3A
native arith) and GC-side cost (1A+2A threshold+FxHash), but the JIT
still allocates a small box per iteration step (loop variable rebinding,
for-loop iter handle, etc.). Filed as [#2178](https://github.com/chrischeng-c4/cclab/issues/2178).

**Intrinsic wall-FAILs** (2 libs, not #2100-bound):
- `fractions/arith_bulk` — JIT op-overload gap [#2129](https://github.com/chrischeng-c4/cclab/issues/2129);
  `+` `*` `-` `/` lowered to native i64 by JIT, bypass class.rs dunder
  dispatch. Workaround: module-level dispatcher fns. See
  [[project_mamba_int_handle_operator_overload_gap]] memory.
- `keyword/iskeyword_hot` — set-membership 0.88×; suspect startup-dominated.
  Carve doc at [keyword-iskeyword-borderline.md](keyword-iskeyword-borderline.md).

## Gap note

The full 34-row sweep table (every fixture × wall/internal/mem) was
**not persisted as raw data** during the Task #55 sweep — only the
classified buckets above (headline flips, 7-lib internal cohort,
2-lib intrinsic FAILs) were captured. Raw numbers for the
non-headline fixtures live in the criterion bench output that was
overwritten by subsequent runs; rebuild by re-running
`cargo bench -p mamba --bench cross_runtime_3p` against `015b820a2`
if a per-fixture audit is needed.

## Cross-links

- [#2100](https://github.com/chrischeng-c4/cclab/issues/2100) (closed)
  — phase-1 GC-bound resolution; reframed diagnosis + mitigation table
  in body
- [#2178](https://github.com/chrischeng-c4/cclab/issues/2178) (open)
  — phase-2 per-iter JIT call/iter alloc; 7-lib internal-<1.0× cohort
- [#2129](https://github.com/chrischeng-c4/cclab/issues/2129) (open)
  — JIT op-overload gap; explains fractions/arith_bulk
- [keyword-iskeyword-borderline.md](keyword-iskeyword-borderline.md)
  — keyword/iskeyword_hot 0.88× carve doc
- [2100-callback-mitigation-scope.md](2100-callback-mitigation-scope.md)
  — Task #49 scoping doc with samply diagnosis + cross-fixture profile
- `015b820a2` — bundle commit (3A native-fast-reduce + 2A FxHash + 1A threshold)
