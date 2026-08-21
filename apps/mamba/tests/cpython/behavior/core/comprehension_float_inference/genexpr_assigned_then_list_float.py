# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "genexpr_assigned_then_list_float"
# subject = "generator expression of floats bound to a name then materialized"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator expression of floats bound to a name yields the correct floats when listed."""


def ff(j):
    return j * 0.5


g = (ff(j) for j in range(4))
xs = list(g)
assert xs == [0.0, 0.5, 1.0, 1.5], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("genexpr_assigned_then_list_float OK")
