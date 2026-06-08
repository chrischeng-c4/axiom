# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "randrange_empty_range_raises_valueerror"
# subject = "random.randrange"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randrange: randrange_empty_range_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.randrange(5, 5)
except ValueError:
    _raised = True
assert _raised, "randrange_empty_range_raises_valueerror: expected ValueError"
print("randrange_empty_range_raises_valueerror OK")
