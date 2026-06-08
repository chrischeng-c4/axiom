# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "get_args_unpacks_parameters"
# subject = "typing.get_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.get_args: get_args unpacks the type parameters: List[int] -> (int,), Dict[str, int] -> (str, int), Union[int, str] -> (int, str)"""
import typing

assert typing.get_args(typing.List[int]) == (int,), "get_args(List[int])"
assert typing.get_args(typing.Dict[str, int]) == (str, int), "get_args(Dict[str, int])"
assert typing.get_args(typing.Union[int, str]) == (int, str), "get_args(Union[int, str])"
print("get_args_unpacks_parameters OK")
