# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "errors"
# case = "format_exception_mixed_args_raises"
# subject = "traceback.format_exception"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: format_exception_mixed_args_raises (errors)."""
import traceback

_raised = False
try:
    traceback.format_exception(Exception, Exception('x'))
except ValueError:
    _raised = True
assert _raised, "format_exception_mixed_args_raises: expected ValueError"
print("format_exception_mixed_args_raises OK")
