# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "parse_float_hook_decimal"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_misc.py"
# status = "filled"
# ///
"""tomllib.loads: the parse_float hook routes every float token through a custom callable (Decimal), including inf and nan, leaving other types untouched"""
import tomllib
from decimal import Decimal

_doc = """
val = 0.1
biggest = inf
smallest = -inf
notnum = nan
count = 7
"""
_obj = tomllib.loads(_doc, parse_float=Decimal)

assert isinstance(_obj["val"], Decimal), f"val type = {type(_obj['val'])!r}"
assert _obj["val"] == Decimal("0.1"), f"val = {_obj['val']!r}"
assert isinstance(_obj["biggest"], Decimal) and _obj["biggest"] == Decimal("inf"), f"biggest = {_obj['biggest']!r}"
assert isinstance(_obj["smallest"], Decimal) and _obj["smallest"] == Decimal("-inf"), f"smallest = {_obj['smallest']!r}"
assert isinstance(_obj["notnum"], Decimal) and _obj["notnum"].is_nan(), f"notnum = {_obj['notnum']!r}"
# Integers are not routed through parse_float — they stay plain ints.
assert _obj["count"] == 7 and not isinstance(_obj["count"], Decimal), f"count = {_obj['count']!r}"

print("parse_float_hook_decimal OK")
