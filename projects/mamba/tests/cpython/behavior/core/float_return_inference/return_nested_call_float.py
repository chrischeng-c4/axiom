# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_nested_call_float"
# subject = "float returned from a nested call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""An outer function returning the result of an inner float-returning call must be correct."""


def inner(a, b):
    return a / b


def outer(a, b):
    return inner(a, b) + 0.5


r = outer(7, 2)
assert r == 4.0, r
assert isinstance(r, float), type(r)
assert inner(1, 4) == 0.25, inner(1, 4)
print("return_nested_call_float OK")
