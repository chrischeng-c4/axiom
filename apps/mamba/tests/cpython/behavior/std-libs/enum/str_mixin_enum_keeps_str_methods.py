# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "str_mixin_enum_keeps_str_methods"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: a plain (str, Enum) mixin equals its str value and inherits str methods, but str() shows the qualified 'Class.NAME' (unlike StrEnum)"""
import enum

class Direction(str, enum.Enum):
    EAST = "east"
    WEST = "west"

assert Direction.EAST == "east"
assert Direction.EAST.upper() == "EAST"
assert Direction.EAST.startswith("e")
assert str(Direction.EAST) == "Direction.EAST"   # qualified, unlike StrEnum
assert isinstance(Direction.EAST, str)

print("str_mixin_enum_keeps_str_methods OK")
