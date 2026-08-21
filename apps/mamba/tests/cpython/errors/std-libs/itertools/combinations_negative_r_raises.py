# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "combinations_negative_r_raises"
# subject = "itertools.combinations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.combinations: combinations_negative_r_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.combinations([1, 2, 3], -1))
except ValueError:
    _raised = True
assert _raised, "combinations_negative_r_raises: expected ValueError"
print("combinations_negative_r_raises OK")
