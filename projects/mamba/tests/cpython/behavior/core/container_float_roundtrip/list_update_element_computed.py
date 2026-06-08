# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_update_element_computed"
# subject = "list element overwritten with a computed float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Overwriting a list element with a computed float must store the correct float value."""
xs = [0.0, 0.0, 0.0]
xs[1] = 3.0 / 2.0
got = xs[1]
assert got == 1.5, got
assert isinstance(got, float), type(got)
assert xs == [0.0, 1.5, 0.0], xs
print("list_update_element_computed OK")
