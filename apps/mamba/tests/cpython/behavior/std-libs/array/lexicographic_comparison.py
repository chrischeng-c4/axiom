# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "lexicographic_comparison"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: arrays compare lexicographically element by element, with a longer prefix-equal array comparing greater"""
import array

assert array.array("i", [1, 2]) == array.array("i", [1, 2]), "equal"
assert array.array("i", [1, 2]) < array.array("i", [1, 3]), "less-than"
assert array.array("i", [1, 2, 3]) > array.array("i", [1, 2]), "longer prefix-equal is greater"

print("lexicographic_comparison OK")
