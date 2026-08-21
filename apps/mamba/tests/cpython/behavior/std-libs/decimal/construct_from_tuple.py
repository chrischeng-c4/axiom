# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "construct_from_tuple"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal((sign, digits, exponent)) builds the value, and a 'F' exponent yields Infinity"""
from decimal import Decimal

# From tuple (sign, digits, exponent); 'F' exponent = Infinity.
assert str(Decimal((0, (0,), 0))) == "0", "tuple 0"
assert str(Decimal((1, (4, 5), 0))) == "-45", "tuple -45"
assert str(Decimal((0, (4, 5, 3, 4), -2))) == "45.34", "tuple 45.34"
assert str(Decimal((0, (), "F"))) == "Infinity", "tuple Infinity"

print("construct_from_tuple OK")
