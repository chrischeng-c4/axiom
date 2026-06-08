# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_operator_arith_compare_ops"
# subject = "cpython321.test_operator_arith_compare_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_operator_arith_compare_ops.py"
# status = "filled"
# ///
"""cpython321.test_operator_arith_compare_ops: execute CPython 3.12 seed test_operator_arith_compare_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for operator surfaces beyond
# test_operator_ops and test_operator_bitwise_seq_ops. Focus: every
# function-form operator across the four categories — arithmetic
# (binary add/sub/mul/truediv/floordiv/mod/pow plus unary neg/pos/abs);
# comparison (eq/ne/lt/le/gt/ge); logical (not_/truth/is_/is_not —
# yes operator.is_/is_not are exposed as functions for use in
# higher-order code); bitwise (and_/or_/xor/inv/lshift/rshift); and
# container membership (contains, which is the function form of `in`).
# Each call is verified against the equivalent operator-expression
# result so this fixture doubles as a sanity check that the named
# functions agree with the syntactic operators.
import operator
_ledger: list[int] = []

# Arithmetic — binary
assert operator.add(3, 4) == 7; _ledger.append(1)
assert operator.add(3, 4) == 3 + 4; _ledger.append(1)
assert operator.sub(10, 4) == 6; _ledger.append(1)
assert operator.sub(10, 4) == 10 - 4; _ledger.append(1)
assert operator.mul(3, 4) == 12; _ledger.append(1)
assert operator.mul(3, 4) == 3 * 4; _ledger.append(1)
assert operator.truediv(10, 4) == 2.5; _ledger.append(1)
assert operator.truediv(10, 4) == 10 / 4; _ledger.append(1)
assert operator.floordiv(10, 3) == 3; _ledger.append(1)
assert operator.floordiv(10, 3) == 10 // 3; _ledger.append(1)
assert operator.mod(10, 3) == 1; _ledger.append(1)
assert operator.mod(10, 3) == 10 % 3; _ledger.append(1)
assert operator.pow(2, 5) == 32; _ledger.append(1)
assert operator.pow(2, 5) == 2 ** 5; _ledger.append(1)

# Arithmetic — unary
assert operator.neg(5) == -5; _ledger.append(1)
assert operator.neg(5) == -5; _ledger.append(1)
assert operator.pos(5) == 5; _ledger.append(1)
assert operator.abs(-5) == 5; _ledger.append(1)
assert operator.abs(-5) == abs(-5); _ledger.append(1)

# Comparison — all six
assert operator.eq(2, 2) == True; _ledger.append(1)
assert operator.eq(2, 3) == False; _ledger.append(1)
assert operator.ne(2, 3) == True; _ledger.append(1)
assert operator.ne(2, 2) == False; _ledger.append(1)
assert operator.lt(2, 3) == True; _ledger.append(1)
assert operator.lt(3, 2) == False; _ledger.append(1)
assert operator.le(2, 2) == True; _ledger.append(1)
assert operator.le(3, 2) == False; _ledger.append(1)
assert operator.gt(3, 2) == True; _ledger.append(1)
assert operator.gt(2, 3) == False; _ledger.append(1)
assert operator.ge(3, 3) == True; _ledger.append(1)
assert operator.ge(2, 3) == False; _ledger.append(1)

# Logical
assert operator.not_(True) == False; _ledger.append(1)
assert operator.not_(False) == True; _ledger.append(1)
# truth — same as bool() coercion
assert operator.truth(1) == True; _ledger.append(1)
assert operator.truth(0) == False; _ledger.append(1)
assert operator.truth("") == False; _ledger.append(1)
assert operator.truth("x") == True; _ledger.append(1)
assert operator.truth([]) == False; _ledger.append(1)
assert operator.truth([1]) == True; _ledger.append(1)
# is_ / is_not — function form of the identity operator
assert operator.is_(None, None) == True; _ledger.append(1)
assert operator.is_not(None, 1) == True; _ledger.append(1)

# Bitwise
assert operator.and_(0xF0, 0x0F) == 0; _ledger.append(1)
assert operator.and_(0xFF, 0x0F) == 0x0F; _ledger.append(1)
assert operator.or_(0xF0, 0x0F) == 0xFF; _ledger.append(1)
assert operator.or_(0, 0) == 0; _ledger.append(1)
assert operator.xor(0xFF, 0x0F) == 0xF0; _ledger.append(1)
assert operator.xor(0xAA, 0xAA) == 0; _ledger.append(1)
# inv — two's-complement bitwise NOT (so ~0 = -1)
assert operator.inv(0) == -1; _ledger.append(1)
assert operator.inv(-1) == 0; _ledger.append(1)
assert operator.lshift(1, 4) == 16; _ledger.append(1)
assert operator.lshift(1, 4) == 1 << 4; _ledger.append(1)
assert operator.rshift(16, 2) == 4; _ledger.append(1)
assert operator.rshift(16, 2) == 16 >> 2; _ledger.append(1)

# Container membership — function form of `in` (on lists and tuples)
assert operator.contains([1, 2, 3], 2) == True; _ledger.append(1)
assert operator.contains([1, 2, 3], 99) == False; _ledger.append(1)
assert operator.contains((1, 2, 3), 2) == True; _ledger.append(1)
assert operator.contains((1, 2, 3), 99) == False; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_operator_arith_compare_ops {sum(_ledger)} asserts")
