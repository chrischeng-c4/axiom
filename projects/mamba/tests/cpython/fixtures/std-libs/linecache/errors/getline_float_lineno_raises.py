# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "getline_float_lineno_raises"
# subject = "linecache.getline"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline_float_lineno_raises (errors)."""
import linecache

_raised = False
try:
    linecache.getline(__file__, 1.1)
except TypeError:
    _raised = True
assert _raised, "getline_float_lineno_raises: expected TypeError"
print("getline_float_lineno_raises OK")
