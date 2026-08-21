# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Reassigning instance.__class__ with layout compatibility rules (CPython 3.12)."""


# Two plain classes share a compatible instance layout, so __class__ can be
# swapped back and forth and attributes survive.
class C:
    pass


class D:
    pass


x = C()
x.a = 1
x.__class__ = D
assert x.__class__ is D
assert isinstance(x, D)
assert x.a == 1                  # instance attribute preserved across swap
x.__class__ = C
assert x.__class__ is C
assert x.a == 1


# Multiple-inheritance siblings with compatible layout also swap.
class Base:
    pass


class L(Base):
    pass


class R(Base):
    pass


y = L()
y.__class__ = R
assert type(y).__name__ == "R"


# Incompatible layouts (user class vs list) raise TypeError.
try:
    C().__class__ = list
    print("to_list: no_raise")
except TypeError as e:
    print("to_list: TypeError", str(e)[:40])

# Built-in immutable instances reject __class__ reassignment.
class MyStr(str):
    __slots__ = ()


try:
    "a".__class__ = MyStr
    print("str_imm: no_raise")
except TypeError:
    print("str_imm: TypeError")

print("set_class OK")
