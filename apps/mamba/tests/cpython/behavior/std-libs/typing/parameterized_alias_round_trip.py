# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "parameterized_alias_round_trip"
# subject = "typing.List"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.List: List[int] and Final[int] are subscriptable special forms; List[int] round-trips through get_origin (list) and get_args ((int,))"""
import typing

list_alias = typing.List[int]
assert typing.get_origin(list_alias) is list, "get_origin(List[int]) is list"
assert typing.get_args(list_alias) == (int,), "get_args(List[int]) is (int,)"

# Final[int] is a subscriptable special form too; subscription must not raise.
final_alias = typing.Final[int]
assert final_alias is not None, "Final[int] should be a usable special form"
print("parameterized_alias_round_trip OK")
