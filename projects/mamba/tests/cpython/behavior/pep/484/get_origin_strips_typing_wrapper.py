# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "get_origin_strips_typing_wrapper"
# subject = "typing.get_origin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_origin: get_origin strips the typing wrapper to its underlying origin: a Generic subclass for Box[int], list for List[int]/list[int], tuple for Tuple[int,str], Union for Union[int,str], Literal/ClassVar/Final/Annotated for their own forms, collections.abc.Callable for Callable[[int],str], type(int|str) for int|str, and None for a bare int"""
import collections.abc
import typing
from typing import (
    Annotated, Callable, ClassVar, Final, Generic, List, Literal, Tuple,
    TypeVar, Union, get_origin,
)

T = TypeVar("T")


class Box(Generic[T]):
    pass


# get_origin strips the typing wrapper to its underlying origin.
assert get_origin(Box[int]) is Box
assert get_origin(int) is None
assert get_origin(List[int]) is list
assert get_origin(Tuple[int, str]) is tuple
assert get_origin(Union[int, str]) is Union
assert get_origin(Literal[42, 43]) is Literal
assert get_origin(ClassVar[int]) is ClassVar
assert get_origin(Final[int]) is Final
assert get_origin(Annotated[int, "tag"]) is Annotated
assert get_origin(Callable[[int], str]) is collections.abc.Callable
assert get_origin(list[int]) is list
assert get_origin(int | str) is type(int | str)

print("get_origin_strips_typing_wrapper OK")
