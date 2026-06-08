# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "repeat_non_int_times_raises"
# subject = "itertools.repeat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.repeat: repeat_non_int_times_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.repeat('x', 'y'))
except TypeError:
    _raised = True
assert _raised, "repeat_non_int_times_raises: expected TypeError"
print("repeat_non_int_times_raises OK")
