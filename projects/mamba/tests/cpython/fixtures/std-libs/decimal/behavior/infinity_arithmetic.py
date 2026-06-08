# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "infinity_arithmetic"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Infinity arithmetic: inf + 1000 == Infinity, -inf == -Infinity, and inf > any finite value"""
from decimal import Decimal

_inf = Decimal("Infinity")
assert _inf + Decimal("1000") == Decimal("Infinity"), "inf + 1000 = inf"
assert -_inf == Decimal("-Infinity"), "-inf"
assert _inf > Decimal("999999"), "inf > large number"

print("infinity_arithmetic OK")
