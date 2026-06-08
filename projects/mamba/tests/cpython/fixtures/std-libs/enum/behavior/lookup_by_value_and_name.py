# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "lookup_by_value_and_name"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: Color(2) returns the member with value 2 and Color['BLUE'] returns the member named BLUE; both are the canonical singleton members"""
import enum


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


assert Color(2) is Color.GREEN, "lookup by value"
assert Color["BLUE"] is Color.BLUE, "lookup by name"

print("lookup_by_value_and_name OK")
