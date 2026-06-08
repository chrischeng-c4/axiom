# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "partial_no_args_raises"
# subject = "functools.partial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial_no_args_raises (errors)."""
import functools

_raised = False
try:
    functools.partial()
except TypeError:
    _raised = True
assert _raised, "partial_no_args_raises: expected TypeError"
print("partial_no_args_raises OK")
