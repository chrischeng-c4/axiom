# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "bytes_eq_params"
# subject = "bytes == bytes as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: distinct bytes objects with equal content compare equal by value."""


def check(result, expect):
    assert result == expect, (result, expect)


check(b"ab" + b"cd", b"abcd")
print("bytes_eq_params OK")
