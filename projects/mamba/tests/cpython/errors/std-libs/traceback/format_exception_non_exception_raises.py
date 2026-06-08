# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "errors"
# case = "format_exception_non_exception_raises"
# subject = "traceback.format_exception"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: format_exception_non_exception_raises (errors)."""
import traceback

_raised = False
try:
    traceback.format_exception(42)
except TypeError:
    _raised = True
assert _raised, "format_exception_non_exception_raises: expected TypeError"
print("format_exception_non_exception_raises OK")
