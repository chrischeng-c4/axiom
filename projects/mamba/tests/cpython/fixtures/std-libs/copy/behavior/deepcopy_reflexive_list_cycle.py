# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_reflexive_list_cycle"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of a self-referential list rebuilds the cycle: the copy points to itself, not to the original"""
import copy

cyclic = [1, 2]
cyclic.append(cyclic)  # self-referential
copied = copy.deepcopy(cyclic)
assert copied is not cyclic, "deepcopy is a new list"
assert copied[2] is copied, "cycle is preserved: the copy points to itself"
assert copied[2] is not cyclic, "the cycle does not leak back to the original"

print("deepcopy_reflexive_list_cycle OK")
