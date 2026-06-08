# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/<area>: class / dunder error paths (CPython 3.12 oracle)."""


# Missing attribute raises AttributeError.
class C:
    pass


try:
    C().no_such_attr  # type: ignore[attr-defined]
    print("missing_attr: no_raise")
except AttributeError as e:
    print("missing_attr:", type(e).__name__, str(e)[:60])


# Setting attr on slotted class without that name raises AttributeError.
class Slotted:
    __slots__ = ("x",)


s = Slotted()
s.x = 1
try:
    s.y = 2  # type: ignore[attr-defined]
    print("slot_unknown: no_raise")
except AttributeError as e:
    print("slot_unknown:", type(e).__name__, str(e)[:60])


# Calling an abstract class raises TypeError.
from abc import ABC, abstractmethod


class A(ABC):
    @abstractmethod
    def method(self) -> int: ...


try:
    A()  # type: ignore[abstract]
    print("abstract: no_raise")
except TypeError as e:
    print("abstract:", type(e).__name__, str(e)[:60])
