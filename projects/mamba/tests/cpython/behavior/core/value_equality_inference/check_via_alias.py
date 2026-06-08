# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "check_via_alias"
# subject = "value equality through an aliased helper call (g = helper; g(a, b))"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A helper bound to a new name still compares its unannotated args by value."""


def helper(result, expect):
    assert result == expect, (result, expect)


g = helper
g([1, 2], [1, 2])
g("abc", "ab" + "c")
print("check_via_alias OK")
