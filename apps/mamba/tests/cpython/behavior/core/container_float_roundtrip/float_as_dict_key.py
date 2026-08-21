# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "float_as_dict_key"
# subject = "float used as a dict key"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float used as a dict key must hash/compare by value so lookup returns the stored value."""
d = {1.5: "a", 2.25: "b"}
assert d[1.5] == "a", d[1.5]
assert d[2.25] == "b", d[2.25]
d[1.5] = "updated"
assert d[1.5] == "updated", d[1.5]
assert len(d) == 2, len(d)
print("float_as_dict_key OK")
