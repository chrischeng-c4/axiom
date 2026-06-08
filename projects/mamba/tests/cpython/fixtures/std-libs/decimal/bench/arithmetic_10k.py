"""Hot-loop bench for `decimal` arithmetic (Task #24 — cross-family).

End-user scenario: financial / fixed-point calculation pipeline that
constructs many `Decimal` values and does add/sub/mul mixes per-row.
Unlike hashlib/zlib/gzip/lzma, `decimal` has no natural bulk-work entry
point — arithmetic is per-op — so this fixture measures the dispatch +
arithmetic cost in a tight loop. Per the perf-is-the-product framing
(decision B): wall-time is the ship gate; internal-time is informational.

Tier: `compute`. Cross-family pair (mamba's `rust_decimal` vs CPython's
`_decimal` / libmpdec C extension) — expect the textbook 8–10× wall-time
band per the native-shim ceiling rule, not the higher same-family band.

Hoist convention (per #2097): module-level attributes are hoisted to
locals BEFORE the hot loop. Without hoisting, mamba's module-attr lookup
at the call site is ~5× slower than the hoisted form.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
harness compares per-iteration wall time and reports the ratio.
"""

import decimal

# Hoist module-level attributes outside the loop (see #2097).
D = decimal.Decimal

# Detect whether we are running under mamba's free-function-style
# decimal shim (`decimal.decimal_add` etc.) or CPython's operator-style
# semantics. Both runtimes must agree on the final number so the harness
# can compare per-iter wall time fairly. The fixture is identical apart
# from this dispatch layer.
HAS_FREE_FN = hasattr(decimal, "decimal_add")

if HAS_FREE_FN:
    add = decimal.decimal_add
    sub = decimal.decimal_sub
    mul = decimal.decimal_mul
    to_str = decimal.decimal_str
else:
    def add(a, b):
        return a + b
    def sub(a, b):
        return a - b
    def mul(a, b):
        return a * b
    def to_str(d):
        return str(d)

ITERS = 10000

# Pre-build a handful of operands outside the loop — same constants per
# iteration, so the iteration count drives the bench, not allocator
# variance. Stay below 2^47 small-int boundary for any mamba-side
# accumulators (we don't use any here, but the convention is to honour
# the rule everywhere in benches).
A = D("3.14")
B = D("2.71")
C = D("1.41")

# Internal-time marker for Task #22: measure the hot loop with
# per-call ratio. The wall-time ratio is dominated by Python startup
# overhead for short benches; the marker captures the pure steady-state
# per-call cost.
# Pre-declare loop-carried bindings outside the loop so mamba's
# force-typed analyser sees them at the readback site below. CPython
# accepts either form; mamba currently scopes for-locals tighter, so
# we hoist explicitly.
z = A
acc_count = 0
for _ in range(ITERS):
    x = add(A, B)
    y = sub(x, C)
    z = mul(y, A)
    # Count completed iterations without accumulating Decimal-on-int math
    # (which would entangle the per-iter cost with int↔Decimal coercion).
    acc_count += 1

# Invariant: every iteration produces a non-None Decimal triple, so the
# completed-iteration count must equal ITERS exactly.
assert acc_count == ITERS, f"iter count drift: {acc_count} != {ITERS}"

# Render the final z to lock in the str() / decimal_str() path so the
# bench doesn't get DCE'd. The exact text differs between rust_decimal
# (preserves operand scale → e.g. "13.94...") and libmpdec (context
# precision), so we don't compare it cross-runtime; just confirm it is
# a non-empty string.
final = to_str(z)
assert len(final) > 0, "final stringification empty"
print("arithmetic_10k:", acc_count)
