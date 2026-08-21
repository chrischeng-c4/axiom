# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "repeat_tiles_elements"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: the * operator tiles the elements n times into a new array"""
import array

tiled = array.array("i", [1, 2]) * 3
assert tiled.tolist() == [1, 2, 1, 2, 1, 2], f"repeat = {tiled.tolist()!r}"

print("repeat_tiles_elements OK")
