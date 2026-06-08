# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "in_list_membership"
# subject = "in-operator membership tests by value over a list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""The in operator finds a member by value, not by identity."""
needle = "b" + "c"
haystack = ["ab", "bc", "cd"]
assert (needle in haystack) is True, needle
assert ("zz" in haystack) is False
print("in_list_membership OK")
