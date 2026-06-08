# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/class_system: class / dunder error paths (CPython 3.12 oracle)."""


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


# __init__ must return None; a non-None return raises TypeError.
class Bad:
    def __init__(self):
        return 10  # type: ignore[return-value]


try:
    Bad()
    print("init_return: no_raise")
except TypeError as e:
    print("init_return:", type(e).__name__, str(e)[:60])


# object.__new__ rejects extra args when the class does not override __new__.
class Plain:
    pass


try:
    object.__new__(Plain, 5)
    print("new_extra_args: no_raise")
except TypeError as e:
    print("new_extra_args:", type(e).__name__, str(e)[:60])


# Duplicate base class in a type() call raises TypeError.
try:
    type("Dup", (C, C), {})
    print("dup_base: no_raise")
except TypeError as e:
    print("dup_base:", type(e).__name__, str(e)[:60])
