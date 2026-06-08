# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "quantize_half_even_bankers"
# subject = "decimal.Decimal.quantize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.quantize: ROUND_HALF_EVEN (banker's rounding) rounds 2.5 -> '2' and 3.5 -> '4' to the unit place"""
from decimal import Decimal, ROUND_HALF_EVEN

_r2 = Decimal("2.5").quantize(Decimal("1"), rounding=ROUND_HALF_EVEN)
assert str(_r2) == "2", f"2.5 HALF_EVEN = {str(_r2)!r}"
_r3 = Decimal("3.5").quantize(Decimal("1"), rounding=ROUND_HALF_EVEN)
assert str(_r3) == "4", f"3.5 HALF_EVEN = {str(_r3)!r}"

print("quantize_half_even_bankers OK")
