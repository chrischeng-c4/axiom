# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "compare_with_int"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal compares against plain int by mathematical value: Decimal('10') == 10, > 9, < 11"""
from decimal import Decimal

# Decimal compares against a plain int by mathematical value.
assert Decimal("10") == 10, "Decimal == int"
assert Decimal("10") > 9, "Decimal > int"
assert Decimal("10") < 11, "Decimal < int"

print("compare_with_int OK")
