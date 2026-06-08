# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_operator_arith_extras_ops"
# subject = "cpython321.test_operator_arith_extras_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_operator_arith_extras_ops.py"
# status = "filled"
# ///
"""cpython321.test_operator_arith_extras_ops: execute CPython 3.12 seed test_operator_arith_extras_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `operator` arithmetic surface
# not covered by `test_operator_ops`, `test_operator_arith_compare_ops`,
# `test_operator_bitwise_seq_ops`, or `test_operator_length_count_index_ops`.
# Existing seeds cover `pow`, `not_`, `truth`. This seed asserts the
# division/modulus quartet (`truediv` returns float, `floordiv` returns
# int, `mod` matches Python `%`), the unary numeric trio (`neg`, `pos`,
# `abs`), and the identity-flavored predicates (`is_`, `is_not`).
import operator as op
_ledger: list[int] = []

# Division/modulus quartet
assert op.truediv(10, 4) == 2.5; _ledger.append(1)
assert op.truediv(7, 2) == 3.5; _ledger.append(1)
assert op.floordiv(10, 3) == 3; _ledger.append(1)
assert op.floordiv(-10, 3) == -4; _ledger.append(1)
assert op.mod(10, 3) == 1; _ledger.append(1)
assert op.mod(-10, 3) == 2; _ledger.append(1)

# Unary numeric trio
assert op.neg(5) == -5; _ledger.append(1)
assert op.neg(-5) == 5; _ledger.append(1)
assert op.neg(0) == 0; _ledger.append(1)
assert op.pos(5) == 5; _ledger.append(1)
assert op.pos(-5) == -5; _ledger.append(1)
assert op.abs(-7) == 7; _ledger.append(1)
assert op.abs(7) == 7; _ledger.append(1)
assert op.abs(0) == 0; _ledger.append(1)

# Identity predicates
assert op.is_(None, None); _ledger.append(1)
assert op.is_not(1, 2); _ledger.append(1)
assert not op.is_not(None, None); _ledger.append(1)

# Coherence: truediv * b == a for clean divisions
assert op.truediv(8, 2) * 2 == 8; _ledger.append(1)
# floordiv + mod reconstruct (a == q*b + r) for positive a, b
q = op.floordiv(17, 5)
r = op.mod(17, 5)
assert q * 5 + r == 17; _ledger.append(1)
# neg(neg(x)) == x
assert op.neg(op.neg(42)) == 42; _ledger.append(1)
# abs is idempotent
assert op.abs(op.abs(-9)) == op.abs(-9); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_operator_arith_extras_ops {sum(_ledger)} asserts")
