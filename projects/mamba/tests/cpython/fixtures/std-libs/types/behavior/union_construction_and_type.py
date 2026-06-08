# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_construction_and_type"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: int | str produces a UnionType instance: isinstance(u, types.UnionType) and type(u) is types.UnionType"""
import types

u = int | str
assert isinstance(u, types.UnionType)
assert type(u) is types.UnionType

print("union_construction_and_type OK")
