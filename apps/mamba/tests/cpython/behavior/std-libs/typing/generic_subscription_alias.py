# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "generic_subscription_alias"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.Generic: subscripting a Generic[T] subclass (Box[int]) yields a parameterized alias whose get_origin is the Box class and get_args is (int,)"""
import typing

T = typing.TypeVar("T")


class Box(typing.Generic[T]):
    pass


alias = Box[int]
assert typing.get_origin(alias) is Box, "get_origin(Box[int]) should be the Box class"
assert typing.get_args(alias) == (int,), "get_args(Box[int]) should be (int,)"
print("generic_subscription_alias OK")
