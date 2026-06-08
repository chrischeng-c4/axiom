# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "as_tuple_strips_leading_zeros"
# subject = "decimal.Decimal.as_tuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.as_tuple: as_tuple normalizes leading-zero payloads: Decimal((1,(0,0,0),37)).as_tuple() == (1,(0,),37)"""
from decimal import Decimal

# as_tuple normalizes leading-zero payloads.
assert Decimal((1, (0, 0, 0), 37)).as_tuple() == (1, (0,), 37), "leading zeros stripped"

print("as_tuple_strips_leading_zeros OK")
