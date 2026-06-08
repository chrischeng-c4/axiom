# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "shallow_list_shares_inner"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: shallow copy of a nested list makes a new outer list but shares the inner sublists"""
import copy

original = [1, [2, 3], 4]
shallow = copy.copy(original)
assert shallow == original and shallow is not original, "shallow outer is new"
assert shallow[1] is original[1], "shallow inner sublist is shared"

print("shallow_list_shares_inner OK")
