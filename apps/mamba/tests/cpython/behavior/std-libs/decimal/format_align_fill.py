# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_align_fill"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: format alignment and fill: default right align, <, >, ^, custom fill char, and fill-after-sign with '='"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Alignment and fill: <, >, ^, and a custom fill character.
assert f("123", "6") == "   123", "default right align"
assert f("123", "<6") == "123   ", "left align"
assert f("123", "^6") == " 123  ", "center align"
assert f("123", "?^5") == "?123?", "custom fill center"
assert f("-45.6", "/=10") == "-/////45.6", "fill after sign"

print("format_align_fill OK")
