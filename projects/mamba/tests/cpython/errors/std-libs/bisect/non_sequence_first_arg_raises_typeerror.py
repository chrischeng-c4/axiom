# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "errors"
# case = "non_sequence_first_arg_raises_typeerror"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: non_sequence_first_arg_raises_typeerror (errors)."""
import bisect

_raised = False
try:
    bisect.bisect_left(10, 10)
except TypeError:
    _raised = True
assert _raised, "non_sequence_first_arg_raises_typeerror: expected TypeError"
print("non_sequence_first_arg_raises_typeerror OK")
