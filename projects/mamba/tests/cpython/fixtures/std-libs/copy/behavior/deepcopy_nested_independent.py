# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_nested_independent"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of a doubly-nested dict makes every level an independent object so mutating a deep value leaves the original untouched"""
import copy

original = {"a": [1, [2, 3]], "b": {"c": [4, 5]}}
deep = copy.deepcopy(original)
assert deep == original and deep is not original, "deep outer is new"
assert deep["a"] is not original["a"], "deep copies the inner list"
assert deep["b"]["c"] is not original["b"]["c"], "deep copies the doubly-nested list"

deep["a"].append(99)
assert original["a"] == [1, [2, 3]], "deep copy is independent of the original"

print("deepcopy_nested_independent OK")
