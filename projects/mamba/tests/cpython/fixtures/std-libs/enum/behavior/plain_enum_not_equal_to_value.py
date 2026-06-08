# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "plain_enum_not_equal_to_value"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: a plain Enum member is not equal to its raw value (Suit.CLUBS != 1) even though Suit.CLUBS.value == 1"""
import enum


class Suit(enum.Enum):
    CLUBS = 1
    DIAMONDS = 2


assert Suit.CLUBS != 1, "plain Enum member is not equal to its raw value"
assert Suit.CLUBS.value == 1, "the .value attribute still equals the raw value"

print("plain_enum_not_equal_to_value OK")
