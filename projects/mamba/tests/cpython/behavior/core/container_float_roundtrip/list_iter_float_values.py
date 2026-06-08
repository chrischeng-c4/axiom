# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_iter_float_values"
# subject = "iterating a list of floats yields the stored floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Iterating a list of floats must yield each stored float, not a leaked box-bits int."""
xs = [0.5, 1.5, 2.5]
collected = []
for v in xs:
    assert isinstance(v, float), type(v)
    collected.append(v)
assert collected == [0.5, 1.5, 2.5], collected
assert sum(xs) == 4.5, sum(xs)
print("list_iter_float_values OK")
