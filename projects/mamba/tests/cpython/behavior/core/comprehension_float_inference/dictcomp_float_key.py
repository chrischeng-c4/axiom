# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "dictcomp_float_key"
# subject = "dict comprehension key from a user function returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A dict comprehension whose keys are user-func floats stores correct float keys."""


def ff(j):
    return j + 0.5


d = {ff(j): j for j in range(4)}
keys = sorted(d.keys())
assert keys == [0.5, 1.5, 2.5, 3.5], keys
for k in keys:
    assert isinstance(k, float), (k, type(k))
assert d[0.5] == 0, d
assert d[3.5] == 3, d
print("dictcomp_float_key OK")
