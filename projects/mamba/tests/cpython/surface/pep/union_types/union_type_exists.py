# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "surface"
# case = "union_type_exists"
# subject = "types.UnionType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: union_type_exists (surface)."""
import types

assert hasattr(types.UnionType, "__args__")
print("union_type_exists OK")
