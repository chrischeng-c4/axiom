# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "members_are_singletons"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: value lookup, name lookup, and attribute access all return the identical singleton object (Suit(1) is Suit['CLUBS'] is Suit.CLUBS)"""
import enum


class Suit(enum.Enum):
    CLUBS = 1
    DIAMONDS = 2
    HEARTS = 3
    SPADES = 4


assert Suit.CLUBS is Suit.CLUBS, "attribute access is a singleton"
assert Suit(1) is Suit.CLUBS, "value lookup returns the same singleton"
assert Suit["CLUBS"] is Suit.CLUBS, "name lookup returns the same singleton"

print("members_are_singletons OK")
