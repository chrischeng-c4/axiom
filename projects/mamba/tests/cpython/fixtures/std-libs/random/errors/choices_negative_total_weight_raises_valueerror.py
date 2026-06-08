# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_negative_total_weight_raises_valueerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_negative_total_weight_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.choices('ABC', weights=[3, -5, 1])
except ValueError:
    _raised = True
assert _raised, "choices_negative_total_weight_raises_valueerror: expected ValueError"
print("choices_negative_total_weight_raises_valueerror OK")
