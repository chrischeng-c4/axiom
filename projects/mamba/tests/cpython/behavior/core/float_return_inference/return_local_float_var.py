# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_local_float_var"
# subject = "float returned via a local variable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function computing a float into a local var then returning it must yield it correctly."""


def compute(a, b):
    total = a / b
    total = total + 1.0
    return total


r = compute(6, 4)
assert r == 2.5, r
assert isinstance(r, float), type(r)
assert r - 1.0 == 1.5, r
print("return_local_float_var OK")
