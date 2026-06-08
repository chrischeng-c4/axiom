# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "member_type_reflects_mixin"
# subject = "enum.IntEnum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.IntEnum: _member_type_ is int for IntEnum, str for a (str, Enum) mixin, and object for a plain Enum"""
import enum


class Num(enum.IntEnum):
    ONE = 100
    TWO = 200


class Word(str, enum.Enum):
    YARN = "soft"


class Plain(enum.Enum):
    VANILLA = "white"


assert Num._member_type_ is int, f"Num member_type = {Num._member_type_!r}"
assert Word._member_type_ is str, f"Word member_type = {Word._member_type_!r}"
assert Plain._member_type_ is object, f"Plain member_type = {Plain._member_type_!r}"

print("member_type_reflects_mixin OK")
