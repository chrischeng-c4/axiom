# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "functional_from_space_string"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: Enum('Name', 'a b c') builds members from a space-separated string, numbered from 1 in order"""
import enum

Months = enum.Enum("Months", "jan feb mar")

assert [m.name for m in Months] == ["jan", "feb", "mar"], "names in order"
assert Months.jan.value == 1, f"jan = {Months.jan.value!r}"
assert Months.mar.value == 3, f"mar = {Months.mar.value!r}"

print("functional_from_space_string OK")
