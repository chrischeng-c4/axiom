# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "prod_type_promotion"
# subject = "math.prod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.prod: math.prod keeps an all-int reduction integer-typed, promotes to float when any factor is float, honours the start kwarg, and returns the empty-product identity (1)"""
import math

assert math.prod([1, 2, 3, 4]) == 24, f"prod ints = {math.prod([1, 2, 3, 4])!r}"
assert isinstance(math.prod([1, 2, 3, 4]), int), "all-int prod stays int"
assert math.prod([1.5, 2]) == 3.0, f"prod with float = {math.prod([1.5, 2])!r}"
assert isinstance(math.prod([1.5, 2]), float), "float factor promotes"
assert math.prod([2, 3, 4], start=2) == 48, f"prod start kwarg = {math.prod([2, 3, 4], start=2)!r}"
assert math.prod([]) == 1, f"empty prod = {math.prod([])!r}"

print("prod_type_promotion OK")
