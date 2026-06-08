# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "get_args_yields_param_tuple"
# subject = "typing.get_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_args: get_args yields the parameter tuple in declaration order: (int,) for List[int], (int,str) for Union[int,str], (1,2,3) for Literal[1,2,3], (int,Ellipsis) for Tuple[int,...], () for a bare int, and ([int,str],bool) for Callable[[int,str],bool]"""
from typing import Callable, List, Literal, Tuple, Union, get_args

# get_args yields the parameter tuple in declaration order.
assert get_args(List[int]) == (int,)
assert get_args(Union[int, str]) == (int, str)
assert get_args(Literal[1, 2, 3]) == (1, 2, 3)
assert get_args(Tuple[int, ...]) == (int, Ellipsis)
assert get_args(int) == ()
assert get_args(Callable[[int, str], bool]) == ([int, str], bool)

print("get_args_yields_param_tuple OK")
