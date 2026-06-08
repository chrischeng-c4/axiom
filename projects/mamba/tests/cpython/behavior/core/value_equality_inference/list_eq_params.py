# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "list_eq_params"
# subject = "list == list inside def check(a, b)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated function params: list == list must compare by value (check pattern)."""


def check(result, expect):
    assert result == expect, (result, expect)


check([1, 2], [1, 2])
print("list_eq_params OK")
