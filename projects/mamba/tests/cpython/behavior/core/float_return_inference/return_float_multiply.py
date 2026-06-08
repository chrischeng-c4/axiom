# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_multiply"
# subject = "float multiply return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a*b on floats must yield the correct float product."""


def scale(a, b):
    return a * b


r = scale(2.5, 4.0)
assert r == 10.0, r
assert isinstance(r, float), type(r)
assert r + 0.5 == 10.5, r
print("return_float_multiply OK")
