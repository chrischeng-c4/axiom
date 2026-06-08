# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_round_sum_default_ops"
# subject = "cpython321.test_round_sum_default_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_round_sum_default_ops.py"
# status = "filled"
# ///
"""cpython321.test_round_sum_default_ops: execute CPython 3.12 seed test_round_sum_default_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for built-in numeric/aggregator
# surface not covered by `test_pow_modular_divmod_ops`,
# `test_sorted_key_ops`, or `lang_any_all_min_max_key_ops`. Those
# seeds cover pow/divmod sign convention, sorted/min/max with key=,
# and any/all vacuous-truth.
# This seed asserts:
#   * round() — Python uses banker's rounding (round half to even)
#     for halfway cases on float arguments: round(0.5)==0, round(1.5)
#     ==2, round(2.5)==2, round(3.5)==4, round(4.5)==4, round(5.5)==6.
#     round(7) returns 7 (passthrough on int). round(3.14159, 2)
#     returns 3.14. round(12345, -2) rounds to nearest 100 = 12300.
#   * sum() with start argument: sum([1,2,3], 10) == 16;
#     sum([]) == 0; sum([], 5) == 5; sum((1,2,3)) == 6 (tuple input);
#     sum(generator expression) works.
#   * min()/max() with default= on the empty iterable.
_ledger: list[int] = []

# round() — banker's rounding on halfway cases
# 0.5 / 1.5: nearest evens are 0 and 2
assert round(0.5) == 0; _ledger.append(1)
assert round(1.5) == 2; _ledger.append(1)
# 2.5 / 3.5: nearest evens are 2 and 4
assert round(2.5) == 2; _ledger.append(1)
assert round(3.5) == 4; _ledger.append(1)
# 4.5 / 5.5: nearest evens are 4 and 6
assert round(4.5) == 4; _ledger.append(1)
assert round(5.5) == 6; _ledger.append(1)
# 10.5 → 10 (10 is even)
assert round(10.5) == 10; _ledger.append(1)
# Negative halfway: -1.5 → -2 (even)
assert round(-1.5) == -2; _ledger.append(1)

# round on int passes through
assert round(0) == 0; _ledger.append(1)
assert round(7) == 7; _ledger.append(1)
# round(float-with-zero-frac) → int 7
assert round(7.0) == 7; _ledger.append(1)

# round with positive ndigits
assert round(3.14159, 2) == 3.14; _ledger.append(1)
assert round(0.123456, 3) == 0.123; _ledger.append(1)

# round with negative ndigits — rounds to nearest 10^|n|
assert round(12345, -2) == 12300; _ledger.append(1)

# sum() — start defaults to 0
assert sum([1, 2, 3, 4, 5]) == 15; _ledger.append(1)
assert sum([]) == 0; _ledger.append(1)

# sum() with explicit start
assert sum([1, 2, 3], 10) == 16; _ledger.append(1)
# Empty iter + start = start
assert sum([], 5) == 5; _ledger.append(1)

# sum() on tuple
assert sum((1, 2, 3)) == 6; _ledger.append(1)
# sum() of floats
assert sum([1.5, 2.5]) == 4.0; _ledger.append(1)

# sum() over a generator expression
assert sum(x for x in [1, 2, 3]) == 6; _ledger.append(1)
assert sum(x * 2 for x in [1, 2, 3]) == 12; _ledger.append(1)

# min() / max() with default= on empty iterable
assert min([], default=99) == 99; _ledger.append(1)
assert max([], default=-1) == -1; _ledger.append(1)
# default= is ignored when iterable is non-empty
assert min([3, 1, 2], default=99) == 1; _ledger.append(1)
assert max([3, 1, 2], default=-1) == 3; _ledger.append(1)

# Return-type invariants
assert isinstance(round(3.5), int); _ledger.append(1)
assert isinstance(round(3.14, 2), float); _ledger.append(1)
assert isinstance(sum([1, 2, 3]), int); _ledger.append(1)
assert isinstance(sum([1.5, 2.5]), float); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_round_sum_default_ops {sum(_ledger)} asserts")
