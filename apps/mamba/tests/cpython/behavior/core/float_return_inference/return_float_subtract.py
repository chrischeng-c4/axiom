# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_subtract"
# subject = "float subtract return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a-b on floats must yield the correct float difference."""


def sub(a, b):
    return a - b


r = sub(5.5, 2.0)
assert r == 3.5, r
assert isinstance(r, float), type(r)
assert r + 2.0 == 5.5, r
print("return_float_subtract OK")
