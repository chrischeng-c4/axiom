# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "subclass_with_members_raises"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: subclassing an Enum that already has members raises TypeError (a populated Enum is final), while subclassing a memberless Enum is allowed"""
import enum


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


# Subclassing a populated Enum to add members raises TypeError.
_raised = False
try:
    class More(Color):
        YELLOW = 4
except TypeError:
    _raised = True
assert _raised, "subclassing a populated Enum must raise TypeError"


# Subclassing a memberless Enum (mixin/base only) is allowed.
class Base(enum.Enum):
    def describe(self):
        return self.name


class Sub(Base):
    A = 1
    B = 2


assert Sub.A.describe() == "A", "subclass of memberless Enum works"

print("subclass_with_members_raises OK")
