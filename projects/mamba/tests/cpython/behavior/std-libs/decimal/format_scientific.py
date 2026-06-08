# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_scientific"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the 'e' format code normalizes to scientific notation with a signed exponent and rounds at the requested precision"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Scientific notation: 'e' normalizes the exponent with a sign.
assert f("1.5", "e") == "1.5e+0", "e basic"
assert f("0.015", "e") == "1.5e-2", "e small"
assert f("9.9999999", ".6e") == "1.000000e+1", "e precision rounds"

print("format_scientific OK")
