# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "accumulate_running_fold"
# subject = "itertools.accumulate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.accumulate: accumulate yields running results: default add, a custom binary op (mul), and a running-max lambda"""
import itertools

import operator
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10], "accumulate sum"
assert list(itertools.accumulate([1, 2, 3, 4], operator.mul)) == [1, 2, 6, 24], "accumulate mul"
assert list(itertools.accumulate([3, 1, 4, 1, 5], lambda a, b: a if a > b else b)) == [3, 3, 4, 4, 5], "running max"
assert list(itertools.accumulate([10])) == [10], "single element"

print("accumulate_running_fold OK")
