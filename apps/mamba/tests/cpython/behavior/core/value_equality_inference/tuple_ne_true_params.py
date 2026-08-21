# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "tuple_ne_true_params"
# subject = "tuple != tuple as params is True when contents differ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: tuples that differ by one element are unequal (!= is True)."""


def differ(result, expect):
    assert result != expect, (result, expect)


differ((1, 2, 3), (1, 2, 4))
print("tuple_ne_true_params OK")
