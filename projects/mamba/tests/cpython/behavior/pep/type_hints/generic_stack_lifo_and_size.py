# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "generic_stack_lifo_and_size"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: a Generic[T] _Stack subclass behaves as a normal LIFO container: push 1,2,3 then pop()==3 and size()==2"""
import typing
from typing import Generic, List, TypeVar

T = TypeVar("T")


class _Stack(Generic[T]):
    def __init__(self):
        self._items: List = []

    def push(self, item: T) -> None:
        self._items.append(item)

    def pop(self) -> T:
        return self._items.pop()

    def size(self) -> int:
        return len(self._items)


_s: _Stack[int] = _Stack()
_s.push(1)
_s.push(2)
_s.push(3)
assert _s.pop() == 3, "pop"
assert _s.size() == 2, f"size = {_s.size()!r}"

print("generic_stack_lifo_and_size OK")
