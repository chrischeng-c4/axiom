# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "optional_is_union_with_none"
# subject = "typing.Optional"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Optional: Optional[X] is exactly Union[X,None]: typing.Optional[int]==Union[int,None] and get_args(Optional[int])==(int,type(None))"""
import typing
from typing import Union, get_args

# Optional[X] is exactly Union[X, None].
assert typing.Optional[int] == Union[int, None]
assert get_args(typing.Optional[int]) == (int, type(None))

print("optional_is_union_with_none OK")
