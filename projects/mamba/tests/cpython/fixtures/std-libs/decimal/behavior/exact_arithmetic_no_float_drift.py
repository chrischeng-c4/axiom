# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "exact_arithmetic_no_float_drift"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal('0.1') + Decimal('0.2') == Decimal('0.3') exactly (str is '0.3'), unlike binary float where 0.1+0.2 != 0.3"""
from decimal import Decimal

# Decimal arithmetic is exact — it avoids the binary-float representation error.
assert Decimal("0.1") + Decimal("0.2") == Decimal("0.3"), "0.1+0.2 == 0.3"
assert str(Decimal("0.1") + Decimal("0.2")) == "0.3", f"str(0.1+0.2) = {str(Decimal('0.1') + Decimal('0.2'))!r}"
# Contrast: the same sum in binary float drifts off 0.3.
assert 0.1 + 0.2 != 0.3, "float 0.1+0.2 != 0.3 (contrast)"
print("exact_arithmetic_no_float_drift OK")
