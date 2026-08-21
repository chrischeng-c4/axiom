# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_repr_flattens"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: union repr flattens nesting and renders None for NoneType: int|str, int|str|list, int|(str|list), int|None, int|type(None)"""
import types  # noqa: F401

assert repr(int | str) == "int | str"
assert repr(int | str | list) == "int | str | list"
assert repr(int | (str | list)) == "int | str | list"
assert repr(int | None) == "int | None"
assert repr(int | type(None)) == "int | None"

print("union_repr_flattens OK")
