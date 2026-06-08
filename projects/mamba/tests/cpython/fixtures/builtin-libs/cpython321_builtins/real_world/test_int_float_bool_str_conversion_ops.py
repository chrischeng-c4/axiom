# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_int_float_bool_str_conversion_ops"
# subject = "cpython321.test_int_float_bool_str_conversion_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_int_float_bool_str_conversion_ops.py"
# status = "filled"
# ///
"""cpython321.test_int_float_bool_str_conversion_ops: execute CPython 3.12 seed test_int_float_bool_str_conversion_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the int/float/bool/str conversion
# constructors. Surface not covered by `test_builtin_conversion_ops`
# (bin/oct/hex prefixes, chr/ord/divmod/abs/pow/round/repr/ascii) or
# `test_chr_ord_hex_repr_ops` (chr/ord/hex/oct/bin). This seed asserts:
#   * int(str, base) for bases 2, 8, 10, 16, with and without the
#     literal prefix (0b, 0o, 0x);
#   * int("-5") parses negative numbers;
#   * float() parses regular decimals, scientific notation, and the
#     special "inf"/"-inf" tokens (round-trip via ==);
#   * float("nan") is a NaN — verify via `x != x` self-inequality;
#   * str() rendering for int, float, bool, None, and list;
#   * bool() truthiness over the canonical falsy set
#     (0, 0.0, "", [], None) and over canonical truthy values.
import math
_ledger: list[int] = []

# int(str) — decimal default
assert int("10") == 10; _ledger.append(1)
assert int("-5") == -5; _ledger.append(1)
assert int("0") == 0; _ledger.append(1)
assert int("123456") == 123456; _ledger.append(1)

# int(str, base) — base 2 / 8 / 16 without prefix
assert int("10", 2) == 2; _ledger.append(1)      # 0b10 == 2
assert int("101", 2) == 5; _ledger.append(1)
assert int("777", 8) == 511; _ledger.append(1)   # 7*64 + 7*8 + 7
assert int("10", 16) == 16; _ledger.append(1)
assert int("ff", 16) == 255; _ledger.append(1)
assert int("FF", 16) == 255; _ledger.append(1)   # case-insensitive

# int(str, base) — with literal prefix
assert int("0x10", 16) == 16; _ledger.append(1)
assert int("0b101", 2) == 5; _ledger.append(1)
assert int("0o17", 8) == 15; _ledger.append(1)

# float() — regular decimals
assert float("3.14") == 3.14; _ledger.append(1)
assert float("-2.5") == -2.5; _ledger.append(1)
assert float("0") == 0.0; _ledger.append(1)
assert float("0.0") == 0.0; _ledger.append(1)

# float() — scientific notation
assert float("1e5") == 100000.0; _ledger.append(1)
assert float("2.5e3") == 2500.0; _ledger.append(1)
assert float("1e-3") == 0.001; _ledger.append(1)

# float() — special tokens; inf is comparable via math.isinf
assert math.isinf(float("inf")); _ledger.append(1)
assert math.isinf(float("-inf")); _ledger.append(1)
assert float("inf") > 1e300; _ledger.append(1)
assert float("-inf") < -1e300; _ledger.append(1)
# float("nan") is a NaN — math.isnan recognises it
nan = float("nan")
assert math.isnan(nan); _ledger.append(1)

# str() rendering for fundamental types
assert str(42) == "42"; _ledger.append(1)
assert str(-7) == "-7"; _ledger.append(1)
assert str(0) == "0"; _ledger.append(1)
assert str(3.14) == "3.14"; _ledger.append(1)
assert str(True) == "True"; _ledger.append(1)
assert str(False) == "False"; _ledger.append(1)
assert str(None) == "None"; _ledger.append(1)
assert str([]) == "[]"; _ledger.append(1)
assert str([1, 2]) == "[1, 2]"; _ledger.append(1)

# bool() — canonical falsy set
assert bool(0) == False; _ledger.append(1)
assert bool(0.0) == False; _ledger.append(1)
assert bool("") == False; _ledger.append(1)
assert bool([]) == False; _ledger.append(1)
assert bool(None) == False; _ledger.append(1)

# bool() — canonical truthy set
assert bool(1) == True; _ledger.append(1)
assert bool(-1) == True; _ledger.append(1)
assert bool(0.5) == True; _ledger.append(1)
assert bool("x") == True; _ledger.append(1)
assert bool(" ") == True; _ledger.append(1)  # space is non-empty
assert bool([0]) == True; _ledger.append(1)   # list with one falsy is still truthy

print(f"MAMBA_ASSERTION_PASS: test_int_float_bool_str_conversion_ops {sum(_ledger)} asserts")
