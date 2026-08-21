# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "functional_from_iterable_and_pairs"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: Enum accepts a list of names (numbered from 1) and a list of (name, value) pairs (explicit values)"""
import enum

# A list of names: values number from 1.
FromList = enum.Enum("FromList", ["x", "y", "z"])
assert FromList.y.value == 2, f"y = {FromList.y.value!r}"

# A list of (name, value) pairs: values are explicit.
FromPairs = enum.Enum("FromPairs", [("LO", 10), ("HI", 20)])
assert FromPairs.LO.value == 10, f"LO = {FromPairs.LO.value!r}"
assert FromPairs.HI.value == 20, f"HI = {FromPairs.HI.value!r}"

print("functional_from_iterable_and_pairs OK")
