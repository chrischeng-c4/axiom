# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "errors"
# case = "negative_lo_raises_valueerror"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: negative_lo_raises_valueerror (errors)."""
import bisect

_raised = False
try:
    bisect.bisect_left([1, 2, 3], 2, lo=-1)
except ValueError:
    _raised = True
assert _raised, "negative_lo_raises_valueerror: expected ValueError"
print("negative_lo_raises_valueerror OK")
