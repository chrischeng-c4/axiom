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


# Zero-argument super() outside any method has no implicit cell -> RuntimeError.
def free_super():
    super()  # type: ignore[call-arg]


try:
    free_super()
    print("super_no_args: no_raise")
except RuntimeError as e:
    print("super_no_args:", type(e).__name__, str(e)[:60])


# Accessing a name not present anywhere in the MRO raises AttributeError
# through the super proxy.
class Sub:
    def method(self):
        return super().missing  # type: ignore[attr-defined]


try:
    Sub().method()
    print("super_attr: no_raise")
except AttributeError as e:
    print("super_attr:", type(e).__name__, str(e)[:60])


# super()'s first argument must be a type, not an arbitrary value.
class BadFirst:
    def method(self):
        return super(1, self).method()  # type: ignore[arg-type]


try:
    BadFirst().method()
    print("super_bad_first: no_raise")
except TypeError as e:
    print("super_bad_first:", type(e).__name__, str(e)[:60])


# Calling super() with too many positional arguments raises TypeError.
try:
    super(int, int, int)  # type: ignore[call-arg]
    print("super_argcount: no_raise")
except TypeError as e:
    print("super_argcount:", type(e).__name__, str(e)[:60])


# The two-argument form still requires a type as its first argument.
try:
    super(1, int)  # type: ignore[arg-type]
    print("super_argtype: no_raise")
except TypeError as e:
    print("super_argtype:", type(e).__name__, str(e)[:60])
