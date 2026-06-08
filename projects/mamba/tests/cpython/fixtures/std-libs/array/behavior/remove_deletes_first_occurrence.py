# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "remove_deletes_first_occurrence"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: remove deletes only the first matching occurrence, leaving later duplicates intact"""
import array

a = array.array("i", [1, 2, 3, 2, 4])
a.remove(2)
assert a.tolist() == [1, 3, 2, 4], f"remove first = {a.tolist()!r}"

print("remove_deletes_first_occurrence OK")
