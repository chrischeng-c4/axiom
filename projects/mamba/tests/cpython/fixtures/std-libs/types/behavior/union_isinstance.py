# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_isinstance"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: isinstance against a runtime union behaves like a tuple of types: isinstance(5, int|str) and isinstance('x', int|str) but not isinstance(1.5, int|str)"""
import types  # noqa: F401

assert isinstance(5, int | str)
assert isinstance("x", int | str)
assert not isinstance(1.5, int | str)

print("union_isinstance OK")
