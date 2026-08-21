# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_empty_population_raises_indexerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_empty_population_raises_indexerror (errors)."""
import random

_raised = False
try:
    random.choices([], k=3)
except IndexError:
    _raised = True
assert _raised, "choices_empty_population_raises_indexerror: expected IndexError"
print("choices_empty_population_raises_indexerror OK")
