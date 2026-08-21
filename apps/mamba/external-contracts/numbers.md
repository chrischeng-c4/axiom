# numbers — external contract (as-is, 2026-07-15)

Domain map: `tech-design/numbers/ARCHITECTURE.md` (EC surface section). Verdict law: `HARNESS.md`.
Oracle = live python3.12 byte-diff; xfail = acknowledged gap, still contract (skip, never executed).
Scope: int/bigint, float, complex, bool, the binop coercion tower, Decimal/Fraction handles,
`math`/`cmath`/`numbers`/`statistics`. Numeric type-inference walls live under type-system
(`type-system/walls-and-widening.md`); numeric hash/`==` key unification is object-model Domain 3.

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

Core numeric types (live-counted 2026-07-15, `tests/cpython/` relative):

| Dir | .py | xfail | Covers |
|---|---|---|---|
| `behavior/std-libs/int/` | 19 | 19 | pylong str↔int conversion, digit-limit DoS guards, int-subclass return semantics |
| `behavior/std-libs/int_literal/` | 6 | 3 | hex/oct/bin literal parsing (signed + unsigned baselines) |
| `behavior/std-libs/float/` | 27 | 27 | CAPI float pack/unpack/check/getinfo, general float parsing (test_capi/test_float port) |
| `behavior/std-libs/numeric_tower/` | 9 | 8 | cross-type `==`/hash unification — int/float/bool/complex/Decimal/Fraction interplay, the domain's headline CPython-parity claim |
| `behavior/std-libs/internals/` * | 6 | 6 | **not numeric** — all 6 source from `Lib/test/test_ctypes/test_internals.py`; see Known contract gaps |

stdlib module surfaces:

| Dir | .py | xfail | Covers |
|---|---|---|---|
| `behavior/std-libs/decimal/` | 336 | 296 | `Decimal` class + `Context` (precision/rounding/flags) — largest single set in the domain |
| `behavior/std-libs/fractions/` | 68 | 42 | `Fraction` class arithmetic/parsing |
| `behavior/std-libs/math/` | 106 | 69 | `math` module function surface |
| `behavior/std-libs/math_property/` | 2 | 2 | `nextafter` property checks (commutativity, count) |
| `behavior/std-libs/cmath/` | 49 | 22 | `cmath` module function surface |
| `behavior/std-libs/statistics/` | 262 | 171 | `statistics` functions; `NormalDist`/`LinearRegression`/`StatisticsError` stub surfaces (see gaps) |
| `behavior/std-libs/numbers/` * | 21 | 16 | mixed — 5 genuine numbers-ABC/Fraction fixtures (`test_abc.py`/`test_fractions.py`, 0 xfail) + 16 misattributed ctypes fixtures; see Known contract gaps |
| `behavior/std-libs/number/` | 11 | 11 | C-API numeric protocol (`test_capi/test_number.py`: binary/unary ops, index, tobase, rshift) |
| `behavior/std-libs/abstract_numbers/` | 7 | 7 | numbers-ABC registration semantics (`test_abstract_numbers.py` port) |

Errors + real-world:

| Dir | .py | xfail | Dir | .py | xfail |
|---|---|---|---|---|---|
| `errors/std-libs/decimal/` | 15 | 0 | `real_world/std-libs/decimal/` | 1 | 0 |
| `errors/std-libs/fractions/` | 6 | 0 | `real_world/std-libs/math/` | 1 | 0 |
| `errors/std-libs/math/` | 16 | 0 | `real_world/std-libs/cmath/` | 1 | 0 |
| `errors/std-libs/cmath/` | 7 | 0 | `real_world/std-libs/statistics/` | 1 | 0 |

Totals (core types + stdlib + errors + real_world): **977 fixtures, 699 xfail** (278 live).
`*` dirs are counted at face value above per the live-count rule; see Known contract gaps for what
they actually test.

Adjacent, type-system jurisdiction (listed for adjacency per ARCHITECTURE.md, not summed into the
totals above — a divergence found here is a type-system bug, not a numbers bug):

