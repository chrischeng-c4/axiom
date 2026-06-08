# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "set_membership"
# subject = "float stored in a set tests membership by value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float in a set must answer membership by value (hash/eq must use the float, not box bits)."""
s = set()
s.add(1.5)
s.add(2.25)
assert 1.5 in s, s
assert 2.25 in s, s
assert 9.5 not in s, s
print("set_membership OK")
