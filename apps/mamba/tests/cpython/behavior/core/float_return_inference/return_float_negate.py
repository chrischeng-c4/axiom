# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_negate"
# subject = "float unary negation return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning -a on a float must yield the correct negated float."""


def negate(a):
    return -a


r = negate(3.5)
assert r == -3.5, r
assert isinstance(r, float), type(r)
assert r + 3.5 == 0.0, r
print("return_float_negate OK")
