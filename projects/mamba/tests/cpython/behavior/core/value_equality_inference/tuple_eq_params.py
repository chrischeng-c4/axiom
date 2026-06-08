# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "tuple_eq_params"
# subject = "tuple == tuple as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: tuples with equal elements compare equal by value."""


def check(result, expect):
    assert result == expect, (result, expect)


check((1, 2, 3), (1, 2, 3))
check(tuple([4, 5]), (4, 5))
print("tuple_eq_params OK")
