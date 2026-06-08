# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "empty_array_ops_stay_empty"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: self slice-assign, concat, repeat and in-place concat on a zero-length array all leave it empty rather than raising; empty tolist()/tobytes() are empty"""
import array

a = array.array("B")
# Assigning an array to its own full slice is a no-op on an empty array.
a[:] = a
assert len(a) == 0, f"after self slice-assign = {len(a)!r}"
# Concatenation, repetition, in-place concat of empties stay empty.
assert len(a + a) == 0, f"empty + empty = {len(a + a)!r}"
assert len(a * 3) == 0, f"empty * 3 = {len(a * 3)!r}"
assert len(a * 0) == 0, f"empty * 0 = {len(a * 0)!r}"
a += a
assert len(a) == 0, f"empty += empty = {len(a)!r}"
# tolist and tobytes of an empty array are empty too.
assert a.tolist() == [], f"empty tolist = {a.tolist()!r}"
assert a.tobytes() == b"", f"empty tobytes = {a.tobytes()!r}"

print("empty_array_ops_stay_empty OK")
