# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "cross_type_vs_float"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal vs float compares by value: Decimal('0.25') == 0.25 and < 3.0, but Decimal('0.1') != float 0.1 (binary inexactness)"""
from decimal import Decimal

da = Decimal("0.25")
db = Decimal("3.0")
# Decimal vs float: ordering and equality compare by mathematical value.
assert da < 3.0 and db > 0.25, "ordering vs float"
assert da == 0.25, "Decimal('0.25') == 0.25"
# 0.1 is not exactly representable as binary float, so it differs from the
# exact Decimal('0.1').
assert Decimal("0.1") != 0.1, "Decimal('0.1') != float 0.1"

print("cross_type_vs_float OK")
