# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_memo_shares_repeated_ref"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: the memo makes two references to the same object inside one structure stay shared (and equal a single new object) after deepcopy"""
import copy

shared = [99]
container = [shared, shared]  # both elements point to the same list
deep = copy.deepcopy(container)
assert deep[0] is deep[1], "memo keeps the repeated reference shared in the copy"
assert deep[0] is not shared, "the shared element is a fresh copy, not the original"

print("deepcopy_memo_shares_repeated_ref OK")
