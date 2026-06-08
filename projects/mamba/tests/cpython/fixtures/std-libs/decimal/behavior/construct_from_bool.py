# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "construct_from_bool"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal(True) == Decimal(1), Decimal(False) == Decimal(0), and bool(Decimal) reflects nonzero-ness"""
from decimal import Decimal

# From bool: True == 1, False == 0; truthiness of Decimal.
assert Decimal(True) == Decimal(1), "Decimal(True)"
assert Decimal(False) == Decimal(0), "Decimal(False)"
assert bool(Decimal(0)) is False, "bool(Decimal(0))"
assert bool(Decimal("0.372")) is True, "bool(nonzero)"

print("construct_from_bool OK")
