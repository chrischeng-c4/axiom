# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "errors"
# case = "insort_tuple_raises_attributeerror"
# subject = "bisect.insort"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort: insort_tuple_raises_attributeerror (errors)."""
import bisect

_raised = False
try:
    bisect.insort((1, 2, 3), 4)
except AttributeError:
    _raised = True
assert _raised, "insort_tuple_raises_attributeerror: expected AttributeError"
print("insort_tuple_raises_attributeerror OK")
