# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_z_negative_zero"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the 'z' format code coerces a negative-zero result to positive zero but keeps a real negative"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# 'z' coerces a negative-zero result to positive zero but keeps a real negative.
assert f("-0.", "z.1f") == "0.0", "z negative-zero coercion"
assert f("-.09", "z.1f") == "-0.1", "z keeps real negative"

print("format_z_negative_zero OK")
