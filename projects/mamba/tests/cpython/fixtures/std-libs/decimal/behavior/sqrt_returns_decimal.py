# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "sqrt_returns_decimal"
# subject = "decimal.Decimal.sqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.sqrt: sqrt() returns a Decimal: Decimal('9').sqrt() == '3'"""
from decimal import Decimal

_sq = Decimal("9").sqrt()
assert isinstance(_sq, Decimal), f"sqrt type = {type(_sq)!r}"
assert str(_sq) == "3", f"sqrt(9) = {str(_sq)!r}"

print("sqrt_returns_decimal OK")
