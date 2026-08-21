# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "randint_inverted_range_raises_valueerror"
# subject = "random.randint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randint: randint_inverted_range_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.randint(10, 5)
except ValueError:
    _raised = True
assert _raised, "randint_inverted_range_raises_valueerror: expected ValueError"
print("randint_inverted_range_raises_valueerror OK")
