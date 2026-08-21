# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_scale_iterable_floats"
# subject = "scaling float iterable elements by a float literal in a comprehension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension `x*0.5` over a float iterable yields the correct scaled floats."""


xs = [x * 0.5 for x in [2.0, 4.0, 6.0, 7.0]]
assert xs == [1.0, 2.0, 3.0, 3.5], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_scale_iterable_floats OK")
