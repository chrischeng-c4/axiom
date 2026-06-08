# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "quantize_round_down_truncates"
# subject = "decimal.Decimal.quantize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.quantize: ROUND_DOWN truncates toward zero: 3.9 -> '3' and -3.9 -> '-3' to the unit place"""
from decimal import Decimal, ROUND_DOWN

_rd = Decimal("3.9").quantize(Decimal("1"), rounding=ROUND_DOWN)
assert str(_rd) == "3", f"3.9 ROUND_DOWN = {str(_rd)!r}"
_rdn = Decimal("-3.9").quantize(Decimal("1"), rounding=ROUND_DOWN)
assert str(_rdn) == "-3", f"-3.9 ROUND_DOWN = {str(_rdn)!r}"

print("quantize_round_down_truncates OK")
