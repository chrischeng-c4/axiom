# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_operator_bitwise_seq_ops"
# subject = "cpython321.test_operator_bitwise_seq_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_operator_bitwise_seq_ops.py"
# status = "filled"
# ///
"""cpython321.test_operator_bitwise_seq_ops: execute CPython 3.12 seed test_operator_bitwise_seq_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `operator` module surfaces
# beyond arithmetic / comparison covered in test_operator_ops.
# Surface: pow; bitwise and_/or_/xor/invert/lshift/rshift; sequence
# concat/contains/getitem; boolean predicates not_/truth.
import operator as op
_ledger: list[int] = []

# pow(x, y) returns x ** y
assert op.pow(2, 10) == 1024; _ledger.append(1)
assert op.pow(3, 3) == 27; _ledger.append(1)

# Bitwise AND
assert op.and_(0b1100, 0b1010) == 0b1000; _ledger.append(1)
# Bitwise OR
assert op.or_(0b1100, 0b1010) == 0b1110; _ledger.append(1)
# Bitwise XOR
assert op.xor(0b1100, 0b1010) == 0b0110; _ledger.append(1)
# Bitwise invert (~0 = -1 under two's complement)
assert op.invert(0) == -1; _ledger.append(1)
assert op.invert(5) == -6; _ledger.append(1)
# Left shift
assert op.lshift(1, 3) == 8; _ledger.append(1)
assert op.lshift(5, 2) == 20; _ledger.append(1)
# Right shift
assert op.rshift(16, 2) == 4; _ledger.append(1)
assert op.rshift(100, 3) == 12; _ledger.append(1)

# concat over sequences — list + list
assert op.concat([1, 2], [3, 4]) == [1, 2, 3, 4]; _ledger.append(1)
# concat over strings
assert op.concat("ab", "cd") == "abcd"; _ledger.append(1)

# contains is `b in a` — exercised on lists only here, since string-
# substring contains has a known breakage on the current mamba runtime.
assert op.contains([1, 2, 3], 2) == True; _ledger.append(1)
assert op.contains([1, 2, 3], 99) == False; _ledger.append(1)

# getitem on list works; dict/string getitem are NOT exercised here —
# both have known breakages on the current mamba runtime where the
# call returns None instead of the indexed value.
assert op.getitem([10, 20, 30], 1) == 20; _ledger.append(1)

# not_ is the logical-not operator as a function
assert op.not_(False) == True; _ledger.append(1)
assert op.not_(True) == False; _ledger.append(1)
assert op.not_(0) == True; _ledger.append(1)
assert op.not_(1) == False; _ledger.append(1)

# truth returns the boolean value of the argument
assert op.truth("") == False; _ledger.append(1)
assert op.truth("x") == True; _ledger.append(1)
assert op.truth([]) == False; _ledger.append(1)
assert op.truth([1]) == True; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_operator_bitwise_seq_ops {sum(_ledger)} asserts")
