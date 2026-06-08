# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choice_empty_raises_indexerror"
# subject = "random.choice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choice: choice_empty_raises_indexerror (errors)."""
import random

_raised = False
try:
    random.choice([])
except IndexError:
    _raised = True
assert _raised, "choice_empty_raises_indexerror: expected IndexError"
print("choice_empty_raises_indexerror OK")
