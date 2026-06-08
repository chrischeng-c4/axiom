# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "sample_k_too_large_raises_valueerror"
# subject = "random.sample"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.sample: sample_k_too_large_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.sample([1, 2], 5)
except ValueError:
    _raised = True
assert _raised, "sample_k_too_large_raises_valueerror: expected ValueError"
print("sample_k_too_large_raises_valueerror OK")
