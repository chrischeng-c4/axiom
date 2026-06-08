# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "dict_eq_params"
# subject = "dict == dict as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: dicts with equal key/value pairs compare equal by value."""


def check(result, expect):
    assert result == expect, (result, expect)


check({"a": 1, "b": 2}, {"a": 1, "b": 2})
print("dict_eq_params OK")
