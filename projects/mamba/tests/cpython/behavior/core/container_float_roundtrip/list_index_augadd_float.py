# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_index_augadd_float"
# subject = "augmented add of a float into a list element (list[i] += float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""list[i] += float must read, add, and store back the correct float value."""
xs = [1.0, 2.0, 3.0]
xs[1] += 0.5
assert xs[1] == 2.5, xs[1]
xs[1] += 0.5
assert xs[1] == 3.0, xs[1]
assert isinstance(xs[1], float), type(xs[1])
assert xs == [1.0, 3.0, 3.0], xs
print("list_index_augadd_float OK")
