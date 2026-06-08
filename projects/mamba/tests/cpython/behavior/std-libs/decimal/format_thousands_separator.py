# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_thousands_separator"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the ',' format code groups thousands, works on negatives, and combines with zero-padding"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Thousands separator: ','.
assert f("1234567", ",") == "1,234,567", "comma grouping"
assert f("-123456", ",") == "-123,456", "comma negative"
assert f("1234.56", "09,") == "01,234.56", "zero-pad with comma"

print("format_thousands_separator OK")
