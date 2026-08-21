# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "construct_from_float_exact"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal(float) captures the exact binary expansion (Decimal(0.1) is the long 0.1000...625 string), and special floats map to qnan/inf/-0"""
from decimal import Decimal

# From float: exact binary expansion, plus special values.
assert str(Decimal(0.1)) == (
    "0.1000000000000000055511151231257827021181583404541015625"
), "Decimal(0.1) exact binary expansion"
assert Decimal(float("nan")).is_qnan(), "Decimal(nan)"
assert Decimal(float("inf")).is_infinite(), "Decimal(inf)"
assert str(Decimal(float("-0.0"))) == "-0", "Decimal(-0.0)"

print("construct_from_float_exact OK")
