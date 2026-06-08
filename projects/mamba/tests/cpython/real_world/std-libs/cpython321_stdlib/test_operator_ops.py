# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_operator_ops"
# subject = "cpython321.test_operator_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_operator_ops.py"
# status = "filled"
# ///
"""cpython321.test_operator_ops: execute CPython 3.12 seed test_operator_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `operator` module functional surface.
# Surface: arithmetic (add/sub/mul/truediv/floordiv/mod/neg/abs),
# comparison (eq/ne/lt/le/gt/ge), boolean callables exposed as functions.
# Companion to stub/test_operator.py — vendored unittest seed.
import operator
_ledger: list[int] = []
# Arithmetic
assert operator.add(3, 4) == 7; _ledger.append(1)
assert operator.sub(10, 3) == 7; _ledger.append(1)
assert operator.mul(2, 5) == 10; _ledger.append(1)
assert operator.truediv(10, 4) == 2.5; _ledger.append(1)
assert operator.floordiv(10, 3) == 3; _ledger.append(1)
assert operator.mod(10, 3) == 1; _ledger.append(1)
assert operator.neg(3) == -3; _ledger.append(1)
assert operator.neg(-7) == 7; _ledger.append(1)
assert operator.abs(-7) == 7; _ledger.append(1)
assert operator.abs(5) == 5; _ledger.append(1)
# Comparison
assert operator.eq(5, 5); _ledger.append(1)
assert not operator.eq(5, 6); _ledger.append(1)
assert operator.ne(5, 6); _ledger.append(1)
assert not operator.ne(5, 5); _ledger.append(1)
assert operator.lt(2, 5); _ledger.append(1)
assert not operator.lt(5, 2); _ledger.append(1)
assert operator.le(2, 2); _ledger.append(1)
assert operator.le(2, 5); _ledger.append(1)
assert operator.gt(5, 2); _ledger.append(1)
assert not operator.gt(2, 5); _ledger.append(1)
assert operator.ge(5, 5); _ledger.append(1)
assert operator.ge(5, 2); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_operator_ops {sum(_ledger)} asserts")
