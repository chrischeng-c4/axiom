# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_fixed"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the 'f' format code uses fixed notation, keeps trailing zeros from scale, and rounds/pads at explicit precision"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Fixed notation: 'f' with explicit precision rounds/pads.
assert f("3.2E2", "f") == "320", "f no fraction"
assert f("3.200E2", "f") == "320.0", "f keeps trailing"
assert f("3.14159265", ".4f") == "3.1416", "f precision"

print("format_fixed OK")
