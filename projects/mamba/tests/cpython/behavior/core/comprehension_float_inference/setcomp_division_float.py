# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "setcomp_division_float"
# subject = "set comprehension element from true division yielding float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A set comprehension `j/4` yields the correct distinct float members."""


s = {j / 4 for j in range(4)}
assert s == {0.0, 0.25, 0.5, 0.75}, s
for v in s:
    assert isinstance(v, float), (v, type(v))
print("setcomp_division_float OK")
