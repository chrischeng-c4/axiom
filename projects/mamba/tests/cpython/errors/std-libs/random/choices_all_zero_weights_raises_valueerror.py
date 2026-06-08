# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_all_zero_weights_raises_valueerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_all_zero_weights_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.choices('AB', weights=[0.0, 0.0])
except ValueError:
    _raised = True
assert _raised, "choices_all_zero_weights_raises_valueerror: expected ValueError"
print("choices_all_zero_weights_raises_valueerror OK")
