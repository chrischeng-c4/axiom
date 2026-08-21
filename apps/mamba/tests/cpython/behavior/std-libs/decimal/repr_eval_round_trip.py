# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "repr_eval_round_trip"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: repr(Decimal) round-trips through eval back to an equal Decimal for several tuple-built values"""
from decimal import Decimal

# repr round-trips through eval back to an equal Decimal.
for _src in [(0, (0,), 0), (1, (4, 5), 0), (0, (4, 5, 3, 4), -2)]:
    _d = Decimal(_src)
    assert _d == eval(repr(_d)), f"eval(repr) round trip {_src}"

print("repr_eval_round_trip OK")
