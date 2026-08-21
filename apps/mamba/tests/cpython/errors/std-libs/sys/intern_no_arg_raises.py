# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "intern_no_arg_raises"
# subject = "sys.intern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.intern: intern_no_arg_raises (errors)."""
import sys

_raised = False
try:
    sys.intern()
except TypeError:
    _raised = True
assert _raised, "intern_no_arg_raises: expected TypeError"
print("intern_no_arg_raises OK")
