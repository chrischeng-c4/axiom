# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "dict_str_to_float"
# subject = "dict[str, float] read back by string key"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A dict mapping str keys to float values must read each float back unchanged."""
prices = {"apple": 1.25, "pear": 2.5, "plum": 0.75}
assert prices["apple"] == 1.25, prices["apple"]
assert prices["pear"] == 2.5, prices["pear"]
assert prices["plum"] == 0.75, prices["plum"]
total = prices["apple"] + prices["pear"] + prices["plum"]
assert total == 4.5, total
print("dict_str_to_float OK")
