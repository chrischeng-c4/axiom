# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "str_eq_params"
# subject = "str == str as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: distinct str objects with equal content compare equal by value."""


def check(result, expect):
    assert result == expect, (result, expect)


check("hello" + "", "hello")
check("".join(["ab", "cd"]), "abcd")
print("str_eq_params OK")
