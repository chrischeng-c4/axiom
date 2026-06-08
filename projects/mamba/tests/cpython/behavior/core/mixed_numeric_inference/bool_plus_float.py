# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "bool_plus_float"
# subject = "bool + float promotes to float value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bool participates as int 1/0 then promotes to float (True + 1.0 == 2.0)."""
r = True + 1.0
assert r == 2.0, r
assert isinstance(r, float), type(r)
assert (False + 0.5) == 0.5, False + 0.5
print("bool_plus_float OK")
