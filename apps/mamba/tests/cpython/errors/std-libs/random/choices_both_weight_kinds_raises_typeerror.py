# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_both_weight_kinds_raises_typeerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_both_weight_kinds_raises_typeerror (errors)."""
import random

_raised = False
try:
    random.choices([1, 2], weights=[1, 1], cum_weights=[1, 2])
except TypeError:
    _raised = True
assert _raised, "choices_both_weight_kinds_raises_typeerror: expected TypeError"
print("choices_both_weight_kinds_raises_typeerror OK")
