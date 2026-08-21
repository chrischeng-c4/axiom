# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_recursive_float"
# subject = "recursion accumulating a float return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A recursive function summing reciprocals must return the correct accumulated float."""


def harmonic(n):
    if n <= 0:
        return 0.0
    return 1.0 / n + harmonic(n - 1)


r = harmonic(4)
assert r == 1.0 + 0.5 + (1.0 / 3) + 0.25, r
assert isinstance(r, float), type(r)
assert harmonic(1) == 1.0, harmonic(1)
print("return_recursive_float OK")
