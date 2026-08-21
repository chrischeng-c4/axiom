# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "excepthook_no_arg_raises"
# subject = "sys.__excepthook__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__excepthook__: excepthook_no_arg_raises (errors)."""
import sys

_raised = False
try:
    sys.__excepthook__()
except TypeError:
    _raised = True
assert _raised, "excepthook_no_arg_raises: expected TypeError"
print("excepthook_no_arg_raises OK")
