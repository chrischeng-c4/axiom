# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_modulo"
# subject = "float modulo return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a%b on floats must yield the correct float remainder."""


def remainder(a, b):
    return a % b


r = remainder(5.5, 2.0)
assert r == 1.5, r
assert isinstance(r, float), type(r)
assert r + 4.0 == 5.5, r
print("return_float_modulo OK")
