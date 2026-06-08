# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "errors"
# case = "fnmatch_int_pattern_raises"
# subject = "fnmatch.fnmatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.fnmatch: fnmatch_int_pattern_raises (errors)."""
import fnmatch

_raised = False
try:
    fnmatch.fnmatch("a.py", 123)
except TypeError:
    _raised = True
assert _raised, "fnmatch_int_pattern_raises: expected TypeError"
print("fnmatch_int_pattern_raises OK")
