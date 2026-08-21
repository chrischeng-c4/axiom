# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "quantize_half_up"
# subject = "decimal.Decimal.quantize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.quantize: quantize(Decimal('0.01'), rounding=ROUND_HALF_UP) rounds 3.14159 to '3.14'"""
from decimal import Decimal, ROUND_HALF_UP

_q = Decimal("3.14159").quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
assert str(_q) == "3.14", f"quantize ROUND_HALF_UP = {str(_q)!r}"

print("quantize_half_up OK")
