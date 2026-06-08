# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "dict_value_by_key"
# subject = "float stored as a dict value and read back by key"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float stored as a dict value must read back as the same float by key."""
d = {}
d["pi"] = 3.125
d["e"] = 2.5
got = d["pi"]
assert got == 3.125, got
assert isinstance(got, float), type(got)
assert d["e"] == 2.5, d["e"]
print("dict_value_by_key OK")
