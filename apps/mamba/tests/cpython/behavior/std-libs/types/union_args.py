# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_args"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: __args__ exposes the union member types in source order: (int | str).__args__ == (int, str)"""
import types  # noqa: F401

assert (int | str).__args__ == (int, str)
assert (str | int).__args__ == (str, int)

print("union_args OK")
