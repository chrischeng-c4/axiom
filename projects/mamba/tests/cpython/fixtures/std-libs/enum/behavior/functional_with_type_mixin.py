# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "functional_with_type_mixin"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: Enum('Name', '...', type=int) mixes in a data type: members equal their int value, lookup by value works, and membership works"""
import enum

Minor = enum.Enum("Minor", "june july august", type=int)

assert len(Minor) == 3, f"len = {len(Minor)!r}"
assert Minor.june == 1, "int mixin member equals its value"
assert Minor(2) is Minor.july, "lookup by value"
assert Minor.august in Minor, "membership test"

print("functional_with_type_mixin OK")
