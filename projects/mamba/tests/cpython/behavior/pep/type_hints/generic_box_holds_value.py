# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "generic_box_holds_value"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: a Generic[T] subclass is an ordinary class at runtime: _Box(Generic[T]) stores and returns any value, _Box(42).get()==42 and _Box('hello').get()=='hello'"""
import typing
from typing import Generic, TypeVar

T = TypeVar("T")


class _Box(Generic[T]):
    def __init__(self, value: T):
        self.value = value

    def get(self) -> T:
        return self.value


assert _Box(42).get() == 42, f"generic get = {_Box(42).get()!r}"
assert _Box("hello").get() == "hello", f"generic str = {_Box('hello').get()!r}"

print("generic_box_holds_value OK")
