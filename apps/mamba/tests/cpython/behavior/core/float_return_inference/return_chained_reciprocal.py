# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_chained_reciprocal"
# subject = "chained float division return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning 1.0/(j+1) must yield the correct reciprocal float."""


def reciprocal(j):
    return 1.0 / (j + 1)


r = reciprocal(3)
assert r == 0.25, r
assert isinstance(r, float), type(r)
assert reciprocal(0) == 1.0, reciprocal(0)
assert reciprocal(1) == 0.5, reciprocal(1)
print("return_chained_reciprocal OK")
