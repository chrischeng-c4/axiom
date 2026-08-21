# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_power"
# subject = "float power return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a**b producing a float must yield the correct float."""


def power(a, b):
    return a ** b


r = power(2.0, 3.0)
assert r == 8.0, r
assert isinstance(r, float), type(r)
assert r / 2.0 == 4.0, r
print("return_float_power OK")
