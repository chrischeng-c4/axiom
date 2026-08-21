# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_hash_order_insensitive"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: unions are order-insensitive for hashing/equality and equal to the typing.Union of the same members"""
import types  # noqa: F401
import typing

assert hash(int | str) == hash(str | int)
assert hash(int | str) == hash(typing.Union[int, str])
assert (int | str) == (str | int)
assert (int | str) == typing.Union[int, str]

print("union_hash_order_insensitive OK")
