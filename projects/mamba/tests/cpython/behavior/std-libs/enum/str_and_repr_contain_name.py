# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "str_and_repr_contain_name"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: str(member) and repr(member) both surface the member name (CPython 3.12 str is 'Class.NAME', repr is '<Class.NAME: value>')"""
import enum


class Suit(enum.Enum):
    CLUBS = 1
    HEARTS = 3


# str() surfaces the qualified 'Class.NAME'; the name is always present.
assert "CLUBS" in str(Suit.CLUBS), f"str contains name: {str(Suit.CLUBS)!r}"
# repr() surfaces both the name and the value.
r = repr(Suit.CLUBS)
assert "CLUBS" in r and "1" in r, f"repr = {r!r}"

print("str_and_repr_contain_name OK")
