# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "nan_propagates"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: NaN propagates through arithmetic: Decimal('NaN') + Decimal('1') is_nan()"""
from decimal import Decimal

_nan = Decimal("NaN")
_nan_result = _nan + Decimal("1")
assert _nan_result.is_nan(), "NaN + 1 is NaN"

print("nan_propagates OK")
