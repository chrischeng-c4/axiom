# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "append_pop_lifo_at_end"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: append adds at the end and pop()/pop(0) remove from end/front like a list"""
import array

a = array.array("i", [1, 2, 3])
a.append(4)
assert a[-1] == 4, f"appended at end = {a[-1]!r}"
last = a.pop()
assert last == 4, f"popped last = {last!r}"
first = a.pop(0)
assert first == 1, f"pop(0) = {first!r}"
assert a.tolist() == [2, 3], f"after two pops = {a.tolist()!r}"

print("append_pop_lifo_at_end OK")
