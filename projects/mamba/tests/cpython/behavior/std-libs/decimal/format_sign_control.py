# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_sign_control"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the '+' sign flag forces a leading sign with padding under '='"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Sign control: '+' forces a sign, padded under '='.
assert f("123", "=+6") == "+  123", "plus sign with pad"

print("format_sign_control OK")
