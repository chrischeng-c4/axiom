# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_int_int_division_is_float"
# subject = "int/int true division returns an exact-valued float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int/int true division that is mathematically whole must still return a float, not an int."""


def divide(a, b):
    return a / b


r = divide(6, 3)
assert r == 2.0, r
assert isinstance(r, float), type(r)
assert r != 6, r
assert r / 2 == 1.0, r
print("return_int_int_division_is_float OK")
