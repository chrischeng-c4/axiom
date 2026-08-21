# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_tuple_with_mutable_rebuilt"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of a tuple containing a list returns a new tuple whose inner list is independent of the original"""
import copy

original = (1, [2, 3])
deep = copy.deepcopy(original)

# Mutating the original's inner list does not change the deep copy.
original[1].append(99)
assert original == (1, [2, 3, 99]), f"original inner list mutated: {original!r}"
assert deep == (1, [2, 3]), f"deep copy's inner list is independent: {deep!r}"

print("deepcopy_tuple_with_mutable_rebuilt OK")
