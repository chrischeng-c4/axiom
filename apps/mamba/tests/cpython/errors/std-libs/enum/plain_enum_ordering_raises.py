# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "plain_enum_ordering_raises"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: plain (non-Int) Enum members are unordered: a < comparison between two members raises TypeError"""
import enum


class Season(enum.Enum):
    SPRING = 1
    SUMMER = 2
    AUTUMN = 3
    WINTER = 4


# Ordering comparisons between plain Enum members are not defined.
_raised = False
try:
    Season.SPRING < Season.WINTER
except TypeError:
    _raised = True
assert _raised, "ordering plain Enum members must raise TypeError"

# Equality is still well-defined (and a member is never equal to a raw value).
assert Season.SPRING != 1, "plain Enum member is not equal to its raw value"

print("plain_enum_ordering_raises OK")
