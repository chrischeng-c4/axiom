# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "as_integer_ratio_reduced"
# subject = "decimal.Decimal.as_integer_ratio"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.as_integer_ratio: as_integer_ratio gives a reduced p/q with q>0: Decimal('1.25').as_integer_ratio() == (5, 4)"""
from decimal import Decimal

# as_integer_ratio gives a reduced p/q with q > 0 reconstructing the value.
_p, _q = Decimal("1.25").as_integer_ratio()
assert (_p, _q) == (5, 4), f"as_integer_ratio(1.25) = {(_p, _q)}"

print("as_integer_ratio_reduced OK")
