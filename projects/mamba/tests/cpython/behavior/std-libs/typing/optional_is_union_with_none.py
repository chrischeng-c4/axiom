# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "optional_is_union_with_none"
# subject = "typing.Optional"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.Optional: Optional[int] is exactly Union[int, None]; get_origin reports typing.Union and get_args reports (int, NoneType)"""
import typing

assert typing.Optional[int] == typing.Union[int, None], "Optional[int] == Union[int, None]"
assert typing.get_origin(typing.Optional[int]) is typing.Union, "get_origin(Optional[int]) is typing.Union"
assert typing.get_args(typing.Optional[int]) == (int, type(None)), "get_args(Optional[int]) == (int, NoneType)"
print("optional_is_union_with_none OK")
