# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_param"
# subject = "float parameter returned unchanged"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a float parameter unchanged must yield that exact float."""


def identity(x):
    return x


r = identity(2.75)
assert r == 2.75, r
assert isinstance(r, float), type(r)
assert r * 4.0 == 11.0, r
print("return_float_param OK")
