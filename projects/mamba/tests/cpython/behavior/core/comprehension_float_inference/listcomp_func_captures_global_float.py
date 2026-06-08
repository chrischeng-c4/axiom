# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_func_captures_global_float"
# subject = "comprehension element from a function reading a global float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A comprehension over a func that multiplies by a global float scale yields correct floats."""


scale = 0.25


def ff(j):
    return j * scale


xs = [ff(j) for j in range(4)]
assert xs == [0.0, 0.25, 0.5, 0.75], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_func_captures_global_float OK")
