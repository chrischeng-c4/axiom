# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_arithmetic_edge"
# subject = "cpython321.lang_arithmetic_edge"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_arithmetic_edge.py"
# status = "filled"
# ///
"""cpython321.lang_arithmetic_edge: execute CPython 3.12 seed lang_arithmetic_edge"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for arithmetic edge-cases that the
# basic test_int_ops / test_builtin_conversion_ops fixtures don't
# already cover.
# Surface: floor-division and modulo on negative operands (Python's
# `result follows divisor sign` convention); mixed int/float
# promotion under +; int truncation through int(float); int(str,
# base) for hex (16) and binary (2); float() parsing of `inf`;
# divmod on positive and negative numerators; bool subtype of int
# (True == 1, False == 0).
_ledger: list[int] = []

# Positive floor div + mod: 7 = 3*2 + 1
assert 7 // 3 == 2; _ledger.append(1)
assert 7 % 3 == 1; _ledger.append(1)

# Negative numerator: Python convention is `result follows divisor
# sign`. With divisor positive, the modulo is non-negative.
assert -7 // 3 == -3; _ledger.append(1)
assert -7 % 3 == 2; _ledger.append(1)
# Verify the invariant a == (a // b) * b + (a % b)
assert (-7 // 3) * 3 + (-7 % 3) == -7; _ledger.append(1)

# // on floats yields a float
assert 7.0 // 2.0 == 3.0; _ledger.append(1)
assert isinstance(7.0 // 2.0, float); _ledger.append(1)

# Mixed-type addition int + float promotes to float
mixed = 1 + 2.0
assert mixed == 3.0; _ledger.append(1)
assert isinstance(mixed, float); _ledger.append(1)
# All-int stays int
assert isinstance(1 + 2, int); _ledger.append(1)

# int(float) truncates toward zero — drops the fractional part
assert int(3.7) == 3; _ledger.append(1)
assert int(-3.7) == -3; _ledger.append(1)

# int(str, base): explicit base parses non-decimal strings
assert int("ff", 16) == 255; _ledger.append(1)
assert int("1010", 2) == 10; _ledger.append(1)
assert int("77", 8) == 63; _ledger.append(1)

# float("inf") parses the special positive-infinity literal
fi = float("inf")
assert fi > 1e308; _ledger.append(1)
# float("-inf") yields negative infinity
nfi = float("-inf")
assert nfi < -1e308; _ledger.append(1)

# divmod returns (quotient, remainder) as a single tuple
assert divmod(17, 5) == (3, 2); _ledger.append(1)
# divmod with negative numerator follows the same convention as // and %
assert divmod(-17, 5) == (-4, 3); _ledger.append(1)

# bool is a subtype of int: True == 1, False == 0
assert True == 1; _ledger.append(1)
assert False == 0; _ledger.append(1)
# Arithmetic on bools promotes to int values
assert True + True == 2; _ledger.append(1)
assert True + False == 1; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_arithmetic_edge {sum(_ledger)} asserts")
