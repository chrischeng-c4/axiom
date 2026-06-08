# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_true_division"
# subject = "float division return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a/b (true division of ints) must yield the correct float."""


def divide(a, b):
    return a / b


r = divide(7, 2)
assert r == 3.5, r
assert isinstance(r, float), type(r)
assert r * 2 == 7.0, r
print("return_true_division OK")
