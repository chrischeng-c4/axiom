# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_userfunc_float"
# subject = "list comprehension element from user function returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension whose element is a user-func call returning float stores the correct float."""


def ff(j):
    return j + 0.5


xs = [ff(j) for j in range(4)]
assert xs == [0.5, 1.5, 2.5, 3.5], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_userfunc_float OK")
