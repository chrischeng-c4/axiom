# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "errors"
# case = "missing_x_arg_raises_typeerror"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: missing_x_arg_raises_typeerror (errors)."""
import bisect

_raised = False
try:
    bisect.bisect_left([1, 2, 3])
except TypeError:
    _raised = True
assert _raised, "missing_x_arg_raises_typeerror: expected TypeError"
print("missing_x_arg_raises_typeerror OK")
