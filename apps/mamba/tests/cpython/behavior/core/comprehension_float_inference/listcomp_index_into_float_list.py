# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_index_into_float_list"
# subject = "comprehension element from indexing a float list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension element `data[j]` indexing a float list preserves the float values."""


data = [0.1, 0.2, 0.3, 0.4]
xs = [data[j] for j in range(4)]
assert xs == [0.1, 0.2, 0.3, 0.4], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_index_into_float_list OK")
