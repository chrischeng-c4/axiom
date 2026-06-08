# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_general"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the 'g' format code switches between fixed and scientific and honors precision"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# General: 'g' switches between fixed and scientific.
assert f("0E1", "g") == "0e+1", "g sci zero"
assert f("3.14159265", ".5g") == "3.1416", "g precision"

print("format_general OK")
