# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "get_origin_list_is_list"
# subject = "typing.get_origin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.get_origin: get_origin(List[int]) is the runtime class list and get_origin(Dict[str, int]) is dict"""
import typing

assert typing.get_origin(typing.List[int]) is list, "get_origin(List[int]) should be list"
assert typing.get_origin(typing.Dict[str, int]) is dict, "get_origin(Dict[str, int]) should be dict"
print("get_origin_list_is_list OK")
