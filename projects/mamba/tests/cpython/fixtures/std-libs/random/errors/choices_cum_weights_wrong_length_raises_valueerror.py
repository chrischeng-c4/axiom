# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_cum_weights_wrong_length_raises_valueerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_cum_weights_wrong_length_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.choices([1, 2, 3], cum_weights=[1, 2])
except ValueError:
    _raised = True
assert _raised, "choices_cum_weights_wrong_length_raises_valueerror: expected ValueError"
print("choices_cum_weights_wrong_length_raises_valueerror OK")
