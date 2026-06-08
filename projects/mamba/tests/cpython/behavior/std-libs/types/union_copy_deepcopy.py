# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_copy_deepcopy"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: copy.copy and copy.deepcopy of a parameterized union (list[T] | int) preserve its __args__ and __parameters__"""
import copy
import types  # noqa: F401
import typing

T = typing.TypeVar("T")
orig = list[T] | int
for clone in (copy.copy(orig), copy.deepcopy(orig)):
    assert clone == orig
    assert clone.__args__ == orig.__args__
    assert clone.__parameters__ == orig.__parameters__

print("union_copy_deepcopy OK")
