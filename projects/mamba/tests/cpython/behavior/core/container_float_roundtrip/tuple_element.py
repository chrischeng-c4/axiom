# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "tuple_element"
# subject = "float stored in a tuple and read back by index"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float held in a tuple must read back as the same float by index."""
t = (1.5, 2.75, 4.0)
assert t[0] == 1.5, t[0]
assert t[1] == 2.75, t[1]
assert t[2] == 4.0, t[2]
assert isinstance(t[1], float), type(t[1])
print("tuple_element OK")
