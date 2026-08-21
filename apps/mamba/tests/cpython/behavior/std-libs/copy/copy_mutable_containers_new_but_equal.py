# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "copy_mutable_containers_new_but_equal"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: shallow copy of bytearray/set/list/dict (including empty ones) yields a distinct object that compares equal, with mutable members still shared"""
import copy

# bytearray: shallow copy is a new, equal buffer.
ba = bytearray(b"abc")
ba_c = copy.copy(ba)
assert ba_c == ba and ba_c is not ba, "bytearray copy new-but-equal"
empty_ba = copy.copy(bytearray())
assert empty_ba == bytearray() and empty_ba is not bytearray(), "empty bytearray copy"

# set: shallow copy is a new, equal set.
s = {1, 2, 3}
assert copy.copy(s) == s and copy.copy(s) is not s, "set copy new-but-equal"
assert copy.copy(set()) == set(), "empty set copy"

# list: shallow copy new outer, equal contents.
ls = [1, 2, 3]
assert copy.copy(ls) == ls and copy.copy(ls) is not ls, "list copy new-but-equal"
assert copy.copy([]) == [], "empty list copy"

# dict: shallow copy new outer, equal contents.
d = {"foo": 1, "bar": 2}
assert copy.copy(d) == d and copy.copy(d) is not d, "dict copy new-but-equal"
assert copy.copy({}) == {}, "empty dict copy"

# Shallow copy shares mutable members; deepcopy does not.
nested = {"x": [1, 2], "y": [3, 4]}
assert copy.copy(nested)["x"] is nested["x"], "shallow dict shares inner list"
deep = copy.deepcopy(nested)
assert deep["x"] is not nested["x"], "deep dict copies inner list"
deep["x"].append(99)
assert nested["x"] == [1, 2], "deep copy is independent of the original"

print("copy_mutable_containers_new_but_equal OK")
