# numbers — architecture (as-is, 2026-07-15)

Scope: the numeric tower — `int`/bigint, `float`, `complex`, `bool`, the
`+ - * / // % ** & | ^` dispatch, and the `decimal`/`fractions`/`numbers`/
`math`/`cmath`/`statistics` stdlib surfaces. The NaN-box value model itself is
`memory/ARCHITECTURE.md` (cross-ref, not restated); numeric hash/`==` key
unification is `object-model/identity-and-keys.md` (Domain 3); numeric
type-inference walls (`mixed_numeric_inference`, `*_float_inference`) are
`type-system/walls-and-widening.md`.

## Responsibilities

- Concrete value representation for the five numeric types and the inline↔heap boundary (48-bit int → `ObjData::BigInt`; `complex` heap pair).
- Overflow-checked integer arithmetic + BigInt fallback + JIT-callable ABI (`bigint_ops.rs`).
- The dynamic binop coercion tower (`builtins/mod.rs:mb_add`…`mb_pow`) — subclass unwrap → numeric handle → BigInt → int/float → complex.
- `Decimal`/`Fraction` as integer-**handle** value types + their dunder routing (`numeric_handles.rs`, `decimal_mod.rs`, `fractions_mod.rs`).
- Cross-type `==`/`hash`/ordering unification so `1 == 1.0 == True == 1+0j == Decimal(1) == Fraction(1)` and hashes agree (`hash.rs:mb_hash`).
- Thin math surfaces: `math`/`cmath` (f64-backed shims), `numbers` (ABC ranks), `statistics` (functions live; classes stubbed).

## Key structures & invariants

