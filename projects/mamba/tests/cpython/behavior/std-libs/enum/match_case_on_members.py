# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "match_case_on_members"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: an Enum member works as a PEP 634 match/case pattern, including an OR-pattern (case A | B) and the wildcard fallback"""
import enum


class Suit(enum.Enum):
    CLUBS = 1
    DIAMONDS = 2
    HEARTS = 3
    SPADES = 4


def describe(s):
    match s:
        case Suit.HEARTS | Suit.DIAMONDS:
            return "red"
        case _:
            return "black"


assert describe(Suit.HEARTS) == "red", "OR-pattern matches HEARTS"
assert describe(Suit.DIAMONDS) == "red", "OR-pattern matches DIAMONDS"
assert describe(Suit.CLUBS) == "black", "wildcard fallback for CLUBS"

print("match_case_on_members OK")
