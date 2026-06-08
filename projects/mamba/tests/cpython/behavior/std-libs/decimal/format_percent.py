# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_percent"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the '%' format code scales by 100 and appends '%'"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Percent: '%' scales by 100 and appends '%'.
assert f("2.34", ".3%") == "234.000%", "percent"
assert f("1.23", ".0%") == "123%", "percent no fraction"

print("format_percent OK")
