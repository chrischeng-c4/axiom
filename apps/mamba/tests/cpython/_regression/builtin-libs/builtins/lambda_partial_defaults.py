# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Lambda partial defaults — `lambda a, b=10: a + b` evaluated at lambda
# creation time, filled in for any trailing args the caller omits.
#
# Root cause: `hir_to_mir.rs::HirExpr::Lambda` lowering only emitted defaults
# when EVERY param had a default (`all_have_defaults`). With a mix of required
# and defaulted params, defaults were dropped — the JIT body read uninitialized
# arg registers, returning garbage like `4617315517961601024`.
#
# Fix:
# 1. Lower defaults unconditionally (filter Some(...)) and tag the closure with
#    its total `arity` via the new `mb_closure_set_arity` extern.
# 2. `mb_call1_val` (1-arg dispatch) and `mb_call_spread` (2+) consult arity
#    when the call supplies fewer args than declared, and pad from the tail
#    of `defaults` (which are stored in trailing-binding order).

# Single trailing default — the original failure case.
print((lambda a, b=10: a + b)(5))                # 15
print((lambda a, b=10: a + b)(5, 20))            # 25

# Two trailing defaults; both filled when only 1 arg is supplied.
print((lambda a, b=10, c=20: a + b + c)(1))      # 31

# Two trailing defaults; only the last is filled when 2 args are supplied.
print((lambda a, b=10, c=20: a + b + c)(1, 2))   # 23

# String defaults (heap-allocated, exercise retention).
m = lambda a, b="hi": a + ":" + b
print(m("foo"))                                   # foo:hi
print(m("foo", "bar"))                            # foo:bar

# Default returned directly — verifies the closure stores the value, not just
# binds it to the param slot.
print((lambda a, b=99: b)(5))                    # 99
print((lambda a, b=99: b)(5, 7))                 # 7

# All-defaults still works (the original happy path).
print((lambda x=1, y=2: x + y)(10))              # 12
print((lambda x=1, y=2: x + y)(10, 20))          # 30
