# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "generic_subclass_instantiates"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.Generic: a Generic[T] subclass instantiates and stores its value at runtime; the type parameter is erased so behavior is plain Python"""
import typing

T = typing.TypeVar("T")


class Box(typing.Generic[T]):
    def __init__(self, value: T) -> None:
        self.value = value


b = Box(5)
assert b.value == 5, "Generic subclass should store its value"
s = Box("hello")
assert s.value == "hello", "the same Generic subclass works for any erased type"
print("generic_subclass_instantiates OK")
