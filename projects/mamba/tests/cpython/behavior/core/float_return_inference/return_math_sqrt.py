# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_math_sqrt"
# subject = "function returning math.sqrt result"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning math.sqrt(x) must yield the correct float root."""
import math


def root(x):
    return math.sqrt(x)


r = root(16.0)
assert r == 4.0, r
assert isinstance(r, float), type(r)
assert root(2.0) == math.sqrt(2.0), root(2.0)
print("return_math_sqrt OK")
