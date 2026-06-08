# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_append_index"
# subject = "float stored in a list and read back by index"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float appended to a list must read back as the same float (no NaN-box-as-int leak)."""
xs = []
xs.append(3.5)
xs.append(0.25)
got = xs[0]
assert got == 3.5, got
assert isinstance(got, float), type(got)
assert xs[1] == 0.25, xs[1]
print("list_append_index OK")
