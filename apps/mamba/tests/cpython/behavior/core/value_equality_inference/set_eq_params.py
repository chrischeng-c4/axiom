# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "set_eq_params"
# subject = "set == set as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: sets with equal members compare equal by value, order-independent."""


def check(result, expect):
    assert result == expect, (result, expect)


check({1, 2, 3}, {3, 2, 1})
print("set_eq_params OK")
