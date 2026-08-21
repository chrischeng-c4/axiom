# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "nan_array_never_equal_or_ordered"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: two 'd' arrays each holding a single NaN follow IEEE-754: != is True, every other relation (==, <, <=, >, >=, even == itself) is False"""
import array

a = array.array("d", [float("nan")])
b = array.array("d", [float("nan")])
# Inequality is the only relation that holds.
assert (a != b) is True, "nan array != nan array"
# Every other relation is False.
assert (a == b) is False, "nan array not equal"
assert (a > b) is False, "nan array not greater"
assert (a >= b) is False, "nan array not greater-or-equal"
assert (a < b) is False, "nan array not less"
assert (a <= b) is False, "nan array not less-or-equal"
# An array even compares unequal to itself when it contains NaN.
assert (a == a) is False, "nan array unequal to itself"
assert (a != a) is True, "nan array != itself"

print("nan_array_never_equal_or_ordered OK")
