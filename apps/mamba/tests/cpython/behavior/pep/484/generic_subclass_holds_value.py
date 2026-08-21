# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "generic_subclass_holds_value"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: a Generic[T] subclass is an ordinary class at runtime: Container(typing.Generic[T]) stores and returns its value, Container(42).value==42"""
import typing

T = typing.TypeVar("T")


class Container(typing.Generic[T]):
    def __init__(self, value: T) -> None:
        self.value = value


# A Generic subclass is an ordinary class at runtime.
assert Container(42).value == 42

print("generic_subclass_holds_value OK")
