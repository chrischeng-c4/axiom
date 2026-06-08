# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "tuple_of_genexpr_float"
# subject = "tuple() materializing a generator expression of floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tuple() over a generator expression of `j/2` floats materializes the correct float tuple."""


def ff(j):
    return j / 2


t = tuple(ff(j) for j in range(4))
assert t == (0.0, 0.5, 1.0, 1.5), t
for v in t:
    assert isinstance(v, float), (v, type(v))
print("tuple_of_genexpr_float OK")