| Dir | .py | xfail |
|---|---|---|
| `behavior/core/builtin_numeric_inference/` | 15 | 0 |
| `behavior/core/mixed_numeric_inference/` | 12 | 0 |
| `behavior/core/comprehension_float_inference/` | 16 | 0 |
| `behavior/core/float_return_inference/` | 18 | 0 |
| `behavior/core/generator_float_inference/` | 15 | 0 |

Rust-side proof — `bigint_ops.rs` in-file unit tests (20 total, all live, no xfail concept at this layer):

| Proof | Test names |
|---|---|
| Overflow → BigInt promotion, `normalize_bigint` demotion | `test_add_overflow_promotes_to_bigint`, `test_sub_overflow`, `test_mul_overflow`, `test_mul_no_overflow`, `test_add_no_overflow`, `test_normalize_back_to_inline` |
| Inline/BigInt hash + cmp + eq parity | `test_hash_inline`, `test_hash_bigint`, `test_cmp_inline`, `test_eq_inline`, `test_bigint_add_bigint` |
| JIT-callable ABI + alias-boundary promotion (#1136) | `test_abi_{add,sub,mul,cmp,eq}`, `test_abi_{add,sub,mul}_promotes_alias_boundaries` |
| Inline range boundary | `test_fits_inline_boundary` |

## Negative contract — what must be REJECTED

This domain DOES own `type/` wall dirs — one per stdlib module it owns (live-counted 2026-07-15):

| Dir | .py | xfail | Notes |
|---|---|---|---|
| `type/std-libs/decimal/` | 124 | 0 | typeshed `_Decimal`/`Context` arg-type walls |
| `type/std-libs/math/` | 61 | 0 | typeshed `math` arg-type walls |
| `type/std-libs/fractions/` | 54 | 0 | typeshed `Fraction` arg-type walls |
| `type/std-libs/cmath/` | 23 | 0 | typeshed `cmath` arg-type walls |
| `type/std-libs/math_integer/` | 4 | 0 | int-only `math` entrypoints (isqrt/comb/perm-shaped) wrong-type walls |
| `type/std-libs/numbers/` | 4 | 0 | numbers-ABC arg-type walls |
| `type/std-libs/_decimal/` | 3 | 0 | C-accelerated `_decimal` module arg-type walls |

Total: **273 walls, 0 xfail** — fully live and enforced, unlike the positive contract's heavy xfail
rot above. Weakening any of these (or adding an xfail) is a contract breach per README.md.

`int`/`float`/`complex`/`bool` own **no dedicated `type/` dir**: their wrong-type call surface rides
inside the shared `type/builtin-libs/builtins/` bucket (449 files total; ~112 match a
`bool__|int__|float__|complex__|abs__|round__|pow__|divmod__|hex__|oct__|bin__` prefix) alongside
str/list/dict/set/etc. walls for every other builtin type. That directory is not numbers-owned by
ARCHITECTURE.md's per-module naming convention — it is type-system's shared builtins surface (same
treatment as `exceptions.md`'s `asyncio_exceptions`/`xml_sax__exceptions` note).

## Known contract gaps

- **Un-xfail campaign stratum dominates this domain**: `int/`, `float/`, `number/`, `abstract_numbers/`
  are 100% xfail (0 of 63 fixtures across those four dirs actually execute); `decimal/` 88% (296/336),
  `statistics/` 65% (171/262), `math/` 65% (69/106), `fractions/` 62% (42/68), `cmath/` 45% (22/49) —
  all carry the same `auto-ported`/`auto-extracted CPython test; mamba promotion pending` marker seen
  in object-model/exceptions. Measured campaign stale rates elsewhere (18.3%/3.2%) imply hidden PASSes
  here too, at larger scale than either sibling domain. tracked: #1768.
- **`internals/` and 16/21 of `numbers/` are ctypes fixtures, not numeric-tower fixtures**: every file
  in `behavior/std-libs/internals/` sources from `Lib/test/test_ctypes/test_internals.py`, and 16 of
  the 21 files in `behavior/std-libs/numbers/` source from `Lib/test/test_ctypes/test_numbers.py` —
  both are CPython ctypes-suite files that collide by filename stem with this domain's own module
  names, so the auto-port tool bucketed them here. Only `numbers/`'s other 5 fixtures
  (`test_abc.py`/`test_fractions.py`-sourced, 0 xfail) are genuinely numbers-ABC content. 22 fixtures
  counted in this domain's positive-contract total above test ctypes object internals, not int/float/
  numbers behavior. No existing tracker names this specific misattribution.
- **Decimal scale carve-out**: exponents outside `rust_decimal`'s `0..=28` window (e.g. `1E-100`,
  `1E100`) collapse toward zero or go unrepresented instead of CPython's arbitrary-precision
  context-driven scale (`decimal_mod.rs:42`, clamp at `decimal_mod.rs:480`).
- **Fraction i64-only carve-out**: numerator/denominator components beyond `i64` silently fail to
  parse rather than promoting to arbitrary precision like CPython's `Fraction`
  (`fractions_mod.rs:436-438`, the surrounding handle shim is i64-only by design).
- **int→float widening saturates instead of raising**: `int_as_f64` (`bigint_ops.rs:314`) returns
  `±inf` for a `BigInt` too large for `f64`; CPython's `float(huge_int)` raises `OverflowError`. No
  existing tracker names this divergence specifically (only the file:line citation in ARCHITECTURE.md).
- **Thin stdlib surfaces** (as-is, not defects-in-progress but real functional gaps): `numbers_mod.rs`'s
  five ABCs are `isinstance` rank tokens (`NUMBERS_ABC_RANKS`) with no actual ABC registration/virtual-
  subclass machinery; `statistics_mod.rs`'s `NormalDist` has `mu`/`sigma` fields but no
  pdf/cdf/inv_cdf/operators, `LinearRegression` is a namedtuple-shaped stub, `StatisticsError` is a
  type-name-only `Str`; the `decimal` context model covers only the fixture-backed precision/flags/
  localcontext subset of CPython's full `Context` API.
- **`__main__` epilogue + BigInt inner-Vec drop suspected double-free** — sweep stays disabled for
  affected fixtures; cross-domain hazard shared with `memory/ARCHITECTURE.md`. #1663 does not cover this — that
  issue is closed and entirely about an unrelated pgpool perf issue (the same wrong citation also appears in
  `tech-design/numbers/ARCHITECTURE.md:65` and `tech-design/memory/ARCHITECTURE.md:36,47`); no correct issue
  was found by keyword search, so treat this as untracked until one exists.

## Verification

```bash
# focused inner loop (seconds; runner-parity verdicts, shared oracle cache; set MAMBA_BIN after a
# release build) — from apps/mamba/
python3 tests/harness/cpython/tools/sweep.py tests/cpython/behavior/std-libs/{int,int_literal,float,numeric_tower,internals}
python3 tests/harness/cpython/tools/sweep.py tests/cpython/behavior/std-libs/{decimal,fractions,math,math_property,cmath,statistics,numbers,number,abstract_numbers}
python3 tests/harness/cpython/tools/sweep.py tests/cpython/errors/std-libs/{decimal,fractions,math,cmath} tests/cpython/real_world/std-libs/{decimal,math,cmath,statistics}
python3 tests/harness/cpython/tools/sweep.py tests/cpython/type/std-libs/{decimal,fractions,math,math_integer,cmath,numbers,_decimal}   # negative contract — must stay 0 xfail

# cargo gate slice (datatest filter is a path substring; one dir per filter run)
cargo test -p mamba --release --test conformance -- std-libs/decimal
cargo test -p mamba --release --test conformance -- std-libs/int

# Rust-side unit proof (20 tests: overflow promotion, normalize demotion, hash/cmp parity, JIT ABI)
cargo test -p mamba --lib bigint_ops

# C2 slice
cargo test -p mamba --release --test perf_pin -- perf_pin   # tests/harness/cpython/config/perf/pins/int_mul_loop_2514.toml

# manifests: config/manifests/std-libs/{cmath,decimal,fractions,math,numbers,statistics}.toml exist;
# int/int_literal/float/numeric_tower/internals/math_property/number/abstract_numbers are
# hand-authored/pre-manifest — after fixture edits in manifest-backed dirs run tools/fixture_lint.py

# full C1 gate (the only progress signal; ~3 min; never concurrent with a cargo build)
cargo test -p mamba --release --test conformance
```
