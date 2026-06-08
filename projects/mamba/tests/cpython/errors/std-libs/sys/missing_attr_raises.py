# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "missing_attr_raises"
# subject = "sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys: missing_attr_raises (errors)."""
import sys

_raised = False
try:
    sys.no_such_attr_xyzzy
except AttributeError:
    _raised = True
assert _raised, "missing_attr_raises: expected AttributeError"
print("missing_attr_raises OK")
