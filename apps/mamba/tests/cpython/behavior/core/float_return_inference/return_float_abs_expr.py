# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_abs_expr"
# subject = "float absolute-value return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning abs() of a computed float must yield the correct float."""


def magnitude(a, b):
    return abs(a - b)


r = magnitude(2.0, 5.5)
assert r == 3.5, r
assert isinstance(r, float), type(r)
assert r * 2.0 == 7.0, r
print("return_float_abs_expr OK")
