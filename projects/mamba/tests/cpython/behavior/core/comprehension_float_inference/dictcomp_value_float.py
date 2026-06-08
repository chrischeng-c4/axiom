# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "dictcomp_value_float"
# subject = "dict comprehension value from a user function returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A dict comprehension whose values are user-func floats stores the correct float values."""


def ff(j):
    return j * 1.5


d = {j: ff(j) for j in range(4)}
vals = [d[k] for k in sorted(d)]
assert vals == [0.0, 1.5, 3.0, 4.5], vals
for v in vals:
    assert isinstance(v, float), (v, type(v))
print("dictcomp_value_float OK")
