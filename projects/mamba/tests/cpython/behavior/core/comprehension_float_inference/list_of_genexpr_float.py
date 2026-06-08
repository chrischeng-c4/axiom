# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "list_of_genexpr_float"
# subject = "list() materializing a generator expression of floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""list() over a generator expression of user-func floats materializes the correct floats."""


def ff(j):
    return j + 0.25


xs = list(ff(j) for j in range(4))
assert xs == [0.25, 1.25, 2.25, 3.25], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("list_of_genexpr_float OK")
