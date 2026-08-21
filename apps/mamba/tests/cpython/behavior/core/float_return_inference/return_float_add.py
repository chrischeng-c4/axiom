# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_add"
# subject = "float add return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a+b on floats must yield the correct float sum."""


def add(a, b):
    return a + b


r = add(1.25, 2.5)
assert r == 3.75, r
assert isinstance(r, float), type(r)
assert r - 0.75 == 3.0, r
print("return_float_add OK")
