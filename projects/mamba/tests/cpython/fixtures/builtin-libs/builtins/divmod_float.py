# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `divmod(a, b)` was integer-only — the type-checker registered the
# builtin as `(int, int) → (int, int)`, so any float operand failed
# at type-check time:
#
#   divmod(17.5, 5)  # type error: argument type mismatch: expected int, got float
#
# Fix:
#   - Loosen the builtin signature to `(any, any) → (any, any)` in
#     `types/builtins.rs` (matches CPython's runtime-dispatched
#     numeric protocol; the runtime narrows by tag).
#   - Extend `mb_divmod` in `runtime/builtins.rs` with a float branch:
#     promote either operand to f64 when not int/int, compute floor
#     quotient + Python-sign remainder via `q.floor()` + `a - q*b`.
#     Mirrors CPython: int/int stays int, mixed promotes to float.

# Pure int — Python floor semantics (sign of remainder follows divisor).
print(divmod(17, 5))           # (3, 2)
print(divmod(-17, 5))          # (-4, 3)
print(divmod(17, -5))          # (-4, -3)
print(divmod(-17, -5))         # (3, -2)
print(divmod(0, 5))            # (0, 0)
print(divmod(5, 17))           # (0, 5)

# Float operands → both result components are floats.
print(divmod(17.5, 5))         # (3.0, 2.5)
print(divmod(17, 5.0))         # (3.0, 2.0)
print(divmod(17.5, 5.0))       # (3.0, 2.5)
print(divmod(-17.5, 5))        # (-4.0, 2.5)
print(divmod(17.5, -5))        # (-4.0, -2.5)

# Boundary cases.
print(divmod(0.0, 5))          # (0.0, 0.0)
print(divmod(2.5, 1.0))        # (2.0, 0.5)
print(divmod(1.5, 0.5))        # (3.0, 0.0)
