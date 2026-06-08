# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "accumulate_non_numeric_op"
# subject = "itertools.accumulate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.accumulate: accumulate works with a non-numeric binary op: operator.is_ over [None,None,None] gives [None, True, False]"""
import itertools

import operator
acc_is = list(itertools.accumulate([None, None, None], operator.is_))
assert acc_is == [None, True, False], f"accumulate is_ = {acc_is!r}"

print("accumulate_non_numeric_op OK")
