# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "insert_places_at_index"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: insert places an element at the given index, including a negative index meaning 'before last'"""
import array

a = array.array("i", [1, 2, 3])
a.insert(1, 99)
assert a.tolist() == [1, 99, 2, 3], f"after insert = {a.tolist()!r}"
a.insert(-1, 88)  # insert before last
assert a[-2] == 88, f"insert before last = {a[-2]!r}"

print("insert_places_at_index OK")
