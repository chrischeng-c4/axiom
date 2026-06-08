# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "getrecursionlimit_extra_arg_raises"
# subject = "sys.getrecursionlimit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getrecursionlimit: getrecursionlimit_extra_arg_raises (errors)."""
import sys

_raised = False
try:
    sys.getrecursionlimit(42)
except TypeError:
    _raised = True
assert _raised, "getrecursionlimit_extra_arg_raises: expected TypeError"
print("getrecursionlimit_extra_arg_raises OK")
