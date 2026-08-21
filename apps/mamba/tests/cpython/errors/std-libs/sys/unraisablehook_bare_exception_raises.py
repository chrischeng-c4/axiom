# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "unraisablehook_bare_exception_raises"
# subject = "sys.__unraisablehook__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__unraisablehook__: unraisablehook_bare_exception_raises (errors)."""
import sys

_raised = False
try:
    sys.__unraisablehook__(ValueError(42))
except TypeError:
    _raised = True
assert _raised, "unraisablehook_bare_exception_raises: expected TypeError"
print("unraisablehook_bare_exception_raises OK")