| Structure | Where | Invariant |
|---|---|---|
| inline int | `value.rs:from_int`/`as_int` (TAG_INT=1) | signed 48-bit `[-(2^47), 2^47-1]`; `from_int` debug-panics out of range; sign-extended on read. |
| `ObjData::BigInt(num_bigint::BigInt)` | `rc.rs:503` (ObjKind=11), `bigint_ops.rs` | heap, immutable, never GC-tracked; every op `normalize_bigint`s the result back to inline when it fits (`bigint_ops.rs:352`). |
| `INT48_MAX/MIN`, `fits_inline` | `bigint_ops.rs:21` | promotion boundary = ±2^47; the sole authority for "does this int stay inline". |
| float | `value.rs:from_float`/`is_float` | raw `f64` bits; ONLY `f64::NAN` and `NEG_CANON_NAN` (sign-carrying) survive the NaN-prefix as floats — all other prefixed NaNs canonicalize to +qNaN. |
| bool | `value.rs:from_bool` (TAG_BOOL=2) | box is DISTINCT from int; `as_int` is strict, `as_int_pyint` (`value.rs:213`, #1680) folds bool→0/1 for int-position use. |
| `ObjData::Complex(f64,f64)` | `rc.rs:503` (ObjKind=12) | heap `(re,im)`, immutable, not GC-tracked; coerced via `complex_helpers.rs:as_complex_pair`. |
| Decimal handle | `decimal_mod.rs:150` `DECIMAL_HANDLE_BASE=1<<46` | NaN-boxed int id ≥ 2^46; thread-local `HashMap<u64,MbDecimal>`; owns `[2^46, 2^47-1]` — LAST slot before NaN-box overflow at 2^47. |
| `MbDecimal{class,value,neg,exact}` | `decimal_mod.rs:91` | finite = `rust_decimal::Decimal` (96-bit mantissa, scale 0..=28); `DecClass::{Inf,QNan,SNan}` tracked separately (rust_decimal can't); `exact:Option<usize>` interns full BigInt coeff/scale beyond the window. |
| Fraction handle | `fractions_mod.rs:115` `FRACTION_HANDLE_BASE=1<<40` | id ≥ 2^40; `FractionState{num:i64,den:i64}` gcd-normalized, den>0 — **i64-only**, no BigInt components. |
| `HANDLE_MIN_ID=1<<40` | `integer_handle_registry.rs:32` | every integer-handle module MUST start ids here or above — `from_int(1)` is bit-identical to handle id 1. |

## Control flow

`mb_add`/`mb_sub`/`mb_mul` (`builtins/mod.rs:1461`+), `mb_truediv`/`mb_floordiv`/`mb_mod`/`mb_pow` (`floor_division.rs`, `power.rs`) share one order:
1. `int_enum_like_value` unwrap (IntEnum → its int) then `numeric_subclass_operands` unwrap (int/float subclass → payload, #1030), re-dispatch.
2. `numeric_handle_binop(op,a,b)` (`numeric_handles.rs:135`) — Decimal checked FIRST, then Fraction; `Some` short-circuits.
3. `bigint_numeric_binop` (`numeric_handles.rs:64`) — fires iff either operand `is_bigint_value`; BigInt⊕float widens both to f64.
4. int fast path: `(a.as_int_pyint(), b.as_int_pyint())` both `Some` → inline `wrapping_*`; else widen to f64 if either coerces.
5. type-specific arms (str/list/tuple/bytes concat for `+`/`*`), then complex arms via `is_complex_obj`/`as_complex_pair`; else TypeError.
6. **JIT-static path**: `Ty::Int` operands lower to `CheckedAdd/Sub/Mul` → extern `mb_bigint_{add,sub,mul}` (`bigint_ops.rs:456`+): raw-i64 fast path guarded by `NAN_PREFIX`; on the boxed slow path it re-checks `numeric_handle_binop` BEFORE `mb_int_add` (#2129/#961) so a Fraction/Decimal handle dispatches its dunder instead of adding ids.

## CPython-parity semantics

- **bool IS int**: `True + 1.0 == 2.0`, `True == 1`, `hash(True) == 1`, `isinstance(True,int)` — via `as_int_pyint`; but the *box* differs (identity ≠ equality — `object-model/identity-and-keys.md` Domain 3).
- **Coercion lattice**: int⊕int → int (BigInt-promoting, never wraps semantically); either float → float; either complex → complex (`floor`/`mod` on complex = TypeError). Decimal/Fraction do NOT silently mix with float — they coerce the *other* side, and Decimal+float raises `TypeError` in CPython (mamba routes Fraction⊕float→float per handle path; Decimal handled in-module).
- **Floor semantics**: `//` rounds toward −∞, `%` takes the DIVISOR's sign (`bigint_ops.rs:floor_div_mod:237`; f64 variant in `numeric_handles.rs`).
- **pow edges**: int**neg → float; `0**neg` → `ZeroDivisionError("0.0 cannot be raised to a negative power")` (`power.rs:6`); complex int-exp via repeated mul.
- **hash unification** (`hash.rs:mb_hash`): integral float folds to the int hash; `hash(-1) → -2`; `complex` = `float_hash(re)+1000003*float_hash(im)`; Decimal/Fraction integral → int hash, exact → float hash. Mixing these as dict keys must collide correctly.
- **int→float widening SATURATES to ±inf** (`bigint_ops.rs:int_as_f64:314`) — no `OverflowError`; a KNOWN divergence from CPython's `float(huge_int)`.
- **BigInt hash** = `value mod (2^61−1)`, sign-preserved (`bigint_ops.rs:395`).

## Known hazards

- **Handle id < `HANDLE_MIN_ID`** — `from_int(1)` == handle id 1; a low base lets primitive-int release corrupt handle tables (`integer_handle_registry.rs:19`).
- **Decimal base 2^46 is the last safe slot** — its range ends at 2^47−1, the NaN-box int ceiling; a wider/higher base overflows `from_int` (`decimal_mod.rs:147`).
- **Numeric-handle check must precede the int fast path** on the boxed slow path — else `Decimal('.1')+Decimal('.2')` adds raw handle ids and aborts (`numeric_handles.rs:4`, #2129).
- **`as_complex_pair` must exclude Decimal/Fraction handles** — else it reads the handle id as the real component (`complex_helpers.rs:9`).
- **Raw-i64 JIT fast path must re-check `fits_inline` before re-boxing** — a >48-bit result silently wraps otherwise (`bigint_ops.rs:465`, #1212 §5b).
- **Alias-boundary boxing**: a raw large i64 that aliases a NaN-box tag prefix (notably `−2^51`) must promote to BigInt, not be mis-read as already-boxed (`reg_to_mbvalue` / `passthrough_boxed_int_candidate`, #1136).
- **Decimal scale outside 0..=28** (e.g. `1E-100`, `1E100`) collapses toward zero / stays unrepresented — declared carve-out (`decimal_mod.rs:42`).
- **Fraction components beyond i64** are out of scope — such literals silently fail to parse (`fractions_mod.rs:436`).
- **`__main__` epilogue + BigInt inner-Vec drop** suspected double-free — sweep stays disabled (`memory/ARCHITECTURE.md` hazards, #1663).

## Extension points

- **New arithmetic op**: add an arm to `numeric_handle_binop` AND `bigint_numeric_binop` AND the `mb_*` dispatcher, in the step-2→5 order above; wire the JIT extern only if `Ty`-static lowering emits it.
- **New handle-pattern numeric type**: register `IntegerHandleHooks{retain,release}` (`integer_handle_registry.rs`), start ids ≥ `HANDLE_MIN_ID` in a non-overlapping band, add a range-guarded `is_*_handle`, and wire it into `numeric_handle_binop`, `hash.rs:mb_hash`, `mb_str`, and the `as_complex_pair` exclusion.
- **New complex op**: gate on `is_complex_obj`, coerce via `as_complex_pair`, mirror the CPython real-only-hash rule.
- **Widening the numeric tower** (new implicit coercion): coordinate with the checker — a widening that flips a `type/` numeric wall is wrong (`type-system/walls-and-widening.md`).
- **math/cmath fn**: add a `dispatch_*`-named wrapper (surface walker keys on the prefix) + tuple-table entry (`math_mod.rs`, `cmath_mod.rs`).

## Thin / stubbed (as-is)

- `numbers_mod.rs` (138 L): the five ABCs are rank tokens for `isinstance` (`NUMBERS_ABC_RANKS`, Integral=4 … Number=0); no actual ABC machinery/registration.
- `statistics_mod.rs`: module functions live; `NormalDist` is an Instance with `mu`/`sigma` fields but pdf/cdf/inv_cdf/operators NOT wired; `LinearRegression` is a namedtuple-shaped stub; `StatisticsError` is a type-name `Str`.
- `decimal` context model: only the fixture-backed subset of precision/flags/localcontext, not the full CPython context.

## EC surface

Per `external-contracts/README.md` (stdlib = `behavior|errors|real_world/std-libs/<mod>`):
- **Core types**: `tests/cpython/behavior/std-libs/{int, int_literal, float, numeric_tower, internals}`; `behavior/core/{builtin_numeric_inference, mixed_numeric_inference, *_float_inference}` (inference = type-system jurisdiction, listed for adjacency).
- **stdlib**: `behavior/std-libs/{decimal, fractions, math, math_property, cmath, statistics, numbers, number, abstract_numbers}`; `errors/std-libs/{decimal, fractions, math, cmath}`; `real_world/std-libs/{decimal, math, cmath, statistics}`.
- **Rust-side proof**: `bigint_ops.rs` unit tests — overflow→BigInt promotion, `normalize_bigint` demotion, `mb_int_hash`, and the alias-boundary ABI tests (`test_abi_{add,sub,mul}_promotes_alias_boundaries`, `−2^50`/`−2^51` slots).
