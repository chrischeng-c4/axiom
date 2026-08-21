# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_filter_float"
# subject = "filtered list comprehension whose surviving elements are floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A filtered list comprehension `j*0.5 for j if j%2==0` keeps the correct surviving floats."""


xs = [j * 0.5 for j in range(8) if j % 2 == 0]
assert xs == [0.0, 1.0, 2.0, 3.0], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_filter_float OK")
