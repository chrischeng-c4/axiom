# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_special_values"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: special values format as words (NaN/Infinity) and honor sign and alignment"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Special values format as words and honor sign/fill.
assert f("NaN", "e") == "NaN", "NaN passthrough"
assert f("Inf", ".3e") == "Infinity", "Infinity word"
assert f("-Inf", ">10") == " -Infinity", "Infinity aligned"

print("format_special_values OK")
