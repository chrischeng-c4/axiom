# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "nested_dict_of_floats"
# subject = "float read back through a nested dict-of-dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float two levels deep in nested dicts must read back as the same float."""
d = {"outer": {"inner": 6.5}}
got = d["outer"]["inner"]
assert got == 6.5, got
assert isinstance(got, float), type(got)
print("nested_dict_of_floats OK")
