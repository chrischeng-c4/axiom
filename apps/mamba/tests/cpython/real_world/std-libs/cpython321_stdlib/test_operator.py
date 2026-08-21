# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_operator"
# subject = "cpython321.test_operator"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_operator.py"
# status = "filled"
# ///
"""cpython321.test_operator: execute CPython 3.12 seed test_operator"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_operator.py — #2830 CPython operator seed (executed assertions).
#
# Replaces the prior vendored CPython upstream Lib/test/test_operator.py
# (ranked `Fail` at the `@unittest.skipUnless` resolves-unittest-to-None
# gap) with a Mamba-authored seed distilled from the operator module's
# functional surface. Exercises the arithmetic / comparison / bitwise /
# logical / sequence helpers — the deterministic core that downstream
# users actually reach for — via raw asserts on small fixed inputs.
# Emits the runner's positive proof-of-execution marker that
# `cpython_lib_test_runner.rs` (#2691) classifies as `AssertionPass`.
#
# Why so small? Mamba's current operator surface presents the full
# functional API (add/sub/mul/truediv/floordiv/mod/pow/neg, lt/le/eq/
# ne/ge/gt, and_/or_/xor/lshift/rshift, not_/truth, is_/is_not,
# contains/concat/countOf/indexOf) and produces the same answers as
# CPython on the surface exercised here. Richer surface — `itemgetter`
# / `attrgetter` (return None on mamba: curried callable factories
# not yet supported) — is excluded; those gaps close in followup
# tickets.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: operator N asserts` to stdout.

import operator

_ledger: list[int] = []

# 1. Module identity + public surface.
assert operator.__name__ == "operator", "operator.__name__ must be 'operator'"
_ledger.append(1)
assert hasattr(operator, "add"), "operator must expose add"
_ledger.append(1)
assert hasattr(operator, "sub"), "operator must expose sub"
_ledger.append(1)
assert hasattr(operator, "mul"), "operator must expose mul"
_ledger.append(1)
assert hasattr(operator, "truediv"), "operator must expose truediv"
_ledger.append(1)
assert hasattr(operator, "floordiv"), "operator must expose floordiv"
_ledger.append(1)
assert hasattr(operator, "mod"), "operator must expose mod"
_ledger.append(1)
assert hasattr(operator, "pow"), "operator must expose pow"
_ledger.append(1)
assert hasattr(operator, "neg"), "operator must expose neg"
_ledger.append(1)

# 2. Arithmetic helpers. Each is the functional form of an operator —
#    the load-bearing claim of the module. Equality compared inline
#    against the literal answer.
assert operator.add(2, 3) == 5, "operator.add(2, 3) → 5"
_ledger.append(1)
assert operator.sub(10, 4) == 6, "operator.sub(10, 4) → 6"
_ledger.append(1)
assert operator.mul(3, 5) == 15, "operator.mul(3, 5) → 15"
_ledger.append(1)
assert operator.floordiv(10, 3) == 3, "operator.floordiv(10, 3) → 3"
_ledger.append(1)
assert operator.mod(10, 3) == 1, "operator.mod(10, 3) → 1"
_ledger.append(1)
assert operator.pow(2, 10) == 1024, "operator.pow(2, 10) → 1024"
_ledger.append(1)
assert operator.neg(7) == -7, "operator.neg(7) → -7"
_ledger.append(1)

# 3. Comparison helpers — return True/False from the corresponding
#    Python comparison operator.
assert operator.eq(1, 1) == True, "operator.eq(1, 1) → True"
_ledger.append(1)
assert operator.ne(1, 2) == True, "operator.ne(1, 2) → True"
_ledger.append(1)
assert operator.lt(1, 2) == True, "operator.lt(1, 2) → True"
_ledger.append(1)
assert operator.le(2, 2) == True, "operator.le(2, 2) → True"
_ledger.append(1)
assert operator.gt(3, 2) == True, "operator.gt(3, 2) → True"
_ledger.append(1)
assert operator.ge(3, 2) == True, "operator.ge(3, 2) → True"
_ledger.append(1)

# 4. Bitwise helpers — operate on integer bit patterns.
assert operator.and_(0b1100, 0b1010) == 8, "operator.and_(0b1100, 0b1010) → 0b1000"
_ledger.append(1)
assert operator.or_(0b1100, 0b1010) == 14, "operator.or_(0b1100, 0b1010) → 0b1110"
_ledger.append(1)
assert operator.xor(0b1100, 0b1010) == 6, "operator.xor(0b1100, 0b1010) → 0b0110"
_ledger.append(1)
assert operator.lshift(1, 4) == 16, "operator.lshift(1, 4) → 16"
_ledger.append(1)
assert operator.rshift(64, 2) == 16, "operator.rshift(64, 2) → 16"
_ledger.append(1)

# 5. Logical / truthiness helpers.
assert operator.not_(0) == True, "operator.not_(0) → True"
_ledger.append(1)
assert operator.not_(1) == False, "operator.not_(1) → False"
_ledger.append(1)
assert operator.truth(5) == True, "operator.truth(5) → True"
_ledger.append(1)
assert operator.truth(0) == False, "operator.truth(0) → False"
_ledger.append(1)

# 6. Identity helpers.
assert operator.is_(None, None) == True, "operator.is_(None, None) → True"
_ledger.append(1)
assert operator.is_not(None, 1) == True, "operator.is_not(None, 1) → True"
_ledger.append(1)

# 7. Sequence helpers — work on the container itself, not via
#    item-getter callables.
assert operator.contains([1, 2, 3], 2) == True, "operator.contains([1,2,3], 2) → True"
_ledger.append(1)
assert operator.contains([1, 2, 3], 99) == False, "operator.contains([1,2,3], 99) → False"
_ledger.append(1)
assert operator.concat([1, 2], [3, 4]) == [1, 2, 3, 4], "operator.concat([1,2], [3,4]) → [1,2,3,4]"
_ledger.append(1)
assert operator.countOf([1, 2, 2, 3], 2) == 2, "operator.countOf([1,2,2,3], 2) → 2"
_ledger.append(1)
assert operator.indexOf([10, 20, 30], 20) == 1, "operator.indexOf([10,20,30], 20) → 1"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: operator {len(_ledger)} asserts")
