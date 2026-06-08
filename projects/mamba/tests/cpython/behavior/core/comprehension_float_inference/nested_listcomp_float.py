# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "nested_listcomp_float"
# subject = "nested list comprehension element from a two-arg user func returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A nested `[[ff(i,j) for j] for i]` comprehension stores the correct float matrix."""


def ff(i, j):
    return i * 10.0 + j * 0.5


m = [[ff(i, j) for j in range(2)] for i in range(2)]
assert m == [[0.0, 0.5], [10.0, 10.5]], m
for row in m:
    for v in row:
        assert isinstance(v, float), (v, type(v))
print("nested_listcomp_float OK")
