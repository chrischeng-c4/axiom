# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_reciprocal_division"
# subject = "float division return value inside a list comprehension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension element `1.0/(j+1)` yields the correct true-division float."""


xs = [1.0 / (j + 1) for j in range(4)]
assert xs[0] == 1.0, xs
assert xs[1] == 0.5, xs
assert xs[2] == 1.0 / 3.0, xs
assert xs[3] == 0.25, xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_reciprocal_division OK")
