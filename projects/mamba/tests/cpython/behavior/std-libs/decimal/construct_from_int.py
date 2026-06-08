# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "construct_from_int"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal from int is exact and sign-preserving across 45, -45, large, and 0"""
from decimal import Decimal

# From int: exact, sign preserved, arbitrary precision.
assert str(Decimal(45)) == "45", "int 45"
assert str(Decimal(-45)) == "-45", "int -45"
assert str(Decimal(500000123)) == "500000123", "int large"
assert str(Decimal(0)) == "0", "int 0"

print("construct_from_int OK")
