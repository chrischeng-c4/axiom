# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_capture_outer_float"
# subject = "comprehension element adding a captured outer float variable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension that captures an outer float variable produces the correct sums."""


base = 0.5
xs = [base + j for j in range(4)]
assert xs == [0.5, 1.5, 2.5, 3.5], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_capture_outer_float OK")
