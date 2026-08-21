# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "exact_large_int_multiply"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal multiplication is exact for large integers: 123456789 * 987654321 == 121932631112635269"""
from decimal import Decimal

_m = Decimal("123456789") * Decimal("987654321")
assert str(_m) == "121932631112635269", f"large int multiply = {str(_m)!r}"

print("exact_large_int_multiply OK")
