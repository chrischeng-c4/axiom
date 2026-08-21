# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "as_tuple_inverse"
# subject = "decimal.Decimal.as_tuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.as_tuple: as_tuple() is the inverse of tuple construction for finite values and Infinity"""
from decimal import Decimal

# as_tuple is the inverse of tuple construction for finite values and Infinity.
assert Decimal(0).as_tuple() == (0, (0,), 0), "as_tuple 0"
assert Decimal(-45).as_tuple() == (1, (4, 5), 0), "as_tuple -45"
assert Decimal("Infinity").as_tuple() == (0, (0,), "F"), "as_tuple inf"

print("as_tuple_inverse OK")
