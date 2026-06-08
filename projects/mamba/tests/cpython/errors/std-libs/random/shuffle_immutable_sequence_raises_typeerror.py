# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "shuffle_immutable_sequence_raises_typeerror"
# subject = "random.shuffle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.shuffle: shuffle_immutable_sequence_raises_typeerror (errors)."""
import random

_raised = False
try:
    random.shuffle('string is immutable')
except TypeError:
    _raised = True
assert _raised, "shuffle_immutable_sequence_raises_typeerror: expected TypeError"
print("shuffle_immutable_sequence_raises_typeerror OK")
