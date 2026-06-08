# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "setcomp_userfunc_float"
# subject = "set comprehension element from a user function returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A set comprehension of user-func float results contains the correct float members."""


def ff(j):
    return j + 0.5


s = {ff(j) for j in range(4)}
assert s == {0.5, 1.5, 2.5, 3.5}, s
for v in s:
    assert isinstance(v, float), (v, type(v))
print("setcomp_userfunc_float OK")
