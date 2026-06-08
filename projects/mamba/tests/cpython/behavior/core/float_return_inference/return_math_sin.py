# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_math_sin"
# subject = "function returning math.sin result"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning math.sin(x) must yield the correct float."""
import math


def sine(x):
    return math.sin(x)


r = sine(0.0)
assert r == 0.0, r
assert isinstance(r, float), type(r)
assert sine(math.pi / 2) == 1.0, sine(math.pi / 2)
print("return_math_sin OK")
