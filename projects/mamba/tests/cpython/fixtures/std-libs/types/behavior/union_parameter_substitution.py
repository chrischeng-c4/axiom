# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_parameter_substitution"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: free type vars chain through subscription/substitution: (float | list[T])[int], list[T]|list[S] parameters and substitution"""
import types  # noqa: F401
import typing

T = typing.TypeVar("T")
S = typing.TypeVar("S")
assert (float | list[T])[int] == float | list[int]
assert (list[T] | list[S]).__parameters__ == (T, S)
assert (list[T] | list[S])[int, T] == list[int] | list[T]

print("union_parameter_substitution OK")
