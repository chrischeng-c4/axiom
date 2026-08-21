# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "errors"
# case = "mixed_types_raise_typeerror"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: mixed_types_raise_typeerror (errors)."""
import bisect

_raised = False
try:
    bisect.bisect_left([1, 2, 3], "x")
except TypeError:
    _raised = True
assert _raised, "mixed_types_raise_typeerror: expected TypeError"
print("mixed_types_raise_typeerror OK")
